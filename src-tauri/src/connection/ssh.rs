use bollard::{API_DEFAULT_VERSION, Docker};
use russh::client::{self, Config, Handle, Handler};
use russh::keys::PublicKey;
use russh::keys::ssh_key::HashAlg;
use russh::{Channel, ChannelMsg};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::LazyLock;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Mutex, oneshot};

use crate::error::{AppError, AppResult};

const SSH_CONNECT_TIMEOUT_SECS: u64 = 15;
const SSH_USER_MAX_LEN: usize = 64;
const SSH_HOST_MAX_LEN: usize = 253;

/// russh 0.54 的 Channel 默认消息类型别名
type SshChannel = Channel<russh::client::Msg>;

/// SSH 桥接配置
#[derive(Debug, Clone)]
pub struct SshConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: Option<String>,
    /// 是否使用 sudo 提升权限调用 docker
    /// 适用于远端用户不在 docker 组、但具备 NOPASSWD sudo 权限的场景
    pub use_sudo: bool,
}

impl SshConfig {
    pub fn validate(&self) -> AppResult<()> {
        if self.host.trim().is_empty() {
            return Err(AppError::SshBridge("SSH 主机地址不能为空".to_string()));
        }
        if self.user.trim().is_empty() {
            return Err(AppError::SshBridge("SSH 用户名不能为空".to_string()));
        }
        if self.port == 0 {
            return Err(AppError::SshBridge("SSH 端口非法".to_string()));
        }
        // 修复 S1-13：白名单限制 host/user 字符集与长度，
        // 防止畸形 host/user 拼接出危险的 ssh 命令行
        if self.user.len() > SSH_USER_MAX_LEN
            || !self
                .user
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.')
        {
            return Err(AppError::SshBridge(format!(
                "SSH 用户名仅允许字母/数字/_-. 且长度 ≤ {}",
                SSH_USER_MAX_LEN
            )));
        }
        if self.host.len() > SSH_HOST_MAX_LEN
            || self.host.chars().any(|c| {
                c.is_whitespace()
                    || matches!(
                        c,
                        '"' | '\'' | '`' | '$' | '|' | '&' | ';' | '<' | '>' | '\\' | ' '
                    )
            })
        {
            return Err(AppError::SshBridge(format!(
                "SSH 主机地址含非法字符或长度超过 {}",
                SSH_HOST_MAX_LEN
            )));
        }
        Ok(())
    }
}

/// 修复 P0-1：替换原 TrustAllHostKeyHandler（永远 Ok(true)）。
///
/// 维护 app_data_dir/known_hosts.json 持久化文件，键 = "host:port"，
/// 值 = SHA256 指纹（hex）。
/// - 首次连接：保存指纹（TOFU，Trust On First Use）；
/// - 后续连接：指纹一致放行；不一致直接拒绝（中间人保护）。
///
/// 不与 OpenSSH 标准 known_hosts 兼容（行式格式 + 多类型），仅用于本应用自身。
pub struct KnownHostsHandler {
    host: String,
    port: u16,
    known_hosts_path: PathBuf,
}

impl KnownHostsHandler {
    pub fn new(host: String, port: u16) -> Self {
        Self {
            host,
            port,
            known_hosts_path: known_hosts_file(),
        }
    }
}

impl Handler for KnownHostsHandler {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        server_public_key: &PublicKey,
    ) -> Result<bool, Self::Error> {
        let key_id = format!("{}:{}", self.host, self.port);
        let fingerprint = server_public_key.fingerprint(HashAlg::Sha256).to_string();

        let mut known = load_known_hosts(&self.known_hosts_path).unwrap_or_default();
        match known.get(&key_id) {
            Some(saved) if saved == &fingerprint => {
                log::debug!("known_hosts 命中: {} → {}", key_id, fingerprint);
                Ok(true)
            }
            Some(saved) => {
                log::error!(
                    "SSH 主机指纹不匹配 (中间人风险): {} 已知={} 收到={}",
                    key_id,
                    saved,
                    fingerprint
                );
                // 指纹改变：拒绝；不再静默放行
                Ok(false)
            }
            None => {
                log::warn!(
                    "首次连接 SSH 主机 {}，自动信任并记录指纹: {}",
                    key_id,
                    fingerprint
                );
                known.insert(key_id, fingerprint);
                if let Err(e) = save_known_hosts(&self.known_hosts_path, &known) {
                    log::error!("写入 known_hosts 失败: {}", e);
                }
                Ok(true)
            }
        }
    }
}

