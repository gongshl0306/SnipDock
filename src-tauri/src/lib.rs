// SnipDock 后端入口。
//
// M1 仅启动一个最小可运行的 Tauri 应用：完成数据库初始化（建表 + 索引 +
// 默认分类），把连接以 `Mutex<Connection>` 注入 State，注册一个 `ping`
// 命令供前端验证后端连通性。后续里程碑在此基础上加 commands 模块。

mod db;
mod error;

use std::sync::Mutex;
use tauri::Manager;

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
            // 解析 app data 目录，确保存在，再打开/初始化数据库。
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
        .invoke_handler(tauri::generate_handler![ping])
        .run(tauri::generate_context!())
        .expect("error while running SnipDock");
}
