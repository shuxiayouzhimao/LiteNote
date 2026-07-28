//! LiteNote 库入口 - 组装 Tauri 应用
//! 注册插件、命令、初始化数据库、设置托盘、全局快捷键、窗口关闭行为

mod commands;
mod config;
mod db;
mod tray;

use db::DbState;
use std::sync::Mutex;
use tauri::{Emitter, Manager, WindowEvent};
use tauri_plugin_global_shortcut::ShortcutState;

/// 应用主入口（供 main.rs 和移动端调用）
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // ---- 插件注册 ----
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        // 开机自启插件（macOS/Linux 可传启动参数，这里为空）
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        // 全局快捷键插件：Ctrl+Shift+N 唤出新建笔记
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_shortcut("CmdOrCtrl+Shift+N")
                .expect("注册全局快捷键失败")
                .with_handler(|app, _shortcut, event| {
                    // 只在按下时触发
                    if event.state() == ShortcutState::Pressed {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.unminimize();
                            let _ = window.set_focus();
                            // 通知前端新建笔记
                            let _ = app.emit("shortcut://new-note", ());
                        }
                    }
                })
                .build(),
        )
        // ---- 应用初始化 ----
        .setup(|app| {
            let handle = app.handle();

            // 1. 初始化数据库
            let db_file = config::db_path(handle).expect("获取数据库路径失败");
            let conn = db::init_db(&db_file).expect("初始化数据库失败");
            app.manage(DbState(Mutex::new(conn)));

            // 2. 创建系统托盘
            tray::create_tray(handle)?;

            Ok(())
        })
        // ---- 窗口事件：关闭时隐藏到托盘而非退出 ----
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                // 阻止默认关闭，改为隐藏
                api.prevent_close();
                let _ = window.hide();
            }
        })
        // ---- 注册所有前端可调用的命令 ----
        .invoke_handler(tauri::generate_handler![
            commands::list_notes,
            commands::search_notes,
            commands::get_note,
            commands::create_note,
            commands::update_note,
            commands::delete_note,
            commands::restore_note,
            commands::permanent_delete,
            commands::clear_trash,
            commands::toggle_favorite,
            commands::toggle_pin,
            commands::get_stats,
            commands::export_note,
            commands::load_config,
            commands::save_config,
        ])
        .run(tauri::generate_context!())
        .expect("运行 LiteNote 应用出错");
}
