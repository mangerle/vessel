pub mod connection;
pub mod docker;
pub mod error;

use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager,
};

/// 修复 P1-11：构建并安装自定义 tokio multi-thread runtime 作为 tauri 全局 runtime。
/// tauri 2 默认 runtime 是 multi-thread 但 worker 数依赖 tokio 默认启发式（=CPU 核数），
/// 对本应用这种 I/O 密集 + 多 Docker 长连接流的场景偏保守。显式设置为
/// `max(NUM_CPUS * 2, 8)`，让 bollard 流、SSH 子进程、HTTP 探测等可并行。
/// 注意：必须持有 Runtime 至 run() 结束，否则 worker 线程会被 drop。
fn install_async_runtime() -> tokio::runtime::Runtime {
    let worker_threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
        .saturating_mul(2)
        .max(8);
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(worker_threads)
        .enable_all()
        .thread_name("vessel-tokio")
        .build()
        .expect("构建 tokio runtime 失败");
    // 将 runtime handle 设为 tauri 全局 async runtime，后续 tauri::async_runtime::spawn
    // 与 #[tauri::command] 都将使用此 runtime。
    tauri::async_runtime::set(runtime.handle().clone());
    runtime
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 修复 P1-11：安装显式配置的 multi-thread runtime 作为全局 async runtime
    let _async_runtime_guard = install_async_runtime();
    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .targets([
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Stdout),
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Webview),
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::LogDir {
                        file_name: Some("vessel".to_string()),
                    }),
                ])
                .level(log::LevelFilter::Info)
                .level_for("hyper", log::LevelFilter::Warn)
                .level_for("tokio", log::LevelFilter::Warn)
                .level_for("bollard", log::LevelFilter::Warn)
                .level_for("vessel_lib", log::LevelFilter::Info)
                .build(),
        )
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--minimized"]),
        ))
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            // 检测到新实例启动，将已有窗口显示并聚焦
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
            // 发送自定义事件给前端
            let _ = app.emit(docker::events::SINGLE_INSTANCE_DETECTED, ());
        }))
        .setup(|app| {
            // 设置托盘
            let quit_i = MenuItem::with_id(app, "quit", "退出 Vessel", true, None::<&str>)?;
            let show_i = MenuItem::with_id(app, "show", "显示窗口", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_i, &quit_i])?;
            let mut tray_builder = TrayIconBuilder::new()
                .menu(&menu)
                .show_menu_on_left_click(false);

            if let Some(icon) = app.default_window_icon() {
                tray_builder = tray_builder.icon(icon.clone());
            }

            let _tray = tray_builder
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

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let window = window.clone();
                
                // 默认拦截并隐藏，由前端设置决定是否退出
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![
            docker::list_local_containers,
            docker::start_container,
            docker::stop_container,
            docker::restart_container,
            docker::remove_container,
            docker::rename_container,
            docker::commit_container,
            docker::inspect_container,
            docker::pause_container,
            docker::unpause_container,
            docker::top_container,
            docker::exec_container,
            docker::stream_container_stats,
            docker::close_container_stats,
            docker::stream_container_logs,
            docker::close_container_logs,
            docker::create_container_terminal,
            docker::close_container_terminal,
            docker::write_to_terminal,
            docker::resize_container_terminal,
            docker::list_images,
            docker::inspect_image,
            docker::remove_image,
            docker::search_images,
            docker::get_image_history,
            docker::pull_image,
            docker::run_image,
            docker::export_image,
            docker::import_image,
            docker::tag_image,
            docker::prune_images,
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
            docker::list_volume_files,
            docker::read_volume_text_file,
            docker::write_volume_text_file,
            docker::list_wsl_distros,
            docker::open_config_dir,
            docker::open_log_dir,
            docker::list_container_files,
            docker::download_file_from_container,
            docker::upload_file_to_container,
            docker::delete_container_file,
            docker::create_container_file,
            docker::rename_container_file,
            docker::read_container_text_file,
            docker::write_container_text_file,
            connection::update_connection_config,
            connection::ping_docker,
            connection::diagnose_ssh_connection,
        ])
        .run(tauri::generate_context!())
        .expect("运行 tauri 应用程序时出错");
}
