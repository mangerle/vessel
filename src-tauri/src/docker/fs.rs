use super::ContainerFileInfo;
use crate::connection::get_docker_client;
use crate::error::AppResult;

fn escape_shell_arg(arg: &str) -> String {
    format!("'{}'", arg.replace('\'', "'\\''"))
}

/// 修复 P0-6（zip-slip 防护）：解压前校验 tar entry 的相对路径是否会逃出根目录。
/// container_path 含 `../` 或绝对路径时，恶意 tar 可在用户主机任意位置写文件。
///
/// `root` 是预期的解压根目录（已 canonicalize），任何 entry 解析后必须仍在 root 内。
fn ensure_safe_tar_entry(
    entry_path: &std::path::Path,
    root: &std::path::Path,
) -> Result<(), String> {
    use std::path::Component;
    if entry_path.is_absolute() {
        return Err(format!(
            "tar 包含绝对路径，拒绝解压（zip-slip 防护）: {}",
            entry_path.display()
        ));
    }
    // 逐段累积，禁止任何 `..` 跳出 root；普通组件保留
    let mut accumulated = root.to_path_buf();
    for comp in entry_path.components() {
        match comp {
            Component::ParentDir => {
                return Err(format!(
                    "tar 含 .. 路径分量，拒绝解压（zip-slip 防护）: {}",
                    entry_path.display()
                ));
            }
            Component::Normal(seg) => {
                accumulated.push(seg);
            }
            // CurDir / RootDir / Prefix 视情况忽略或拒绝
            Component::CurDir => {}
            Component::RootDir | Component::Prefix(_) => {
                return Err(format!(
                    "tar 含非相对路径分量，拒绝解压: {}",
                    entry_path.display()
                ));
            }
        }
    }
    // 最终路径必须仍在 root 之下（防符号链接走捷径在 unpack 阶段交给 tar crate 自身处理）
    if !accumulated.starts_with(root) {
        return Err(format!(
            "tar entry 解析后逃出根目录，拒绝解压: {}",
            entry_path.display()
        ));
    }
    Ok(())
}

/// 在容器中同步运行命令并获取 stdout、stderr 与 exit_code 的辅助函数。
///
/// 修复 S1-12：原实现仅返回 (stdout, stderr) 由调用方拼 stderr 字符串判断成功，
/// 当目标命令本身向 stderr 输出 warning 时会被误判为失败。
/// 改为通过 `docker.inspect_exec()` 拿真正的 exit_code，调用方据此判定。
async fn run_exec_to_string(
    container_id: &str,
    cmd: Vec<String>,
) -> AppResult<(String, String, Option<i64>)> {
    use bollard::container::LogOutput;
    use bollard::exec::{CreateExecOptions, StartExecResults};
    use futures_util::StreamExt;

    let docker = get_docker_client().await?;
    let options = CreateExecOptions {
        attach_stdout: Some(true),
        attach_stderr: Some(true),
        cmd: Some(cmd),
        ..Default::default()
    };
    let exec = docker.create_exec(container_id, options).await?;

    let result = docker.start_exec(&exec.id, None).await?;

    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    if let StartExecResults::Attached { mut output, .. } = result {
        while let Some(msg) = output.next().await {
            match msg? {
                LogOutput::StdOut { message } => {
                    stdout.extend_from_slice(&message);
                }
                LogOutput::StdErr { message } => {
                    stderr.extend_from_slice(&message);
                }
                LogOutput::Console { message } => {
                    stdout.extend_from_slice(&message);
                }
                _ => {}
            }
        }
    }

    let exit_code = docker.inspect_exec(&exec.id).await.ok().and_then(|i| i.exit_code);

    Ok((
        String::from_utf8_lossy(&stdout).into_owned(),
        String::from_utf8_lossy(&stderr).into_owned(),
        exit_code,
    ))
}