/// 计算 known_hosts 文件位置：
/// - 优先 `<app_data_dir>/known_hosts.json`（与 settings.json 同级，跨平台）
/// - 退到 `<temp>/vessel-known-hosts.json` 以防应用未初始化目录时崩溃
fn known_hosts_file() -> PathBuf {
    if let Some(dir) = directories_data_dir() {
        return dir.join("known_hosts.json");
    }
    std::env::temp_dir().join("vessel-known-hosts.json")
}

#[cfg(windows)]
fn directories_data_dir() -> Option<PathBuf> {
    std::env::var_os("APPDATA").map(|p| PathBuf::from(p).join("vessel"))
}

#[cfg(target_os = "macos")]
fn directories_data_dir() -> Option<PathBuf> {
    std::env::var_os("HOME").map(|p| PathBuf::from(p).join("Library/Application Support/vessel"))
}

#[cfg(all(unix, not(target_os = "macos")))]
fn directories_data_dir() -> Option<PathBuf> {
    std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|p| PathBuf::from(p).join(".local/share")))
        .map(|p| p.join("vessel"))
}

fn load_known_hosts(
    path: &PathBuf,
) -> Result<std::collections::HashMap<String, String>, std::io::Error> {
    if !path.exists() {
        return Ok(std::collections::HashMap::new());
    }
    let raw = std::fs::read_to_string(path)?;
    if raw.trim().is_empty() {
        return Ok(std::collections::HashMap::new());
    }
    serde_json::from_str(&raw).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}

fn save_known_hosts(
    path: &PathBuf,
    map: &std::collections::HashMap<String, String>,
) -> Result<(), std::io::Error> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(map)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    std::fs::write(path, json)
}

/// 代理句柄：包含端口与取消信号
struct ProxyHandle {
    port: u16,
    cancel: Option<oneshot::Sender<()>>,
}

static PROXY_HANDLE: LazyLock<Mutex<Option<ProxyHandle>>> = LazyLock::new(|| Mutex::new(None));

/// 关闭并清理当前 TCP 代理
pub async fn reset_proxy() {
    let mut guard = PROXY_HANDLE.lock().await;
    if let Some(handle) = guard.take()
        && let Some(tx) = handle.cancel
    {
        let _ = tx.send(());
    }
}

/// SSH 桥接驱动
pub struct SshBridge {
    pub config: SshConfig,
}

impl SshBridge {
    pub fn new(config: SshConfig) -> Self {
        Self { config }
    }

    /// 通过本地 TCP 代理与远端 Docker daemon 建立连接，返回 bollard Docker 客户端
    pub async fn connect(&self) -> Result<Docker, String> {
        self.config.validate()?;

        let port = self.ensure_proxy().await?;
        let url = format!("http://127.0.0.1:{}", port);
        let docker = Docker::connect_with_http(&url, 120, API_DEFAULT_VERSION)
            .map_err(|e| format!("创建 Docker 客户端失败: {}", e))?;

        docker.ping().await.map_err(|e| {
            format!(
                "SSH 远程 Docker 未响应 (请确认远端 docker 服务正常、用户具备 docker 权限): {}",
                e
            )
        })?;

        Ok(docker)
    }

