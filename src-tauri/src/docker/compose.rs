use super::ComposeProject;
use crate::connection::{ConnectionConfig, ConnectionMode, current_config, get_docker_client, ssh};
use crate::error::AppResult;
use crate::handle_docker_op;
use bollard::container::ListContainersOptions;
use std::collections::HashMap;
use std::process::Stdio;
use tauri::AppHandle;
use tauri::Emitter;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

/// 获取 Compose 项目列表
#[tauri::command]
pub async fn list_compose_projects() -> AppResult<Vec<ComposeProject>> {
    let docker = get_docker_client().await?;
    let containers = docker
        .list_containers(Some(ListContainersOptions::<String> {
            all: true,
            ..Default::default()
        }))
        .await?;

    struct ProjectData {
        total: usize,
        running: usize,
        working_dir: Option<String>,
        config_file: Option<String>,
    }

    let mut projects_map: HashMap<String, ProjectData> = HashMap::new();

    for container in containers {
        if let Some(labels) = container.labels
            && let Some(project_name) = labels.get("com.docker.compose.project")
        {
            let data = projects_map
                .entry(project_name.clone())
                .or_insert(ProjectData {
                    total: 0,
                    running: 0,
                    working_dir: labels
                        .get("com.docker.compose.project.working_dir")
                        .cloned(),
                    config_file: labels
                        .get("com.docker.compose.project.config_files")
                        .cloned(),
                });

            data.total += 1;
            if container.state.as_deref() == Some("running") {
                data.running += 1;
            }
        }
    }

    let projects = projects_map
        .into_iter()
        .map(|(name, data)| ComposeProject {
            name,
            container_count: data.total,
            running_count: data.running,
            status: if data.running > 0 {
                "running".to_string()
            } else {
                "exited".to_string()
            },
            working_dir: data.working_dir,
            config_file: data.config_file,
        })
        .collect();

    Ok(projects)
}

/// 读取 Compose 配置文件内容
///
/// 按当前活动连接模式自动分派：WSL 通过 wsl 命令、SSH 通过 SshBridge、
/// Desktop 走本地文件系统。
#[tauri::command]
pub async fn read_compose_file(path: String) -> AppResult<String> {
    let config = current_config().await;
    log::info!("正在读取 Compose 文件: {} (模式: {:?})", path, config.mode);
    read_file_via_config(&config, &path).await
}

async fn read_file_via_config(config: &ConnectionConfig, path: &str) -> AppResult<String> {
    match config.mode {
        ConnectionMode::Wsl => read_file_via_wsl(config.wsl_distro.as_deref(), path).await,
        ConnectionMode::Ssh => read_file_via_ssh(config, path).await,
        ConnectionMode::Desktop => Ok(tokio::fs::read_to_string(path).await?),
    }
}

async fn read_file_via_wsl(distro: Option<&str>, path: &str) -> AppResult<String> {
    let mut cmd = tokio::process::Command::new("wsl");
    if let Some(d) = distro
        && !d.is_empty()
    {
        cmd.args(["-d", d]);
    }
    cmd.args(["-u", "root", "--", "cat", path]);

    #[cfg(windows)]
    cmd.creation_flags(super::CREATE_NO_WINDOW);

    let out = cmd.output().await?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).to_string().into())
    }
}

async fn read_file_via_ssh(config: &ConnectionConfig, path: &str) -> AppResult<String> {
    let ssh_config = build_ssh_config(config)?;
    let bridge = ssh::SshBridge::new(ssh_config);
    // 与 WSL 分支保持一致：通过 shell_escape_single_quote 完整转义后单引号包裹
    let escaped = shell_escape_single_quote(path);
    let cmd = format!("cat '{}'", escaped);
    let out = bridge.exec_command(&cmd).await?;
    Ok(out)
}

/// 写入 Compose 配置文件内容
#[tauri::command]
pub async fn write_compose_file(path: String, content: String) -> AppResult<()> {
    let config = current_config().await;
    log::info!("正在写入 Compose 文件: {} (模式: {:?})", path, config.mode);
    write_file_via_config(&config, &path, &content).await
}

async fn write_file_via_config(
    config: &ConnectionConfig,
    path: &str,
    content: &str,
) -> AppResult<()> {
    match config.mode {
        ConnectionMode::Wsl => {
            write_file_via_wsl(config.wsl_distro.as_deref(), path, content).await
        }
        ConnectionMode::Ssh => write_file_via_ssh(config, path, content).await,
        ConnectionMode::Desktop => {
            handle_docker_op!("Compose 文件写入", path, tokio::fs::write(path, content))
        }
    }
}