/// 获取容器内目录的文件列表
#[tauri::command]
pub async fn list_container_files(id: String, path: String) -> AppResult<Vec<ContainerFileInfo>> {
    let target_path = if path.is_empty() {
        "/".to_string()
    } else {
        path
    };

    let escaped_path = escape_shell_arg(&target_path);
    let stat_script = format!(
        "cd {} && for f in * .[^.]*; do ( [ \"$f\" = \".\" ] || [ \"$f\" = \"..\" ] || [ ! -e \"$f\" ] && [ ! -L \"$f\" ] ) && continue; stat -c '%F|%s|%Y|%A|%n' \"$f\" 2>/dev/null; done",
        escaped_path
    );

    let cmd = vec!["sh".to_string(), "-c".to_string(), stat_script];
    let mut files = Vec::new();

    match run_exec_to_string(&id, cmd).await {
        Ok((stdout, _stderr, _exit)) if !stdout.trim().is_empty() => {
            for line in stdout.lines() {
                let parts: Vec<&str> = line.splitn(5, '|').collect();
                if parts.len() < 5 {
                    continue;
                }
                let file_type = parts[0];
                let size_str = parts[1];
                let mtime_str = parts[2];
                let permissions = parts[3].to_string();
                let name = parts[4].to_string();

                let is_dir = file_type.contains("directory");
                let size = size_str.parse::<u64>().unwrap_or(0);
                let mtime = mtime_str.parse::<u64>().unwrap_or(0) * 1000;

                files.push(ContainerFileInfo {
                    name,
                    is_dir,
                    size,
                    mtime,
                    permissions,
                });
            }
        }
        _ => {
            let ls_script = format!("ls -la {}", escaped_path);
            let cmd = vec!["sh".to_string(), "-c".to_string(), ls_script];
            if let Ok((stdout, _stderr, _exit)) = run_exec_to_string(&id, cmd).await {
                for line in stdout.lines() {
                    let trimmed = line.trim();
                    if trimmed.is_empty() || trimmed.starts_with("total") {
                        continue;
                    }

                    let parts: Vec<&str> = trimmed.split_whitespace().collect();
                    if parts.len() < 8 {
                        continue;
                    }

                    let permissions_str = parts[0];
                    let is_dir = permissions_str.starts_with('d');

                    let mut current_idx = 0;
                    for part in parts.iter().take(8) {
                        if let Some(pos) = trimmed[current_idx..].find(*part) {
                            current_idx += pos + part.len();
                        } else {
                            break;
                        }
                    }
                    let name = trimmed[current_idx..].trim().to_string();
                    if name == "." || name == ".." {
                        continue;
                    }

                    let size_idx = parts.len() - 5;
                    let size = parts[size_idx].parse::<u64>().unwrap_or(0);

                    files.push(ContainerFileInfo {
                        name,
                        is_dir,
                        size,
                        mtime: 0,
                        permissions: permissions_str.to_string(),
                    });
                }
            } else {
                return Err("无法读取容器内文件列表，容器可能没有安装 sh 解释器"
                    .to_string()
                    .into());
            }
        }
    }

    files.sort_by(|a, b| {
        if a.is_dir != b.is_dir {
            b.is_dir.cmp(&a.is_dir)
        } else {
            a.name.to_lowercase().cmp(&b.name.to_lowercase())
        }
    });

    Ok(files)
}

