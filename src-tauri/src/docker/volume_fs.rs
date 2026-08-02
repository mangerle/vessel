//! 数据卷文件浏览器后端命令
//!
//! 修复 P0-7 / P0-17：原 Volumes.vue 直接 import `@tauri-apps/plugin-shell::Command`
//! 起 `wsl -d ... -- sh -c "cat << 'EOF' > $path\n$content\nEOF"`，存在：
//! 1. 路径与内容未做 shell 转义，可命令注入；
//! 2. SSH 模式直接 mock 假文件列表，欺骗用户；
//! 3. 视图层完全绕过 api → tauri::command 协议层。
//!
//! 本模块把 list / read / write 三条路径下推到后端，按 ConnectionMode
//! 自动分派 Desktop（直接 std::fs）/ WSL（wsl 子进程 + heredoc 安全转义）/ SSH（SshBridge）。
//! 路径全部走 `validate_volume_path` 白名单（不允许 `..` / NUL / 超长）。

use super::models::VolumeFileEntry;
use crate::connection::{ConnectionConfig, ConnectionMode, current_config, ssh};
use crate::error::{AppError, AppResult};
use std::process::Stdio;

#[cfg(windows)]
use crate::docker::CREATE_NO_WINDOW;

const VOLUME_DATA_PREFIX: &str = "/var/lib/docker/volumes";

/// 校验：禁止 `..` / NUL / 超长，确保拼出的真实路径不会跳出 volume 数据根目录
fn validate_volume_path(path_in_volume: &str) -> AppResult<()> {
    if path_in_volume.len() > 4096 {
        return Err(AppError::Custom("路径长度超限（>4096）".to_string()));
    }
    if path_in_volume.contains('\0') {
        return Err(AppError::Custom("路径包含 NUL 字符".to_string()));
    }
    for seg in path_in_volume.split('/') {
        if seg == ".." {
            return Err(AppError::Custom(format!(
                "路径含 .. 段，拒绝执行: {}",
                path_in_volume
            )));
        }
    }
    Ok(())
}

fn validate_volume_name(name: &str) -> AppResult<()> {
    if name.is_empty() {
        return Err(AppError::Custom("卷名称不能为空".to_string()));
    }
    if name.len() > 255 {
        return Err(AppError::Custom("卷名称过长".to_string()));
    }
    // Docker 卷名仅允许字母数字 + _-
    if !name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.')
    {
        return Err(AppError::Custom(format!(
            "卷名称含非法字符（仅允许字母数字 _-.）: {}",
            name
        )));
    }
    Ok(())
}

/// 拼出 volume 数据目录下的绝对路径
fn build_full_path(volume_name: &str, path_in_volume: &str) -> String {
    let trimmed = path_in_volume.trim_start_matches('/');
    if trimmed.is_empty() {
        format!("{}/{}/_data", VOLUME_DATA_PREFIX, volume_name)
    } else {
        format!("{}/{}/_data/{}", VOLUME_DATA_PREFIX, volume_name, trimmed)
    }
}

fn shell_escape_single_quote(s: &str) -> String {
    s.replace('\'', "'\\''")
}

/// 列出卷内目录文件（按当前连接模式分派）
#[tauri::command]
pub async fn list_volume_files(volume: String, path: String) -> AppResult<Vec<VolumeFileEntry>> {
    validate_volume_name(&volume)?;
    validate_volume_path(&path)?;
    let config = current_config().await;
    let full = build_full_path(&volume, &path);
    log::info!(
        "列卷文件: volume={} path={} mode={:?}",
        volume,
        path,
        config.mode
    );
    list_dir_via_config(&config, &full, &path).await
}

/// 读取卷内文本文件
#[tauri::command]
pub async fn read_volume_text_file(volume: String, path: String) -> AppResult<String> {
    validate_volume_name(&volume)?;
    validate_volume_path(&path)?;
    if path.is_empty() || path == "/" {
        return Err(AppError::Custom("文件路径不能为空或根目录".to_string()));
    }
    let config = current_config().await;
    let full = build_full_path(&volume, &path);
    log::info!(
        "读卷文件: volume={} path={} mode={:?}",
        volume,
        path,
        config.mode
    );
    read_file_via_config(&config, &full).await
}

