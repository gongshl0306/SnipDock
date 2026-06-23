// 系统托盘 + 主窗口显隐逻辑。
//
// 行为（用户已确认）：
//   - 点窗口关闭按钮 X → 拦截 close，隐藏窗口（不销毁），进程常驻托盘
//   - 点托盘图标 / 托盘菜单「显示」→ toggle 主窗口
//   - 托盘菜单「退出」→ 真正退出（app.exit）
//   - 全局快捷键（settings.toggle_shortcut）→ 也调 toggle_main_window
//
// 关闭拦截在 lib.rs 的 on_window_event 里接 WindowEvent::CloseRequested，
// 这里只提供 toggle/show/hide 等纯函数式工具。

use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, Runtime,
};

/// 主窗口的 label（与 tauri.conf.json 的 windows[0].label 一致）。
pub const MAIN_WINDOW: &str = "main";

/// 托盘菜单项 id（用这些 id 在事件回调里区分点击）。
const MENU_SHOW: &str = "show";
const MENU_QUIT: &str = "quit";

/// 创建系统托盘。图标用打包进二进制的 icon.ico。
pub fn create_tray<R: Runtime>(app: &tauri::AppHandle<R>) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, MENU_SHOW, "显示 / 隐藏", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, MENU_QUIT, "退出", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &quit])?;

    TrayIconBuilder::with_id("main-tray")
        .icon(app.default_window_icon().cloned().expect("no window icon"))
        .tooltip("SnipDock")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            MENU_SHOW => {
                toggle_main_window(app);
            }
            MENU_QUIT => {
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            // 左键单击切换窗口（更符合 Win/macOS 直觉）。
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                toggle_main_window(tray.app_handle());
            }
        })
        .build(app)?;
    Ok(())
}

/// 切换主窗口可见性：可见→隐藏，隐藏→显示+聚焦。
/// 托盘点击、托盘菜单「显示」、全局快捷键都调本函数。
pub fn toggle_main_window<R: Runtime, M: Manager<R>>(manager: &M) {
    if let Some(window) = manager.get_webview_window(MAIN_WINDOW) {
        if window.is_visible().unwrap_or(false) {
            let _ = window.hide();
        } else {
            let _ = window.show();
            let _ = window.set_focus();
        }
    }
}
