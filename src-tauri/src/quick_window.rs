// 快速搜索窗口（Alfred/Raycast 模型）。
//
// 行为（用户已确认）：
//   - 应用启动时预创建（hidden），快捷键即时 show（零延迟）
//   - 失焦自动隐藏（lib.rs 的 on_window_event 监听 Focused(false)）
//   - 复制成功后短暂显示「已复制」再隐藏（前端控制，这里只提供 hide）
//   - 快捷键 toggle 这个窗口；托盘 toggle 主窗口，互不干扰
//
// 窗口属性：无边框、置顶、不占任务栏、不可缩放、初始隐藏、居中。
// 用 WebviewUrl::App 加载与主窗口相同的 index.html，前端按
// getCurrentWindow().label 区分渲染 App 还是 QuickApp（外部 URL 会
// 让 Tauri IPC 不注入，导致 invoke 失败 → 白屏，所以必须用 App 而非 External）。

use tauri::{Manager, Runtime, WebviewUrl, WebviewWindowBuilder};

/// 快速窗口的 label。
pub const QUICK_WINDOW: &str = "quick";

/// 启动时预创建快速窗口（隐藏）。已存在则跳过（幂等）。
pub fn create_quick_window<R: Runtime>(app: &tauri::AppHandle<R>) -> tauri::Result<()> {
    if app.get_webview_window(QUICK_WINDOW).is_some() {
        return Ok(());
    }

    WebviewWindowBuilder::new(app, QUICK_WINDOW, WebviewUrl::App("index.html".into()))
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
