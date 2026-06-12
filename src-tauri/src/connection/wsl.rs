use bollard::{API_DEFAULT_VERSION, Docker};
use std::pin::Pin;
use std::process::Stdio;
use std::sync::Arc;
use std::sync::LazyLock;
use std::sync::atomic::{AtomicU64, Ordering};
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::net::TcpListener;
use tokio::process::Command;
use tokio::sync::Mutex;

#[cfg(windows)]
use crate::docker::CREATE_NO_WINDOW;
const TIMEOUT_CHECK_INTERVAL_SECS: u64 = 5;
const IDLE_TIMEOUT_SECS: u64 = 15;

/// WSL 桥接驱动
#[derive(Default)]
pub struct WslBridge {
    pub distro: Option<String>,
}

/// 代理句柄：包含端口与取消信号，用于真正停止后台 listener 任务
struct ProxyHandle {
    port: u16,
    cancel: Option<tokio::sync::oneshot::Sender<()>>,
}

/// 全局 WSL 代理句柄，初始为空
static PROXY_HANDLE: LazyLock<Mutex<Option<ProxyHandle>>> = LazyLock::new(|| Mutex::new(None));

/// 关闭并清理当前 WSL TCP 代理（停止后台 listener 任务，释放端口）
pub async fn reset_proxy_port() {
    let mut guard = PROXY_HANDLE.lock().await;
    if let Some(handle) = guard.take()
        && let Some(tx) = handle.cancel
    {
        let _ = tx.send(());
    }
    log::info!("WSL 代理已收到关闭信号");
}

impl WslBridge {
    pub fn new(distro: Option<String>) -> Self {
        Self { distro }
    }

    pub async fn connect(&self) -> Result<Docker, String> {
        // 1. 确保代理服务器已启动
        let port = self.ensure_proxy().await?;

        // 2. 通过 TCP 代理连接到 WSL Docker
        let url = format!("http://127.0.0.1:{}", port);
        let docker = Docker::connect_with_http(&url, 120, API_DEFAULT_VERSION)
            .map_err(|e| format!("创建 Docker 客户端失败: {}", e))?;

        // 3. 验证连接
        docker.ping().await.map_err(|e| {
            format!(
                "WSL Docker 未响应 (请确保 WSL 中已安装 Docker 且 wsl 命令可用): {}",
                e
            )
        })?;

        Ok(docker)
    }

    /// 确保 TCP 代理服务器正在运行
    async fn ensure_proxy(&self) -> Result<u16, String> {
        // 已有代理则直接复用
        {
            let guard = PROXY_HANDLE.lock().await;
            if let Some(h) = guard.as_ref() {
                return Ok(h.port);
            }
        }

        // 关闭可能残留的旧代理
        reset_proxy_port().await;

        // 绑定到随机可用端口
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .map_err(|e| format!("无法绑定代理端口: {}", e))?;
        let port = listener.local_addr().map_err(|e| e.to_string())?.port();

        let (cancel_tx, mut cancel_rx) = tokio::sync::oneshot::channel::<()>();
        let distro = self.distro.clone();

        // 在后台运行代理逻辑
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = &mut cancel_rx => {
                        log::info!("WSL 代理收到关闭信号，停止监听");
                        break;
                    }
                    accept = listener.accept() => {
                        match accept {
                            Ok((mut client_socket, _)) => {
                                let distro_clone = distro.clone();
                                tokio::spawn(async move {
                                    handle_proxy_connection(client_socket, distro_clone).await;
                                });
                            }
                            Err(e) => {
                                log::error!("WSL 代理 accept 错误: {}", e);
                                break;
                            }
                        }
                    }
                }
            }
            // listener 在此 drop，端口自动释放
        });

        *PROXY_HANDLE.lock().await = Some(ProxyHandle {
            port,
            cancel: Some(cancel_tx),
        });

        log::info!("WSL TCP 代理已启动于 127.0.0.1:{}", port);
        Ok(port)
    }
}