/// 下载容器内的文件或目录
#[tauri::command]
pub async fn download_file_from_container(
    id: String,
    container_path: String,
    local_path: String,
) -> AppResult<()> {
    use futures_util::StreamExt;
    use std::path::Path;
    use tar::Archive;
    use tokio::io::AsyncWriteExt;

    log::info!(
        "正在从容器 {} 下载文件: {} -> {}",
        id,
        container_path,
        local_path
    );

    let result = async {
        let docker = get_docker_client().await?;

        let options = bollard::container::DownloadFromContainerOptions {
            path: container_path.clone(),
        };

        let mut stream = docker.download_from_container(&id, Some(options));

        let temp_dir = std::env::temp_dir();
        let temp_file_name = format!("vessel_dl_{}.tar", uuid::Uuid::new_v4());
        let temp_file_path = temp_dir.join(&temp_file_name);

        {
            let mut temp_file = tokio::fs::File::create(&temp_file_path).await?;

            while let Some(chunk_result) = stream.next().await {
                let chunk = chunk_result?;
                temp_file.write_all(&chunk).await?;
            }
            temp_file.flush().await?;
        }

        let local_path_buf = Path::new(&local_path);
        let is_dir =
            local_path_buf.is_dir() || local_path.ends_with('/') || local_path.ends_with('\\');

        // 同步 IO (File::open + tar 解包) 移交到 blocking 线程池，
        // 避免阻塞 tokio worker 拖慢 stats/logs 心跳流
        // 修复 P0-6：解包前逐个校验 entry path，拒绝包含 `..` / 绝对路径的恶意 tar，
        // 防止 zip-slip 攻击向用户主机任意位置写文件
        let local_path_for_blocking = local_path.clone();
        let temp_file_path_for_blocking = temp_file_path.clone();
        tokio::task::spawn_blocking(move || -> Result<(), String> {
            let local_path_buf = Path::new(&local_path_for_blocking);
            // 计算解压根目录
            let unpack_root: std::path::PathBuf = if is_dir {
                local_path_buf.to_path_buf()
            } else {
                local_path_buf
                    .parent()
                    .unwrap_or_else(|| Path::new("."))
                    .to_path_buf()
            };
            // 解压前先确保根目录存在并 canonicalize
            std::fs::create_dir_all(&unpack_root)
                .map_err(|e| format!("准备解压目录失败: {}", e))?;
            let canonical_root = std::fs::canonicalize(&unpack_root)
                .map_err(|e| format!("规整解压根目录失败: {}", e))?;

            // 第一遍：仅校验所有 entry 的相对路径，无 IO 副作用
            {
                let tar_file = std::fs::File::open(&temp_file_path_for_blocking)
                    .map_err(|e| e.to_string())?;
                let mut archive = tar::Archive::new(tar_file);
                for entry in archive.entries().map_err(|e| e.to_string())? {
                    let entry = entry.map_err(|e| e.to_string())?;
                    let path = entry.path().map_err(|e| e.to_string())?;
                    ensure_safe_tar_entry(&path, &canonical_root)?;
                }
            }

            // 第二遍：实际解压（archive.entries() 是消费式迭代器，必须重开 File）
            let tar_file = std::fs::File::open(&temp_file_path_for_blocking)
                .map_err(|e| e.to_string())?;
            let mut archive = Archive::new(tar_file);
            archive
                .unpack(&canonical_root)
                .map_err(|e| e.to_string())?;
            Ok(())
        })
        .await
        .map_err(|e| format!("解压任务失败: {}", e))??;

        if !is_dir {
            let parent = local_path_buf.parent().unwrap_or_else(|| Path::new("."));
            let container_file_name = Path::new(&container_path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");

            if !container_file_name.is_empty() {
                let extracted_path = parent.join(container_file_name);
                if extracted_path.exists() && extracted_path != local_path_buf {
                    if local_path_buf.exists() {
                        let _ = tokio::fs::remove_file(local_path_buf).await;
                    }
                    tokio::fs::rename(&extracted_path, local_path_buf).await?;
                }
            }
        }

        let _ = tokio::fs::remove_file(&temp_file_path).await;

        Ok(())
    }
    .await;

    match &result {
        Ok(_) => log::info!(
            "下载成功: 容器 {} 的 {} -> {}",
            id,
            container_path,
            local_path
        ),
        Err(e) => log::error!(
            "下载失败 (容器 {}): 从 {} 下载到 {} 出错: {}",
            id,
            container_path,
            local_path,
            e
        ),
    }

    result
}

/// 上传宿主机的文件或目录到容器内
#[tauri::command]
pub async fn upload_file_to_container(
    id: String,
    local_path: String,
    container_dir: String,
) -> AppResult<()> {
    use std::path::Path;
    use tar::Builder;

    log::info!(
        "正在上传文件到容器 {}: {} -> {}",
        id,
        local_path,
        container_dir
    );
    let docker = get_docker_client().await?;

    let local_path_buf = Path::new(&local_path).to_path_buf();
    let local_path_clone = local_path.clone();

    let tar_data = tokio::task::spawn_blocking(move || -> Result<Vec<u8>, String> {
        let mut data = Vec::new();
        let mut builder = Builder::new(&mut data);

        let file_name = local_path_buf
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| format!("无法获取本地文件名: {}", local_path_clone))?;

        if local_path_buf.is_dir() {
            builder
                .append_dir_all(file_name, &local_path_buf)
                .map_err(|e| e.to_string())?;
        } else {
            let mut file = std::fs::File::open(&local_path_buf).map_err(|e| e.to_string())?;
            builder
                .append_file(file_name, &mut file)
                .map_err(|e| e.to_string())?;
        }
        builder.finish().map_err(|e| e.to_string())?;
        drop(builder);
        Ok(data)
    })
    .await
    .map_err(|e| format!("线程池错误: {}", e))??;

    let options = bollard::container::UploadToContainerOptions {
        path: container_dir.clone(),
        no_overwrite_dir_non_dir: "false".to_string(),
    };

    match docker
        .upload_to_container(&id, Some(options), tar_data.into())
        .await
    {
        Ok(_) => {
            log::info!(
                "上传成功: {} -> 容器 {} 的 {}",
                local_path,
                id,
                container_dir
            );
            Ok(())
        }
        Err(e) => {
            log::error!("上传失败 (ID: {}): {}", id, e);
            Err(e.into())
        }
    }
}