async fn write_file_via_wsl(distro: Option<&str>, path: &str, content: &str) -> AppResult<()> {
    // 修复 S1-10：原实现 `sh -c "cat > \"$1\"" -- "$path"` 在 path 含 `$`/双引号/`\` 时
    // 会被 shell 二次解释，存在命令注入风险。改为：
    // 1) 将 content 走 stdin 输入，避免任何 shell 解释
    // 2) 远端 sh -c 中用单引号包裹的转义路径，单引号内除单引号外不会触发任何 shell 解释
    let escaped_path = shell_escape_single_quote(path);

    let mut cmd = tokio::process::Command::new("wsl");
    if let Some(d) = distro
        && !d.is_empty()
    {
        cmd.args(["-d", d]);
    }
    cmd.args([
        "-u",
        "root",
        "--",
        "sh",
        "-c",
        &format!("cat > '{}'", escaped_path),
    ]);
    cmd.stdin(Stdio::piped());

    #[cfg(windows)]
    cmd.creation_flags(super::CREATE_NO_WINDOW);

    let mut child = cmd.spawn()?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(content.as_bytes()).await?;
        drop(stdin);
    }

    let out = child.wait_with_output().await?;
    if out.status.success() {
        log::info!("Compose 文件写入成功 (WSL): {}", path);
        Ok(())
    } else {
        let err = String::from_utf8_lossy(&out.stderr).to_string();
        log::error!("Compose 文件写入失败 (WSL) {}: {}", path, err);
        Err(err.into())
    }
}

/// 把字符串中的单引号替换为 `'\''`，配合外层单引号包裹即可安全嵌入 sh -c。
fn shell_escape_single_quote(s: &str) -> String {
    s.replace('\'', "'\\''")
}

async fn write_file_via_ssh(config: &ConnectionConfig, path: &str, content: &str) -> AppResult<()> {
    let ssh_config = build_ssh_config(config)?;
    let bridge = ssh::SshBridge::new(ssh_config);
    // 使用 base64 编码避免引号/换行/特殊字符的 shell 注入问题
    use base64::Engine;
    let encoded = base64::engine::general_purpose::STANDARD.encode(content.as_bytes());
    let escaped_path = shell_escape_single_quote(path);
    let cmd = format!("echo '{}' | base64 -d > '{}'", encoded, escaped_path);
    bridge.exec_command(&cmd).await?;
    log::info!("Compose 文件写入成功 (SSH): {}", path);
    Ok(())
}

fn build_ssh_config(config: &ConnectionConfig) -> AppResult<ssh::SshConfig> {
    let ssh_cfg =
        ssh::SshConfig {
            host: config.ssh_host.clone().ok_or_else(|| {
                crate::error::AppError::ConfigMissing("SSH 主机未配置".to_string())
            })?,
            port: config.ssh_port.unwrap_or(22),
            user: config.ssh_user.clone().ok_or_else(|| {
                crate::error::AppError::ConfigMissing("SSH 用户未配置".to_string())
            })?,
            password: config.ssh_password.clone(),
            use_sudo: config.use_sudo,
        };
    ssh_cfg.validate()?;
    Ok(ssh_cfg)
}

/// 执行 Compose 命令并实时流式传输输出
#[tauri::command]
pub async fn run_compose_command(
    app: AppHandle,
    project_dir: String,
    args: Vec<String>,
) -> AppResult<()> {
    let config = current_config().await;
    let args_str = args.join(" ");
    log::info!(
        "正在执行 Compose 命令: docker compose {} (目录: {}, 模式: {:?})",
        args_str,
        project_dir,
        config.mode
    );

    let mut cmd = build_compose_command(&config, &project_dir, &args)?;

    cmd.stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());

    #[cfg(windows)]
    {
        cmd.creation_flags(super::CREATE_NO_WINDOW);
    }

    let mut child = cmd.spawn()?;

    let stdout = child
        .stdout
        .take()
        .ok_or("无法获取 compose 进程的 stdout")?;
    let stderr = child
        .stderr
        .take()
        .ok_or("无法获取 compose 进程的 stderr")?;

    let app_clone = app.clone();
    tauri::async_runtime::spawn(async move {
        let reader = tokio::io::BufReader::new(stdout);
        let mut lines = reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            let _ = app_clone.emit("compose-cmd-output", line);
        }
    });

    let app_clone_err = app.clone();
    tauri::async_runtime::spawn(async move {
        let reader = tokio::io::BufReader::new(stderr);
        let mut lines = reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            let _ = app_clone_err.emit("compose-cmd-output", line);
        }
    });

    let app_clone_finish = app.clone();
    tauri::async_runtime::spawn(async move {
        match child.wait().await {
            Ok(status) => {
                if status.success() {
                    let _ = app_clone_finish.emit("compose-cmd-finished", ());
                } else {
                    let _ = app_clone_finish.emit(
                        "compose-cmd-error",
                        format!("Process exited with status: {}", status),
                    );
                }
            }
            Err(e) => {
                let _ = app_clone_finish.emit(
                    "compose-cmd-error",
                    format!("Failed to wait for process: {}", e),
                );
            }
        }
    });

    Ok(())
}

