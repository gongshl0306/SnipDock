// SnipDock 后端入口。
//
// 启动职责：初始化数据库（建表/索引/默认分类）、把连接以
// `Mutex<Connection>` 注入 State、注册 commands。
// ping 命令保留作为启动连通性自检，M6 收尾时再移除。

mod commands;
mod db;
mod error;

use std::sync::Mutex;
use tauri::Manager;

use commands::{
    create_category, create_snippet, delete_category, delete_snippet,
    list_categories, list_snippets, list_snippets_by_category, mark_snippet_used,
    update_category, update_snippet,
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
        .setup(|app| {
            let data_dir = app
                .path()
                .app_data_dir()
                .expect("failed to resolve app data dir");
            std::fs::create_dir_all(&data_dir).expect("failed to create app data dir");
            let db_path = data_dir.join("snipdock.db");
            let conn = db::open_and_migrate(&db_path).expect("failed to init database");
            app.manage(Mutex::new(conn));
            Ok(())
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running SnipDock");
}
