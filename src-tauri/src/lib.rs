pub mod connection;
pub mod db;
pub mod docker;

use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--minimized"]),
        ))
        .setup(|app| {
            // 设置托盘
            let quit_i = MenuItem::with_id(app, "quit", "退出 Vessel", true, None::<&str>)?;
            let show_i = MenuItem::with_id(app, "show", "显示窗口", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_i, &quit_i])?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => {
                        // 在退出前销毁所有窗口，避免 1412 错误
                        for window in app.webview_windows().values() {
                            let _ = window.destroy();
                        }
                        app.exit(0);
                    }
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            let app_handle = app.handle().clone();
            tauri::async_runtime::block_on(async move {
                let app_dir = app_handle
                    .path()
                    .app_data_dir()
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
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let window = window.clone();
                
                // 默认拦截并隐藏，由前端设置决定是否退出
                api.prevent_close();
                window.hide().unwrap();
            }
        })
        .invoke_handler(tauri::generate_handler![
            docker::list_local_containers,
            docker::start_container,
            docker::stop_container,
            docker::restart_container,
            docker::remove_container,
            docker::inspect_container,
            docker::stream_container_stats,
            docker::stream_container_logs,
            docker::create_container_terminal,
            docker::write_to_terminal,
            docker::resize_container_terminal,
            docker::list_images,
            docker::inspect_image,
            docker::remove_image,
            docker::search_images,
            docker::get_image_history,
            docker::pull_image,
            docker::list_compose_projects,
            docker::read_compose_file,
            docker::write_compose_file,
            docker::run_compose_command,
            docker::list_networks,
            docker::get_network_details,
            docker::remove_network,
            docker::prune_networks,
            docker::disconnect_network,
            docker::list_volumes,
            docker::list_volume_containers,
            docker::remove_volume,
            docker::open_volume_path,
            docker::prune_volumes,
        ])
        .run(tauri::generate_context!())
        .expect("运行 tauri 应用程序时出错");
}