/// 写入卷内文本文件
#[tauri::command]
pub async fn write_volume_text_file(
    volume: String,
    path: String,
    content: String,
) -> AppResult<()> {
    validate_volume_name(&volume)?;
    validate_volume_path(&path)?;
    if path.is_empty() || path == "/" {
        return Err(AppError::Custom("文件路径不能为空或根目录".to_string()));
    }
    let config = current_config().await;
    let full = build_full_path(&volume, &path);
    log::info!(
        "写卷文件: volume={} path={} mode={:?}",
        volume,
        path,
        config.mode
    );
    write_file_via_config(&config, &full, &content).await
}

// ============== 分派实现 ==============

async fn list_dir_via_config(
    config: &ConnectionConfig,
    full: &str,
    path_in_volume: &str,
) -> AppResult<Vec<VolumeFileEntry>> {
    match config.mode {
        ConnectionMode::Desktop => list_dir_local(full, path_in_volume),
        ConnectionMode::Wsl => {
            list_dir_wsl(config.wsl_distro.as_deref(), full, path_in_volume).await
        }
        ConnectionMode::Ssh => list_dir_ssh(config, full, path_in_volume).await,
    }
}

fn list_dir_local(full: &str, path_in_volume: &str) -> AppResult<Vec<VolumeFileEntry>> {
    let mut out = Vec::new();
    let entries =
        std::fs::read_dir(full).map_err(|e| AppError::Custom(format!("读取目录失败: {}", e)))?;
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
        let child_path = if path_in_volume.is_empty() || path_in_volume == "/" {
            format!("/{}", name)
        } else {
            format!("{}/{}", path_in_volume.trim_end_matches('/'), name)
        };
        out.push(VolumeFileEntry {
            name,
            is_dir,
            path: child_path,
        });
    }
    Ok(out)
}

async fn list_dir_wsl(
    distro: Option<&str>,
    full: &str,
    path_in_volume: &str,
) -> AppResult<Vec<VolumeFileEntry>> {
    let stdout = run_wsl_cmd(distro, "ls", &["-p", full]).await?;
    Ok(parse_ls_p(&stdout, path_in_volume))
}

async fn list_dir_ssh(
    config: &ConnectionConfig,
    full: &str,
    path_in_volume: &str,
) -> AppResult<Vec<VolumeFileEntry>> {
    let bridge = build_ssh_bridge(config)?;
    let escaped = shell_escape_single_quote(full);
    let cmd = format!("ls -p '{}'", escaped);
    let stdout = bridge
        .exec_command(&cmd)
        .await
        .map_err(AppError::SshBridge)?;
    Ok(parse_ls_p(&stdout, path_in_volume))
}

/// 解析 `ls -p` 输出：以 `/` 结尾的是目录
fn parse_ls_p(stdout: &str, parent: &str) -> Vec<VolumeFileEntry> {
    stdout
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .map(|line| {
            let is_dir = line.ends_with('/');
            let name = line.trim_end_matches('/').to_string();
            let child_path = if parent.is_empty() || parent == "/" {
                format!("/{}", name)
            } else {
                format!("{}/{}", parent.trim_end_matches('/'), name)
            };
            VolumeFileEntry {
                name,
                is_dir,
                path: child_path,
            }
        })
        .collect()
}

async fn read_file_via_config(config: &ConnectionConfig, full: &str) -> AppResult<String> {
    match config.mode {
        ConnectionMode::Desktop => tokio::fs::read_to_string(full).await.map_err(Into::into),
        ConnectionMode::Wsl => run_wsl_cmd(config.wsl_distro.as_deref(), "cat", &[full]).await,
        ConnectionMode::Ssh => {
            let bridge = build_ssh_bridge(config)?;
            let escaped = shell_escape_single_quote(full);
            bridge
                .exec_command(&format!("cat '{}'", escaped))
                .await
                .map_err(AppError::SshBridge)
        }
    }
}

async fn write_file_via_config(
    config: &ConnectionConfig,
    full: &str,
    content: &str,
) -> AppResult<()> {
    match config.mode {
        ConnectionMode::Desktop => tokio::fs::write(full, content).await.map_err(Into::into),
        ConnectionMode::Wsl => write_file_wsl(config.wsl_distro.as_deref(), full, content).await,
        ConnectionMode::Ssh => write_file_ssh(config, full, content).await,
    }
}