/// 删除容器内的文件或文件夹
#[tauri::command]
pub async fn delete_container_file(id: String, path: String) -> AppResult<()> {
    log::info!("正在删除容器 {} 内的文件: {}", id, path);
    if path.is_empty() || path == "/" {
        log::error!("拒绝删除容器 {} 的根目录", id);
        return Err("安全起见，禁止删除容器根目录".to_string().into());
    }
    let cmd = vec![
        "sh".to_string(),
        "-c".to_string(),
        format!("rm -rf {}", escape_shell_arg(&path)),
    ];
    let (_stdout, stderr, exit_code) = run_exec_to_string(&id, cmd).await?;
    // 修复 S1-12：以 exit_code != 0 判定失败，stderr 仅作日志线索；
    // 部分容器内程序成功时也可能写 stderr（warning），不能用空判作失败
    if matches!(exit_code, Some(code) if code != 0) {
        let err_msg = format!("删除失败 (exit={:?}): {}", exit_code, stderr.trim());
        log::error!("删除容器 {} 文件 {} 失败: {}", id, path, err_msg);
        return Err(err_msg.into());
    }
    log::info!("删除容器 {} 文件成功: {}", id, path);
    Ok(())
}

/// 在容器中新建文件或文件夹
#[tauri::command]
pub async fn create_container_file(id: String, path: String, is_dir: bool) -> AppResult<()> {
    log::info!(
        "正在容器 {} 内创建{}: {}",
        id,
        if is_dir { "目录" } else { "文件" },
        path
    );
    if path.is_empty() {
        log::error!("创建失败：路径不能为空");
        return Err("路径不能为空".to_string().into());
    }
    let cmd = if is_dir {
        vec![
            "sh".to_string(),
            "-c".to_string(),
            format!("mkdir -p {}", escape_shell_arg(&path)),
        ]
    } else {
        vec![
            "sh".to_string(),
            "-c".to_string(),
            format!("touch {}", escape_shell_arg(&path)),
        ]
    };
    let (_stdout, stderr, exit_code) = run_exec_to_string(&id, cmd).await?;
    // 修复 S1-12：以 exit_code 判定，stderr 不再作为成功失败开关
    if matches!(exit_code, Some(code) if code != 0) {
        let err_msg = format!("创建失败 (exit={:?}): {}", exit_code, stderr.trim());
        log::error!("容器 {} 内创建 {} 失败: {}", id, path, err_msg);
        return Err(err_msg.into());
    }
    log::info!("容器 {} 内创建 {} 成功", id, path);
    Ok(())
}