    /// 在远端执行一条 shell 命令并返回 stdout（用于 compose 文件读写）
    pub async fn exec_command(&self, cmd: &str) -> Result<String, String> {
        let session = self.create_session().await?;
        let mut channel: SshChannel = session
            .channel_open_session()
            .await
            .map_err(|e| format!("打开 SSH 通道失败: {}", e))?;

        channel
            .exec(true, cmd)
            .await
            .map_err(|e| format!("执行远程命令失败: {}", e))?;

        let mut output = Vec::new();
        let mut exit_status: Option<u32> = None;

        loop {
            match channel.wait().await {
                Some(ChannelMsg::Data { data }) => output.extend_from_slice(&data),
                Some(ChannelMsg::ExtendedData { data, .. }) => {
                    log::warn!("远端命令 stderr 输出: {}", String::from_utf8_lossy(&data));
                }
                Some(ChannelMsg::ExitStatus { exit_status: s }) => exit_status = Some(s),
                Some(ChannelMsg::Eof) | Some(ChannelMsg::Close) | None => break,
                Some(_) => continue,
            }
        }

        let _ = channel.eof().await;
        let _ = channel.close().await;

        if let Some(s) = exit_status
            && s != 0
        {
            log::warn!("远端命令退出码非零: {} (命令: {})", s, cmd);
        }

        Ok(String::from_utf8_lossy(&output).to_string())
    }

    /// 开启 TCP 代理：监听本地随机端口，每个 TCP 连接都建立独立的 russh 会话与 channel
    ///
    /// 原子化：检查旧代理 → 清理旧代理 → 绑定端口 → spawn listener → 写回 handle
    /// 全部在 PROXY_HANDLE 同一把写锁内完成，避免并发切换连接时
    /// get → reset → bind → spawn 之间的窗口期产生多个 TCP 监听导致端口泄漏。
    async fn ensure_proxy(&self) -> Result<u16, String> {
        let mut guard = PROXY_HANDLE.lock().await;

        // 1. 已有代理则直接复用
        if let Some(h) = guard.as_ref() {
            return Ok(h.port);
        }

        // 2. 关闭可能残留的旧代理（同锁内）
        if let Some(handle) = guard.take()
            && let Some(tx) = handle.cancel
        {
            let _ = tx.send(());
        }

        // 3. 绑定到随机可用端口
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .map_err(|e| format!("无法绑定代理端口: {}", e))?;
        let port = listener.local_addr().map_err(|e| e.to_string())?.port();

        let (cancel_tx, cancel_rx) = oneshot::channel::<()>();
        let ssh_config = self.config.clone();

        log::info!(
            "SSH TCP 代理即将启动 (远端: {}@{}:{})",
            ssh_config.user,
            ssh_config.host,
            ssh_config.port
        );

        // 4. 在后台运行代理逻辑
        tokio::spawn(async move {
            let mut cancel_rx = cancel_rx;
            loop {
                tokio::select! {
                    _ = &mut cancel_rx => {
                        log::info!("SSH 代理收到关闭信号，停止监听");
                        break;
                    }
                    accept = listener.accept() => {
                        match accept {
                            Ok((stream, _)) => {
                                let cfg = ssh_config.clone();
                                tokio::spawn(async move {
                                    if let Err(e) = handle_proxy_connection(stream, cfg).await {
                                        log::warn!("SSH 代理连接异常退出: {}", e);
                                    }
                                });
                            }
                            Err(e) => {
                                log::error!("SSH 代理 accept 错误: {}", e);
                                break;
                            }
                        }
                    }
                }
            }
        });

        // 5. 同一锁内写回新 handle（其他并发 ensure_proxy 不会重复创建）
        *guard = Some(ProxyHandle {
            port,
            cancel: Some(cancel_tx),
        });

        log::info!("SSH TCP 代理已启动于 127.0.0.1:{}", port);

        Ok(port)
    }

