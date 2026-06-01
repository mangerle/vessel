use futures_util::StreamExt;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, LazyLock};
use tauri::{AppHandle, Emitter};
use tokio::sync::{oneshot, Mutex};

pub type StreamMap = HashMap<String, (oneshot::Sender<()>, u64)>;

pub static STATS_STREAMS: LazyLock<Mutex<StreamMap>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub static LOGS_STREAMS: LazyLock<Mutex<StreamMap>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub static STREAM_COUNTER: AtomicU64 = AtomicU64::new(0);

/// 通用的流管理函数
pub async fn spawn_stream_handler<S, T, E>(
    app: AppHandle,
    id: String,
    mut stream: S,
    stream_map: Arc<Mutex<StreamMap>>,
    event_name: String,
    stream_type: &str,
) where
    S: futures_util::Stream<Item = Result<T, E>> + Unpin + Send + 'static,
    T: Serialize + Send + 'static,
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

    tauri::async_runtime::spawn(async move {
        loop {
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
                        None => {
                            break;
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