/// 重命名容器内的文件或文件夹
#[tauri::command]
pub async fn rename_container_file(id: String, src: String, dest: String) -> AppResult<()> {
    log::info!("正在容器 {} 内重命名: {} -> {}", id, src, dest);
    if src.is_empty() || dest.is_empty() {
        log::error!("重命名失败：路径不能为空");
        return Err("路径不能为空".to_string().into());
    }
    let cmd = vec![
        "sh".to_string(),
        "-c".to_string(),
        format!("mv {} {}", escape_shell_arg(&src), escape_shell_arg(&dest)),
    ];
    let (_stdout, stderr, exit_code) = run_exec_to_string(&id, cmd).await?;
    // 修复 S1-12：以 exit_code 判定，stderr 不再作为成功失败开关
    if matches!(exit_code, Some(code) if code != 0) {
        let err_msg = format!("重命名失败 (exit={:?}): {}", exit_code, stderr.trim());
        log::error!("容器 {} 内重命名失败 ({} -> {}): {}", id, src, dest, err_msg);
        return Err(err_msg.into());
    }
    log::info!("容器 {} 内重命名成功: {} -> {}", id, src, dest);
    Ok(())
}

/// 读取容器内文本文件内容
#[tauri::command]
pub async fn read_container_text_file(id: String, path: String) -> AppResult<String> {
    use futures_util::StreamExt;
    use std::io::Read;
    use tar::Archive;

    log::info!("正在读取容器 {} 的文本文件: {}", id, path);

    let docker = get_docker_client().await?;

    let options = bollard::container::DownloadFromContainerOptions { path: path.clone() };

    let mut stream = docker.download_from_container(&id, Some(options));

    let mut tar_bytes = Vec::new();
    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result?;
        tar_bytes.extend_from_slice(&chunk);
    }

    if tar_bytes.is_empty() {
        return Err("文件为空或不存在".to_string().into());
    }

    // tar 解压为同步 IO，移交到 blocking 线程池避免阻塞 tokio worker
    let content = tokio::task::spawn_blocking(move || -> Result<String, String> {
        let mut archive = Archive::new(std::io::Cursor::new(tar_bytes));
        if let Some(entry_result) = archive.entries().map_err(|e| e.to_string())?.next() {
            let mut entry = entry_result.map_err(|e| e.to_string())?;
            let mut content_bytes = Vec::new();
            entry.read_to_end(&mut content_bytes).map_err(|e| e.to_string())?;
            Ok(String::from_utf8_lossy(&content_bytes).into_owned())
        } else {
            Err("在归档中未找到任何文件".to_string())
        }
    })
    .await
    .map_err(|e| format!("解压任务失败: {}", e))??;

    log::info!("成功读取容器 {} 的文本文件: {}", id, path);
    Ok(content)
}

/// 写入容器内文本文件内容
#[tauri::command]
pub async fn write_container_text_file(id: String, path: String, content: String) -> AppResult<()> {
    use std::path::Path;
    use tar::Builder;

    log::info!("正在写入内容到容器 {} 的文件: {}", id, path);
    let docker = get_docker_client().await?;

    let path_buf = Path::new(&path);
    let file_name = path_buf
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| {
            log::error!("写入失败：非法的路径 {}", path);
            "非法的文件路径".to_string()
        })?;

    let parent_dir = path_buf.parent().and_then(|p| p.to_str()).unwrap_or("/");

    let mut tar_data = Vec::new();
    {
        let mut builder = Builder::new(&mut tar_data);

        let mut header = tar::Header::new_gnu();
        header.set_size(content.len() as u64);
        header.set_mode(0o644);

        builder.append_data(&mut header, file_name, content.as_bytes())?;

        builder.finish()?;
    }

    let options = bollard::container::UploadToContainerOptions {
        path: parent_dir.to_string(),
        no_overwrite_dir_non_dir: "false".to_string(),
    };

    match docker
        .upload_to_container(&id, Some(options), tar_data.into())
        .await
    {
        Ok(_) => {
            log::info!("成功写入内容到容器 {} 的文件: {}", id, path);
            Ok(())
        }
        Err(e) => {
            log::error!("向容器 {} 内文件 {} 写入内容失败: {}", id, path, e);
            Err(e.into())
        }
    }
}
