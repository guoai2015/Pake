#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod app;
mod util;

use app::{invoke, menu, window};
use invoke::{download_file, download_file_by_binary};
use menu::{get_menu, menu_event_handle};
use tauri::{command};
use tauri_plugin_window_state::Builder as windowStatePlugin;
use util::{get_data_dir, get_pake_config};
use window::get_window;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub fn run_app() {
    let (pake_config, tauri_config) = get_pake_config();
    let show_menu = pake_config.show_menu();
    let menu = get_menu();
    let data_dir = get_data_dir(tauri_config);

    let mut tauri_app = tauri::Builder::default();

    if show_menu {
        tauri_app = tauri_app.menu(menu).on_menu_event(menu_event_handle);
    }

    #[cfg(not(target_os = "macos"))]
    {
        use menu::{get_system_tray, system_tray_handle};

        let show_system_tray = pake_config.show_system_tray();
        let system_tray = get_system_tray(show_menu);

        if show_system_tray {
            tauri_app = tauri_app
                .system_tray(system_tray)
                .on_system_tray_event(system_tray_handle);
        }
    }

    // 在应用程序初始化时创建一个标记
    let can_exit = Arc::new(AtomicBool::new(false));

    // 在处理器中检查是否允许应用程序退出
    #[command]
    fn can_app_exit() -> bool {
        can_exit.load(Ordering::Relaxed)
    }

    // 注册 can_app_exit 命令
    command!(can_app_exit);

    tauri_app
        .plugin(windowStatePlugin::default().build())
        .plugin(tauri_plugin_oauth::init())
        .invoke_handler(tauri::generate_handler![
            download_file,
            download_file_by_binary,
            can_app_exit
        ])
        .setup(|app| {
            let _window = get_window(app, pake_config, data_dir);
            // Prevent initial shaking
            _window.show().unwrap();
            Ok(())
        })
        .on_window_event(|event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event.event() {
                #[cfg(target_os = "macos")]
                {
                    event.window().minimize().unwrap();
                }

                #[cfg(not(target_os = "macos"))]
                event.window().hide().unwrap();

                // 设置标记，阻止应用程序退出
                can_exit.store(false, Ordering::Relaxed);
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn main() {
    run_app()
}