    /// 建立一个新的 russh 会话
    async fn create_session(&self) -> Result<Handle<KnownHostsHandler>, String> {
        let ssh_config = Arc::new(Config::default());
        let addr = (self.config.host.as_str(), self.config.port);

        let handler = KnownHostsHandler::new(self.config.host.clone(), self.config.port);
        let mut session = tokio::time::timeout(
            Duration::from_secs(SSH_CONNECT_TIMEOUT_SECS),
            client::connect(ssh_config, addr, handler),
        )
        .await
        .map_err(|_| {
            format!(
                "连接 SSH 服务器 {}:{} 超时",
                self.config.host, self.config.port
            )
        })?
        .map_err(|e| format!("SSH 连接失败 (可能是主机指纹变化触发拒绝): {}", e))?;

        let auth_result = match &self.config.password {
            Some(pw) => session
                .authenticate_password(&self.config.user, pw)
                .await
                .map_err(|e| format!("密码鉴权失败: {}", e))?,
            None => {
                return Err("未配置密码且尚未实现密钥鉴权，请先在连接中配置密码".to_string());
            }
        };

        if !auth_result.success() {
            return Err(format!(
                "SSH 鉴权失败：用户名 {} 或密码错误",
                self.config.user
            ));
        }

        Ok(session)
    }

    /// 诊断 SSH 远端 Docker 环境：返回每一步是否通过与可读的错误信息，
    /// 方便用户在 UI 上自助排查。
    pub async fn diagnose(&self) -> SshDiagnostic {
        let mut diag = SshDiagnostic::default();

        // 1. SSH 连通性 + 基本环境探测
        let session = match self.create_session().await {
            Ok(s) => {
                diag.ssh_ok = true;
                s
            }
            Err(e) => {
                diag.ssh_error = Some(e);
                diag.recommendation =
                    "无法连接 SSH 服务器，请检查主机地址、端口、网络与凭据".to_string();
                return diag;
            }
        };

        // 2. 当前用户与所属组
        match self.run_remote_cmd(&session, "id -un").await {
            Ok(out) => {
                diag.current_user = out.trim().to_string();
            }
            Err(e) => {
                diag.remote_error = Some(format!("id 命令失败: {}", e));
            }
        }

        match self.run_remote_cmd(&session, "id -Gn").await {
            Ok(out) => {
                let groups: Vec<String> = out.split_whitespace().map(|s| s.to_string()).collect();
                diag.groups = groups.clone();
                diag.user_in_docker_group = groups.iter().any(|g| g == "docker");
            }
            Err(e) => {
                diag.remote_error = Some(format!("id -Gn 命令失败: {}", e));
            }
        }

        // 3. docker socket 信息
        match self.run_remote_cmd(&session, "ls -la /var/run/docker.sock 2>/dev/null || ls -la /run/docker.sock 2>/dev/null || echo 'NOT_FOUND'").await {
            Ok(out) => {
                let trimmed = out.trim();
                if trimmed == "NOT_FOUND" {
                    diag.docker_socket_path = "未找到".to_string();
                    diag.docker_socket_perms = "".to_string();
                } else {
                    // 拆分路径与权限字符串
                    // 输出形如: "/var/run/docker.sock -> ../../srw-rw---- 1 root docker ..."
                    if let Some((path, rest)) = trimmed.split_once("->") {
                        diag.docker_socket_path = path.trim().to_string();
                        let rest = rest.trim();
                        // 权限段是第 2 列（空格分隔后第 2 段）
                        let parts: Vec<&str> = rest.split_whitespace().collect();
                        if parts.len() >= 2 {
                            diag.docker_socket_perms = parts[1].to_string();
                            let group = parts.get(3).copied().unwrap_or("");
                            diag.docker_socket_group = group.to_string();
                        }
                    }
                }
            }
            Err(e) => {
                diag.remote_error = Some(format!("查看 docker socket 失败: {}", e));
            }
        }

        // 4. 不带 sudo 跑 docker ps
        match self
            .run_remote_cmd(&session, "docker ps --format '{{.ID}}' 2>&1")
            .await
        {
            Ok(out) => {
                let trimmed = out.trim();
                if trimmed.is_empty() {
                    diag.docker_works_without_sudo = true;
                } else {
                    diag.docker_error_without_sudo = Some(trimmed.to_string());
                }
            }
            Err(e) => {
                diag.docker_error_without_sudo = Some(e);
            }
        }

        // 5. 带 sudo 跑 docker ps
        match self
            .run_remote_cmd(&session, "sudo -n docker ps --format '{{.ID}}' 2>&1")
            .await
        {
            Ok(out) => {
                let trimmed = out.trim();
                if trimmed.is_empty() {
                    diag.docker_works_with_sudo = true;
                } else {
                    diag.docker_error_with_sudo = Some(trimmed.to_string());
                }
            }
            Err(e) => {
                diag.docker_error_with_sudo = Some(e);
            }
        }

        // 6. 给出建议
        diag.recommendation = build_recommendation(&diag);
        diag
    }

