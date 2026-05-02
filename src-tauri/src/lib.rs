pub mod connection;
pub mod db;
pub mod docker;

// 了解有关 Tauri 命令的更多信息，请访问 https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("你好，{}！你已从 Rust 收到问候！", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            use tauri::Manager;
            let app_handle = app.handle().clone();
            tauri::async_runtime::block_on(async move {
                let app_dir = app_handle.path().app_data_dir()
                    .map_err(|e| anyhow::anyhow!("无法获取应用数据目录: {}", e))?;
                
                if !app_dir.exists() {
                    std::fs::create_dir_all(&app_dir)
                        .map_err(|e| anyhow::anyhow!("无法创建应用数据目录: {}", e))?;
                }
                
                let db_path = app_dir.join("docker-manager.sqlite");
                let db_url = format!("sqlite:{}?mode=rwc", db_path.display());
                db::init_db(&db_url).await?;
                Ok::<(), anyhow::Error>(())
            })?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            docker::list_local_containers
        ])
        .run(tauri::generate_context!())
        .expect("运行 tauri 应用程序时出错");
}
