use bollard::{API_DEFAULT_VERSION, Docker};
use russh::client::{self, Config, Handle, Handler};
use russh::keys::PublicKey;
use russh::{Channel, ChannelMsg};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{oneshot, Mutex};
use std::sync::LazyLock;

const SSH_CONNECT_TIMEOUT_SECS: u64 = 15;

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
    pub fn validate(&self) -> Result<(), String> {
        if self.host.trim().is_empty() {
            return Err("SSH 主机地址不能为空".to_string());
        }
        if self.user.trim().is_empty() {
            return Err("SSH 用户名不能为空".to_string());
        }
        if self.port == 0 {
            return Err("SSH 端口非法".to_string());
        }
        Ok(())
    }
}

/// 跳过主机密钥校验的 Handler
struct TrustAllHostKeyHandler;

impl Handler for TrustAllHostKeyHandler {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        _server_public_key: &PublicKey,
    ) -> Result<bool, Self::Error> {
        Ok(true)
    }
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
    async fn ensure_proxy(&self) -> Result<u16, String> {
        // 已有代理则直接复用
        {
            let guard = PROXY_HANDLE.lock().await;
            if let Some(h) = guard.as_ref() {
                return Ok(h.port);
            }
        }

        // 关闭可能残留的旧代理
        reset_proxy().await;

        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .map_err(|e| format!("无法绑定代理端口: {}", e))?;
        let port = listener.local_addr().map_err(|e| e.to_string())?.port();

        let (cancel_tx, mut cancel_rx) = oneshot::channel::<()>();
        let ssh_config = self.config.clone();

        log::info!(
            "SSH TCP 代理即将启动 (远端: {}@{}:{})",
            ssh_config.user,
            ssh_config.host,
            ssh_config.port
        );

        // 在后台运行代理逻辑
        tokio::spawn(async move {
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

        *PROXY_HANDLE.lock().await = Some(ProxyHandle {
            port,
            cancel: Some(cancel_tx),
        });

        log::info!("SSH TCP 代理已启动于 127.0.0.1:{}", port);

        Ok(port)
    }

    /// 建立一个新的 russh 会话
    async fn create_session(&self) -> Result<Handle<TrustAllHostKeyHandler>, String> {
        let ssh_config = Arc::new(Config::default());
        let addr = (self.config.host.as_str(), self.config.port);

        let mut session = tokio::time::timeout(
            Duration::from_secs(SSH_CONNECT_TIMEOUT_SECS),
            client::connect(ssh_config, addr, TrustAllHostKeyHandler),
        )
        .await
        .map_err(|_| format!("连接 SSH 服务器 {}:{} 超时", self.config.host, self.config.port))?
        .map_err(|e| format!("SSH 连接失败: {}", e))?;

        let auth_result = match &self.config.password {
            Some(pw) => session
                .authenticate_password(&self.config.user, pw)
                .await
                .map_err(|e| format!("密码鉴权失败: {}", e))?,
            None => {
                return Err(
                    "未配置密码且尚未实现密钥鉴权，请先在连接中配置密码".to_string(),
                );
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