/// 按当前活动连接模式构造 compose 子进程命令
fn build_compose_command(
    config: &ConnectionConfig,
    project_dir: &str,
    args: &[String],
) -> AppResult<tokio::process::Command> {
    match config.mode {
        ConnectionMode::Wsl => {
            let mut c = tokio::process::Command::new("wsl");
            if let Some(d) = config.wsl_distro.as_deref()
                && !d.is_empty()
            {
                c.args(["-d", d]);
            }
            c.args(["--cd", project_dir, "--", "docker", "compose"]);
            c.args(args);
            Ok(c)
        }
        ConnectionMode::Ssh => build_ssh_compose_command(config, project_dir, args),
        ConnectionMode::Desktop => {
            let mut c = tokio::process::Command::new("docker");
            c.arg("compose").args(args).current_dir(project_dir);
            Ok(c)
        }
    }
}

/// 通过 SSH 执行 docker compose 命令：把 `cd <dir> && docker compose <args>` 远端执行
fn build_ssh_compose_command(
    config: &ConnectionConfig,
    project_dir: &str,
    args: &[String],
) -> AppResult<tokio::process::Command> {
    build_ssh_config(config)?; // 校验配置

    let mut c = tokio::process::Command::new("ssh");
    c.args([
        "-o",
        "BatchMode=no",
        "-o",
        "StrictHostKeyChecking=no",
        "-o",
        &format!(
            "UserKnownHostsFile={}",
            if cfg!(windows) { "NUL" } else { "/dev/null" }
        ),
        "-o",
        "LogLevel=ERROR",
        "-p",
        &config.ssh_port.unwrap_or(22).to_string(),
    ])
    .arg(format!(
        "{}@{}",
        config.ssh_user.clone().unwrap_or_default(),
        config.ssh_host.clone().unwrap_or_default()
    ));

    // 拼接远端命令：cd <dir> && [sudo] docker compose <args>
    // 对每个 arg 做单引号转义后用单引号包裹
    let docker_prefix = if config.use_sudo {
        "sudo -n docker"
    } else {
        "docker"
    };
    let mut remote_cmd = format!(
        "cd '{}' && {} compose",
        shell_escape_single_quote(project_dir),
        docker_prefix
    );
    for a in args {
        remote_cmd.push(' ');
        remote_cmd.push('\'');
        remote_cmd.push_str(&shell_escape_single_quote(a));
        remote_cmd.push('\'');
    }

    c.arg(remote_cmd);

    if let Some(pw) = &config.ssh_password {
        // 通过 SSH_ASKPASS 机制注入密码
        let askpass_path =
            write_askpass_script(pw).map_err(|e| format!("写入 askpass 脚本失败: {}", e))?;
        c.env("SSH_ASKPASS", &askpass_path)
            .env("SSH_ASKPASS_REQUIRE", "force");
        #[cfg(not(windows))]
        c.env("DISPLAY", ":0");
    }

    Ok(c)
}

fn write_askpass_script(password: &str) -> Result<std::path::PathBuf, String> {
    let dir = std::env::temp_dir().join("vessel_ssh_askpass");
    std::fs::create_dir_all(&dir).map_err(|e| format!("无法创建 askpass 目录: {}", e))?;

    #[cfg(windows)]
    let (filename, content) = {
        let filename = format!("compose_askpass_{}.bat", std::process::id());
        let content = format!("@echo off\r\necho {}\r\n", password);
        (filename, content)
    };
    #[cfg(not(windows))]
    let (filename, content) = {
        let filename = format!("compose_askpass_{}.sh", std::process::id());
        let escaped = password.replace('\'', "'\\''");
        let content = format!("#!/bin/sh\necho '{}'\n", escaped);
        (filename, content)
    };

    let path = dir.join(&filename);
    std::fs::write(&path, &content).map_err(|e| format!("无法写入 askpass 脚本: {}", e))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700))
            .map_err(|e| format!("无法设置 askpass 脚本权限: {}", e))?;
    }
    #[cfg(windows)]
    {
        // Windows 上使用 icacls 收紧 ACL：仅当前用户可读写，
        // 避免其他低权限进程读到明文密码
        let _ = std::process::Command::new("icacls")
            .args([
                path.to_str().unwrap_or(""),
                "/inheritance:r",                    // 移除继承的 ACL
                "/grant:r",                          // 替换现有权限
                &format!("{}:(R,W)", whoami_safe()), // 仅当前用户读写
            ])
            .output();
    }
    Ok(path)
}

/// 安全获取当前 Windows 用户名（用于 icacls）
#[cfg(windows)]
fn whoami_safe() -> String {
    std::env::var("USERNAME").unwrap_or_else(|_| "%USERNAME%".to_string())
}
