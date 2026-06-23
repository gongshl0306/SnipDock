// 快速搜索窗口（Alfred/Raycast 模型）。
//
// 行为（用户已确认）：
//   - 应用启动时预创建（hidden），快捷键即时 show（零延迟）
//   - 失焦自动隐藏（lib.rs 的 on_window_event 监听 Focused(false)）
//   - 复制成功后短暂显示「已复制」再隐藏（前端控制，这里只提供 hide）
//   - 快捷键 toggle 这个窗口；托盘 toggle 主窗口，互不干扰
//
// 窗口属性：无边框、置顶、不占任务栏、不可缩放、初始隐藏、居中。
// 通过 URL hash `#quick` 让前端 main.ts 渲染 QuickApp 而非 App。

use tauri::{Manager, Runtime, WebviewUrl, WebviewWindowBuilder};

/// 快速窗口的 label。
pub const QUICK_WINDOW: &str = "quick";

/// 启动时预创建快速窗口（隐藏）。已存在则跳过（幂等）。
pub fn create_quick_window<R: Runtime>(app: &tauri::AppHandle<R>) -> tauri::Result<()> {
    if app.get_webview_window(QUICK_WINDOW).is_some() {
        return Ok(());
    }

    // 复用主窗口的 URL（devUrl 或 frontendDist），追加 #quick hash。
    // 这样前端用 window.location.hash 判断渲染哪个根组件。
    let main_url = app
        .get_webview_window(crate::tray::MAIN_WINDOW)
        .and_then(|w| w.url().ok())
        .map(|u| u.to_string())
        .unwrap_or_default();
    let quick_url = format!("{main_url}#quick");

    WebviewWindowBuilder::new(app, QUICK_WINDOW, WebviewUrl::External(quick_url.parse().unwrap_or_else(|_| "tauri://localhost#quick".parse().unwrap())))
        .title("SnipDock Quick")
        .inner_size(600.0, 420.0)
        .center()
        .decorations(false)
        .always_on_top(true)
        .visible(false)
        .resizable(false)
        .skip_taskbar(true)
        .focused(false)
        .build()?;
    Ok(())
}

/// 切换快速窗口可见性：可见→隐藏，隐藏→显示+聚焦+居中。
pub fn toggle_quick_window<R: Runtime, M: Manager<R>>(manager: &M) {
    if let Some(window) = manager.get_webview_window(QUICK_WINDOW) {
        if window.is_visible().unwrap_or(false) {
            let _ = window.hide();
        } else {
            // 重新居中（防多屏/分辨率变化后位置偏移），再显示聚焦。
            let _ = window.center();
            let _ = window.show();
            let _ = window.set_focus();
        }
    }
}