    /// 在已建立的 SSH 会话上执行一条命令并返回 stdout
    async fn run_remote_cmd(
        &self,
        session: &Handle<KnownHostsHandler>,
        cmd: &str,
    ) -> Result<String, String> {
        let mut channel: SshChannel = session
            .channel_open_session()
            .await
            .map_err(|e| format!("打开 SSH 通道失败: {}", e))?;
        channel
            .exec(true, cmd)
            .await
            .map_err(|e| format!("执行远程命令失败: {}", e))?;
        let mut output = Vec::new();
        loop {
            match channel.wait().await {
                Some(ChannelMsg::Data { data }) => output.extend_from_slice(&data),
                Some(ChannelMsg::ExtendedData { data, .. }) => {
                    output.extend_from_slice(&data);
                }
                Some(ChannelMsg::Eof)
                | Some(ChannelMsg::Close)
                | Some(ChannelMsg::ExitStatus { .. })
                | None => break,
                Some(_) => continue,
            }
        }
        let _ = channel.eof().await;
        let _ = channel.close().await;
        Ok(String::from_utf8_lossy(&output).to_string())
    }
}

/// SSH 远端 Docker 环境诊断结果
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct SshDiagnostic {
    /// SSH 凭据与网络是否可达
    pub ssh_ok: bool,
    pub ssh_error: Option<String>,

    /// 当前 SSH 登录的用户
    pub current_user: String,
    /// 用户所属的附加组列表
    pub groups: Vec<String>,
    /// 是否在 docker 组
    pub user_in_docker_group: bool,

    /// docker socket 路径
    pub docker_socket_path: String,
    /// docker socket 权限串
    pub docker_socket_perms: String,
    /// docker socket 属组
    pub docker_socket_group: String,

    /// `docker ps` 不带 sudo 是否成功
    pub docker_works_without_sudo: bool,
    pub docker_error_without_sudo: Option<String>,

    /// `sudo -n docker ps` 是否成功
    pub docker_works_with_sudo: bool,
    pub docker_error_with_sudo: Option<String>,

    /// 远端环境其他错误
    pub remote_error: Option<String>,

    /// 自动给出的修复建议
    pub recommendation: String,
}

fn build_recommendation(d: &SshDiagnostic) -> String {
    if !d.ssh_ok {
        return "请先确认 SSH 主机、端口、用户名密码正确无误".to_string();
    }
    if d.docker_socket_path == "未找到" {
        return "远端未检测到 docker socket，请先安装并启动 Docker 守护进程".to_string();
    }
    if d.docker_works_without_sudo {
        return "当前用户已具备 docker 权限，可直接连接，无需 sudo".to_string();
    }
    if d.docker_works_with_sudo {
        return "需在连接配置中勾选「使用 sudo 提升权限」，并确认远端已配置 NOPASSWD sudo"
            .to_string();
    }
    if !d.user_in_docker_group {
        return "当前用户既不在 docker 组，sudo 也不可用。请将用户加入 docker 组（需要重新登录生效）：sudo usermod -aG docker <user>".to_string();
    }
    "请检查远端 Docker 服务是否正常运行 (systemctl status docker)，或检查 SELinux/AppArmor 是否拦截"
        .to_string()
}

