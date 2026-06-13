use futures_util::StreamExt;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::LazyLock;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use tokio::sync::{Mutex, oneshot};

/// Windows 平台创建无窗口进程的标志
#[cfg(windows)]
pub const CREATE_NO_WINDOW: u32 = 0x08000000;

/// 修复 P0-18：前后端事件名集中定义。
///
/// 任何 `app.emit("...")` 都必须引用本模块内的常量或格式化函数；
/// 不允许在业务模块直接拼字面量，否则会与前端 `src/api/events.ts::EVT` 静默漂移。
pub mod events {
    /// Compose 命令实时输出事件
    pub const COMPOSE_CMD_OUTPUT: &str = "compose-cmd-output";
    /// Compose 命令成功完成事件
    pub const COMPOSE_CMD_FINISHED: &str = "compose-cmd-finished";
    /// Compose 命令异常退出事件
    pub const COMPOSE_CMD_ERROR: &str = "compose-cmd-error";

    /// 镜像拉取进度
    pub const IMAGE_PULL_PROGRESS: &str = "image-pull-progress";
    /// 镜像拉取错误
    pub const IMAGE_PULL_ERROR: &str = "image-pull-error";
    /// 镜像拉取完成
    pub const IMAGE_PULL_FINISHED: &str = "image-pull-finished";

    /// 镜像导出进度
    pub const IMAGE_EXPORT_PROGRESS: &str = "image-export-progress";
    /// 镜像导出错误
    pub const IMAGE_EXPORT_ERROR: &str = "image-export-error";
    /// 镜像导出完成
    pub const IMAGE_EXPORT_FINISHED: &str = "image-export-finished";

    /// 镜像导入进度
    pub const IMAGE_IMPORT_PROGRESS: &str = "image-import-progress";
    /// 镜像导入错误
    pub const IMAGE_IMPORT_ERROR: &str = "image-import-error";
    /// 镜像导入完成
    pub const IMAGE_IMPORT_FINISHED: &str = "image-import-finished";

    /// 单实例检测事件
    pub const SINGLE_INSTANCE_DETECTED: &str = "single-instance-detected";
    /// 连接配置变更通知
    pub const CONNECTION_UPDATED: &str = "connection-updated";

    /// 容器统计流事件名（按容器 id 区分频道）
    pub fn container_stats(id: &str) -> String {
        format!("container-stats-{}", id)
    }
    /// 容器日志流事件名（按容器 id 区分频道）
    pub fn container_logs(id: &str) -> String {
        format!("container-logs-{}", id)
    }
    /// 终端 stdout 事件名（按 exec id 区分频道）
    pub fn container_terminal_stdout(exec_id: &str) -> String {
        format!("container-terminal-stdout-{}", exec_id)
    }
    /// 终端退出事件名（按 exec id 区分频道）
    pub fn container_terminal_exit(exec_id: &str) -> String {
        format!("container-terminal-exit-{}", exec_id)
    }
}

pub type StreamMap = HashMap<String, (oneshot::Sender<()>, u64)>;

pub static STATS_STREAMS: LazyLock<Mutex<StreamMap>> = LazyLock::new(|| Mutex::new(HashMap::new()));

pub static LOGS_STREAMS: LazyLock<Mutex<StreamMap>> = LazyLock::new(|| Mutex::new(HashMap::new()));

pub static STREAM_COUNTER: AtomicU64 = AtomicU64::new(0);

/// 通用的流管理函数
///
/// `throttle`: 若为 `Some(duration)`，则窗口内只保留最新数据并按周期 emit（替代语义）；
///              末尾与取消前会 flush 最后一帧；错误与致命流终止仍立即上报。
pub async fn spawn_stream_handler<S, T, E>(
    app: AppHandle,
    id: String,
    mut stream: S,
    stream_map: &'static Mutex<StreamMap>,
    event_name: String,
    stream_type: &str,
    throttle: Option<Duration>,
) where
    S: futures_util::Stream<Item = Result<T, E>> + Unpin + Send + 'static,
    T: Serialize + Clone + Send + 'static,
    E: std::fmt::Display + Send + 'static,
{
    let (tx, mut rx) = oneshot::channel::<()>();
    let token = STREAM_COUNTER.fetch_add(1, Ordering::SeqCst);

    {
        let mut streams = stream_map.lock().await;
        if let Some((old_tx, _)) = streams.insert(id.clone(), (tx, token)) {
            let _ = old_tx.send(());
        }
    }

    let id_clone = id.clone();
    let stream_type = stream_type.to_string();
    let event_name = event_name.clone();

    // 节流 ticker：首次立即 tick，需要在循环开始前消耗以避免首次 select 命中空 tick
    let mut ticker = throttle.map(|d| {
        let mut t = tokio::time::interval(d);
        t.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        t
    });

    tauri::async_runtime::spawn(async move {
        if let Some(t) = ticker.as_mut() {
            t.tick().await; // 消耗 ticker 首次立即触发
        }
        // 缓冲最新待发数据（节流时使用）
        let mut pending: Option<T> = None;

        loop {
            if let Some(t) = ticker.as_mut() {
                tokio::select! {
                    _ = &mut rx => {
                        log::debug!("收到停止信号，停止 {} 流: {}", stream_type, id_clone);
                        break;
                    }
                    msg = stream.next() => {
                        match msg {
                            Some(Ok(data)) => {
                                // 节流：覆盖 pending，等下次 tick 统一 emit
                                pending = Some(data);
                            }
                            Some(Err(e)) => {
                                log::error!("获取 {} 数据失败: {}", stream_type, e);
                                break;
                            }
                            None => {
                                // 流自然结束：flush 最后一帧
                                if let Some(data) = pending.take() {
                                    let _ = app.emit(&event_name, data);
                                }
                                break;
                            }
                        }
                    }
                    _ = t.tick() => {
                        if let Some(data) = pending.take()
                            && let Err(e) = app.emit(&event_name, data)
                        {
                            log::error!("发送 {} 事件失败: {}", stream_type, e);
                            break;
                        }
                    }
                }
            } else {
                tokio::select! {
                    _ = &mut rx => {
                        log::debug!("收到停止信号，停止 {} 流: {}", stream_type, id_clone);
                        break;
                    }
                    msg = stream.next() => {
                        match msg {
                            Some(Ok(data)) => {
                                if let Err(e) = app.emit(&event_name, data) {
                                    log::error!("发送 {} 事件失败: {}", stream_type, e);
                                    break;
                                }
                            }
                            Some(Err(e)) => {
                                log::error!("获取 {} 数据失败: {}", stream_type, e);
                                break;
                            }
                            None => break,
                        }
                    }
                }
            }
        }
        // 协程退出时清理
        let mut streams = stream_map.lock().await;
        if let Some((_, t)) = streams.get(&id_clone)
            && *t == token
        {
            streams.remove(&id_clone);
        }
    });
}

#[macro_export]
macro_rules! handle_docker_op {
    ($op_name:expr, $target:expr, $future:expr) => {
        match $future.await {
            Ok(_) => {
                log::info!("{} {} 成功", $op_name, $target);
                Ok(())
            }
            Err(e) => {
                log::error!("{} {} 失败: {}", $op_name, $target, e);
                Err(e.into())
            }
        }
    };
}