/// 修复 P0-7：避免 here-doc 被 fileContent 中的 `EOF` 标记意外终止，
/// 改为通过 child stdin 注入 content；远端 sh 仅看到一个被单引号转义的目标路径。
async fn write_file_wsl(distro: Option<&str>, full: &str, content: &str) -> AppResult<()> {
    use tokio::io::AsyncWriteExt;
    let escaped = shell_escape_single_quote(full);
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
        &format!("cat > '{}'", escaped),
    ]);
    cmd.stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);
    let mut child = cmd.spawn()?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(content.as_bytes()).await?;
        drop(stdin);
    }
    let out = child.wait_with_output().await?;
    if out.status.success() {
        Ok(())
    } else {
        Err(AppError::Wsl(
            String::from_utf8_lossy(&out.stderr).to_string(),
        ))
    }
}

async fn write_file_ssh(config: &ConnectionConfig, full: &str, content: &str) -> AppResult<()> {
    use base64::Engine;
    let bridge = build_ssh_bridge(config)?;
    let encoded = base64::engine::general_purpose::STANDARD.encode(content.as_bytes());
    let escaped = shell_escape_single_quote(full);
    let cmd = format!("echo '{}' | base64 -d > '{}'", encoded, escaped);
    bridge
        .exec_command(&cmd)
        .await
        .map_err(AppError::SshBridge)?;
    Ok(())
}

async fn run_wsl_cmd(distro: Option<&str>, bin: &str, args: &[&str]) -> AppResult<String> {
    let mut cmd = tokio::process::Command::new("wsl");
    if let Some(d) = distro
        && !d.is_empty()
    {
        cmd.args(["-d", d]);
    }
    cmd.args(["-u", "root", "--", bin]);
    cmd.args(args);
    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);
    let out = cmd.output().await?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).to_string())
    } else {
        Err(AppError::Wsl(
            String::from_utf8_lossy(&out.stderr).to_string(),
        ))
    }
}

fn build_ssh_bridge(config: &ConnectionConfig) -> AppResult<ssh::SshBridge> {
    let ssh_cfg = ssh::SshConfig {
        host: config
            .ssh_host
            .clone()
            .ok_or_else(|| AppError::ConfigMissing("SSH 主机未配置".to_string()))?,
        port: config.ssh_port.unwrap_or(22),
        user: config
            .ssh_user
            .clone()
            .ok_or_else(|| AppError::ConfigMissing("SSH 用户未配置".to_string()))?,
        password: config.ssh_password.clone(),
        use_sudo: config.use_sudo,
    };
    ssh_cfg.validate()?;
    Ok(ssh::SshBridge::new(ssh_cfg))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_volume_name_basic() {
        assert!(validate_volume_name("my_data").is_ok());
        assert!(validate_volume_name("my-data.1").is_ok());
        assert!(validate_volume_name("").is_err());
        assert!(validate_volume_name("evil;rm").is_err());
        assert!(validate_volume_name("with space").is_err());
    }

    #[test]
    fn validate_volume_path_rejects_dot_dot() {
        assert!(validate_volume_path("/etc/../escape").is_err());
        assert!(validate_volume_path("/normal/path.json").is_ok());
        assert!(validate_volume_path("").is_ok(), "空路径表示根目录");
    }

    #[test]
    fn build_full_path_concats_correctly() {
        assert_eq!(
            build_full_path("vol", ""),
            "/var/lib/docker/volumes/vol/_data"
        );
        assert_eq!(
            build_full_path("vol", "/sub"),
            "/var/lib/docker/volumes/vol/_data/sub"
        );
        assert_eq!(
            build_full_path("vol", "/a/b/c.json"),
            "/var/lib/docker/volumes/vol/_data/a/b/c.json"
        );
    }

    #[test]
    fn parse_ls_p_distinguishes_dir_and_file() {
        let stdout = "logs/\nconfig.json\nbin/\n";
        let entries = parse_ls_p(stdout, "/");
        assert_eq!(entries.len(), 3);
        assert!(entries[0].is_dir && entries[0].name == "logs");
        assert!(!entries[1].is_dir && entries[1].name == "config.json");
        assert_eq!(entries[1].path, "/config.json");
    }

    #[test]
    fn parse_ls_p_handles_nested_path() {
        let entries = parse_ls_p("a.txt\nb/\n", "/sub");
        assert_eq!(entries[0].path, "/sub/a.txt");
        assert_eq!(entries[1].path, "/sub/b");
    }
}