/// 处理一个 TCP 代理连接：建立 russh 会话 + 透传到 channel
async fn handle_proxy_connection(mut tcp: TcpStream, config: SshConfig) -> Result<(), String> {
    let bridge = SshBridge::new(config.clone());
    let session = bridge.create_session().await?;

    let channel: SshChannel = session
        .channel_open_session()
        .await
        .map_err(|e| format!("打开 SSH 通道失败: {}", e))?;

    channel
        .exec(true, dial_stdio_command(&config).as_str())
        .await
        .map_err(|e| format!("执行 docker dial-stdio 失败: {}", e))?;

    // 将 channel 分离为独立的读写两端
    let (mut channel_read, channel_write) = channel.split();
    let mut ssh_reader = channel_read.make_reader();
    let mut ssh_writer = channel_write.make_writer();
    let (mut tcp_read, mut tcp_write) = tcp.split();

    let copy_to_ssh = async {
        let mut buf = vec![0u8; 8192];
        loop {
            match tcp_read.read(&mut buf).await {
                Ok(0) => break,
                Ok(n) => {
                    if ssh_writer.write_all(&buf[..n]).await.is_err() {
                        break;
                    }
                    if ssh_writer.flush().await.is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    };

    let copy_from_ssh = async {
        let mut buf = vec![0u8; 8192];
        loop {
            match ssh_reader.read(&mut buf).await {
                Ok(0) => break,
                Ok(n) => {
                    if tcp_write.write_all(&buf[..n]).await.is_err() {
                        break;
                    }
                    if tcp_write.flush().await.is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    };

    tokio::select! {
        _ = copy_to_ssh => {},
        _ = copy_from_ssh => {},
    }
    Ok(())
}

/// 构造 dial-stdio 命令，必要时加 sudo 前缀
fn dial_stdio_command(config: &SshConfig) -> String {
    if config.use_sudo {
        "sudo -n docker system dial-stdio".to_string()
    } else {
        "docker system dial-stdio".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn good() -> SshConfig {
        SshConfig {
            host: "192.168.1.105".into(),
            port: 22,
            user: "root".into(),
            password: Some("pw".into()),
            use_sudo: false,
        }
    }

    #[test]
    fn validate_accepts_normal_host_user() {
        assert!(good().validate().is_ok());
    }

    #[test]
    fn validate_rejects_empty_host_or_user() {
        let mut c = good();
        c.host = "".into();
        assert!(c.validate().is_err(), "空 host 应拒绝");

        let mut c = good();
        c.user = "".into();
        assert!(c.validate().is_err(), "空 user 应拒绝");

        let mut c = good();
        c.port = 0;
        assert!(c.validate().is_err(), "零端口应拒绝");
    }

    #[test]
    fn validate_rejects_shell_metacharacters_in_host() {
        for bad_host in [
            "host;rm -rf /",
            "host`whoami`",
            "host$x",
            "host\"y",
            "host'z",
            "host with space",
            "host\\sub",
            "host|pipe",
            "host&bg",
            "host>redir",
            "host<in",
        ] {
            let mut c = good();
            c.host = bad_host.into();
            assert!(c.validate().is_err(), "非法 host 应拒绝: {}", bad_host);
        }
    }

    #[test]
    fn validate_rejects_shell_metacharacters_in_user() {
        for bad_user in ["us er", "us;er", "us`r", "us$r", "us|r"] {
            let mut c = good();
            c.user = bad_user.into();
            assert!(c.validate().is_err(), "非法 user 应拒绝: {}", bad_user);
        }
    }

    #[test]
    fn validate_rejects_overlong_host_or_user() {
        let mut c = good();
        c.host = "h".repeat(SSH_HOST_MAX_LEN + 1);
        assert!(c.validate().is_err(), "超长 host 应拒绝");

        let mut c = good();
        c.user = "u".repeat(SSH_USER_MAX_LEN + 1);
        assert!(c.validate().is_err(), "超长 user 应拒绝");
    }

    #[test]
    fn dial_stdio_command_respects_use_sudo_flag() {
        let mut c = good();
        c.use_sudo = false;
        assert_eq!(dial_stdio_command(&c), "docker system dial-stdio");
        c.use_sudo = true;
        assert_eq!(dial_stdio_command(&c), "sudo -n docker system dial-stdio");
    }
}