/// 处理单个 TCP 代理连接：启动一个 wsl 进程并透传字节流
async fn handle_proxy_connection(
    mut client_socket: tokio::net::TcpStream,
    distro: Option<String>,
) {
    let mut cmd = Command::new("wsl");
    if let Some(d) = distro.as_deref()
        && !d.is_empty()
    {
        cmd.args(["-d", d]);
    }
    cmd.args(["docker", "system", "dial-stdio"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());

    #[cfg(windows)]
    {
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            log::error!("无法启动 wsl 子进程: {}", e);
            return;
        }
    };

    let mut stdin = match child.stdin.take() {
        Some(s) => s,
        None => {
            log::error!("无法获取 WSL 进程的 stdin");
            return;
        }
    };
    let mut stdout = match child.stdout.take() {
        Some(s) => s,
        None => {
            log::error!("无法获取 WSL 进程的 stdout");
            return;
        }
    };

    let (client_reader, client_writer) = client_socket.split();

    let last_activity = Arc::new(AtomicU64::new(get_elapsed_seconds()));

    let mut reader = TimeoutIO {
        inner: client_reader,
        last_activity: last_activity.clone(),
    };
    let mut writer = TimeoutIO {
        inner: client_writer,
        last_activity: last_activity.clone(),
    };

    let last_activity_clone = last_activity.clone();
    let timeout_task = async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(TIMEOUT_CHECK_INTERVAL_SECS)).await;
            let now = get_elapsed_seconds();
            let last = last_activity_clone.load(Ordering::Relaxed);
            if now - last > IDLE_TIMEOUT_SECS {
                // 超过 15 秒无任何数据收发，则判定为空闲，主动断开连接
                break;
            }
        }
    };

    let copy_task = async {
        tokio::select! {
            _ = tokio::io::copy(&mut reader, &mut stdin) => {},
            _ = tokio::io::copy(&mut stdout, &mut writer) => {},
        };
    };

    tokio::select! {
        _ = copy_task => {},
        _ = timeout_task => {},
    }

    // 显式释放所有 IO 句柄与管道，向 wsl 发送 EOF，引导其自动关闭
    drop(stdin);
    drop(stdout);
    drop(reader);
    drop(writer);

    // 强行终止子进程并带有超时保护地等待其退出，防止发生协程卡死
    let _ = child.kill().await;
    let _ = tokio::time::timeout(std::time::Duration::from_secs(1), child.wait()).await;
}

// ================== 空闲清理辅助组件 ==================

static START_INSTANT: LazyLock<std::time::Instant> = LazyLock::new(std::time::Instant::now);

/// 获取应用启动后的累计秒数，用于防 NTP 时钟回拨的单调递增时间戳
fn get_elapsed_seconds() -> u64 {
    START_INSTANT.elapsed().as_secs()
}

/// 包装 TcpStream 读写端口，通过拦截 poll 读写事件更新最后活跃时间
struct TimeoutIO<T> {
    inner: T,
    last_activity: Arc<AtomicU64>,
}

impl<T: AsyncRead + Unpin> AsyncRead for TimeoutIO<T> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        let before = buf.filled().len();
        match Pin::new(&mut self.inner).poll_read(cx, buf) {
            Poll::Ready(Ok(())) => {
                if buf.filled().len() > before {
                    self.last_activity
                        .store(get_elapsed_seconds(), Ordering::Relaxed);
                }
                Poll::Ready(Ok(()))
            }
            p => p,
        }
    }
}

impl<T: AsyncWrite + Unpin> AsyncWrite for TimeoutIO<T> {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, std::io::Error>> {
        match Pin::new(&mut self.inner).poll_write(cx, buf) {
            Poll::Ready(Ok(n)) => {
                if n > 0 {
                    self.last_activity
                        .store(get_elapsed_seconds(), Ordering::Relaxed);
                }
                Poll::Ready(Ok(n))
            }
            p => p,
        }
    }

    fn poll_flush(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        Pin::new(&mut self.inner).poll_flush(cx)
    }

    fn poll_shutdown(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        Pin::new(&mut self.inner).poll_shutdown(cx)
    }
}
