//! 系统托盘模块 - 托盘图标、菜单（显示/新建/退出）
//! 关闭窗口时最小化到托盘，点击托盘图标恢复窗口

use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, Emitter,
};

/// 创建系统托盘
pub fn create_tray(app: &AppHandle) -> tauri::Result<()> {
    // ---- 托盘菜单项 ----
    let show_item = MenuItem::with_id(app, "show", "显示窗口", true, None::<&str>)?;
    let new_item = MenuItem::with_id(app, "new", "新建笔记", true, None::<&str>)?;
    let sep = PredefinedMenuItem::separator(app)?;
    let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&show_item, &new_item, &sep, &quit_item])?;

    // ---- 构建托盘图标 ----
    TrayIconBuilder::with_id("main-tray")
        .tooltip("LiteNote 灵光记事本")
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .show_menu_on_left_click(false)
        // 菜单点击事件
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => show_main_window(app),
            "new" => {
                show_main_window(app);
                // 通知前端新建笔记
                let _ = app.emit("tray://new-note", ());
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        // 托盘图标点击事件（左键单击/双击恢复窗口）
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_main_window(tray.app_handle());
            }
        })
        .build(app)?;

    Ok(())
}

/// 显示并激活主窗口
fn show_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}
