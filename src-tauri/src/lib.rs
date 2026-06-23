// SnipDock 后端入口。
//
// 启动职责：
//   1. 加载设置（JSON 文件，不存在则默认值）
//   2. 初始化数据库（建表/索引/默认分类）
//   3. 创建系统托盘
//   4. 若 settings.toggle_shortcut 有值，注册全局呼出快捷键
//   5. 拦截主窗口关闭 → 改为隐藏（缩到托盘，进程常驻）
//   6. 注册所有 commands
//
// ping 命令保留作为启动连通性自检，M6 收尾时再移除。

mod commands;
mod db;
mod error;
mod quick_window;
mod settings;
mod tray;

use std::sync::Mutex;
use tauri::{Manager, WindowEvent};
use tauri_plugin_global_shortcut::GlobalShortcutExt;

use commands::{
    create_category, create_snippet, delete_category, delete_snippet,
    get_settings, list_categories, list_snippets, list_snippets_by_category,
    mark_snippet_used, set_toggle_shortcut, update_category, update_snippet,
};

/// 简单连通性检查命令，返回数据库中分类数量，证明 DB 已可用。
#[tauri::command]
fn ping(state: tauri::State<'_, Mutex<rusqlite::Connection>>) -> Result<String, error::AppError> {
    let conn = state.lock().expect("db mutex poisoned");
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM categories", [], |row| row.get(0))
        .map_err(|e| error::AppError::Db(e.to_string()))?;
    Ok(format!("SnipDock 后端就绪，分类数 = {}", count))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            let app_handle = app.handle().clone();

            // 1. 加载设置（先于托盘/快捷键，供后续读取）。
            let app_settings = settings::load(&app_handle);
            app.manage(Mutex::new(app_settings.clone()));

            // 2. 数据库初始化。
            let data_dir = app
                .path()
                .app_data_dir()
                .expect("failed to resolve app data dir");
            std::fs::create_dir_all(&data_dir).expect("failed to create app data dir");
            let db_path = data_dir.join("snipdock.db");
            let conn = db::open_and_migrate(&db_path).expect("failed to init database");
            app.manage(Mutex::new(conn));

            // 3. 创建系统托盘。
            tray::create_tray(app.handle()).expect("failed to create tray");

            // 4. 预创建快速搜索窗口（隐藏）。快捷键即显用。
            quick_window::create_quick_window(app.handle())
                .expect("failed to create quick window");

            // 5. 若配置了呼出快捷键，启动时注册（呼出的是 quick 窗口）。
            //    注册失败只打日志，不阻断启动（用户可在设置里改）。
            if let Some(accel) = app_settings.toggle_shortcut.as_deref() {
                if let Err(e) =
                    app.global_shortcut().on_shortcut(accel, move |app, _s, _e| {
                        quick_window::toggle_quick_window(app);
                    })
                {
                    eprintln!("启动注册呼出快捷键 {accel} 失败：{e}");
                }
            }

            Ok(())
        })
        // 6. 窗口事件：main 关闭→隐藏（缩托盘）；quick 失焦→隐藏。
        .on_window_event(|window, event| {
            let label = window.label();
            match event {
                WindowEvent::CloseRequested { api, .. } => {
                    if label == tray::MAIN_WINDOW {
                        let _ = window.hide();
                        api.prevent_close();
                    }
                }
                WindowEvent::Focused(false) => {
                    if label == quick_window::QUICK_WINDOW {
                        let _ = window.hide();
                    }
                }
                _ => {}
            }
        })
        .invoke_handler(tauri::generate_handler![
            ping,
            list_categories,
            create_category,
            update_category,
            delete_category,
            list_snippets,
            list_snippets_by_category,
            create_snippet,
            update_snippet,
            delete_snippet,
            mark_snippet_used,
            get_settings,
            set_toggle_shortcut,
        ])
        .run(tauri::generate_context!())
        .expect("error while running SnipDock");
}
