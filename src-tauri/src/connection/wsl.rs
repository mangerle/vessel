use bollard::{Docker, API_DEFAULT_VERSION};
use std::process::Stdio;
use tokio::net::TcpListener;
use tokio::process::Command;
use std::sync::Arc;
use tokio::sync::Mutex;
use once_cell::sync::Lazy;

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
                    if let Some(d) = distro_clone {
                        if !d.is_empty() && !d.contains("发行版名称") {
                            cmd.args(["-d", &d]);
                        }
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

                        let (mut client_reader, mut client_writer) = client_socket.split();

                        // 双向转发流，带有 30 秒空闲超时
                        let _ = tokio::select! {
                            _ = copy_with_idle_timeout(&mut client_reader, &mut stdin) => {},
                            _ = copy_with_idle_timeout(&mut stdout, &mut client_writer) => {},
                        };
                        
                        // 确保进程结束并回收资源
                        let _ = child.kill().await;
                        let _ = child.wait().await;
                    }
                });
            }
        });

        *port_lock = Some(port);
        Ok(port)
    }
}

/// 带有空闲超时的复制逻辑
async fn copy_with_idle_timeout<R, W>(reader: &mut R, writer: &mut W) -> std::io::Result<()> 
where 
    R: tokio::io::AsyncRead + Unpin,
    W: tokio::io::AsyncWrite + Unpin,
{
    use tokio::io::AsyncReadExt;
    use tokio::io::AsyncWriteExt;
    use tokio::time::{timeout, Duration};

    let mut buf = [0u8; 8192];
    loop {
        // 每次读取操作都有 30 秒超时
        let n = match timeout(Duration::from_secs(30), reader.read(&mut buf)).await {
            Ok(Ok(0)) => return Ok(()), // 读取结束
            Ok(Ok(n)) => n,
            Ok(Err(e)) => return Err(e),
            Err(_) => return Err(std::io::Error::new(std::io::ErrorKind::TimedOut, "连接空闲超时")),
        };

        writer.write_all(&buf[..n]).await?;
        writer.flush().await?;
    }
}
