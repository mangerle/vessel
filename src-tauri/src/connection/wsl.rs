use bollard::{Docker, API_DEFAULT_VERSION};
use std::process::Stdio;
use tokio::net::TcpListener;
use tokio::process::Command;
use std::sync::Arc;
use tokio::sync::Mutex;
use once_cell::sync::Lazy;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use std::pin::Pin;
use std::task::{Context, Poll};

/// WSL 桥接驱动
#[derive(Default)]
pub struct WslBridge {
    pub distro: Option<String>,
}

/// 存储代理端口，避免重复启动代理服务器
static PROXY_PORT: Lazy<Arc<Mutex<Option<u16>>>> = Lazy::new(|| Arc::new(Mutex::new(None)));

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
        docker.ping().await.map_err(|e| format!("WSL Docker 未响应 (请确保 WSL 中已安装 Docker 且 wsl 命令可用): {}", e))?;

        Ok(docker)
    }

    /// 确保 TCP 代理服务器正在运行
    async fn ensure_proxy(&self) -> Result<u16, String> {
        let mut port_lock = PROXY_PORT.lock().await;
        
        if let Some(port) = *port_lock {
            return Ok(port);
        }

        // 绑定到随机可用端口
        let listener = TcpListener::bind("127.0.0.1:0").await
            .map_err(|e| format!("无法绑定代理端口: {}", e))?;
        let port = listener.local_addr().map_err(|e| e.to_string())?.port();

        let distro = self.distro.clone();

        // 在后台运行代理逻辑
        tokio::spawn(async move {
            while let Ok((mut client_socket, _)) = listener.accept().await {
                let distro_clone = distro.clone();
                tokio::spawn(async move {
                    // 为每个连接启动一个 wsl 进程
                    let mut cmd = Command::new("wsl");
                    if let Some(d) = distro_clone
                        && !d.is_empty() && !d.contains("发行版名称") {
                            cmd.args(["-d", &d]);
                        }
                    cmd.args(["docker", "system", "dial-stdio"])
                        .stdin(Stdio::piped())
                        .stdout(Stdio::piped())
                        .stderr(Stdio::null());

                    #[cfg(windows)]
                    {
                        cmd.creation_flags(0x08000000);
                    }

                    let child = cmd.spawn();

                    if let Ok(mut child) = child {
                        let mut stdin = child.stdin.take().unwrap();
                        let mut stdout = child.stdout.take().unwrap();

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
                                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                                let now = get_elapsed_seconds();
                                let last = last_activity_clone.load(Ordering::Relaxed);
                                if now - last > 15 {
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
                        let _ = tokio::time::timeout(
                            std::time::Duration::from_secs(1),
                            child.wait()
                        ).await;
                    }
                });
            }
        });

        *port_lock = Some(port);
        Ok(port)
    }
}

// ================== WSL 进程与连接空闲清理辅助组件 ==================

static START_INSTANT: Lazy<std::time::Instant> = Lazy::new(std::time::Instant::now);

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
                    self.last_activity.store(get_elapsed_seconds(), Ordering::Relaxed);
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
                    self.last_activity.store(get_elapsed_seconds(), Ordering::Relaxed);
                }
                Poll::Ready(Ok(n))
            }
            p => p,
        }
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), std::io::Error>> {
        Pin::new(&mut self.inner).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), std::io::Error>> {
        Pin::new(&mut self.inner).poll_shutdown(cx)
    }
}
