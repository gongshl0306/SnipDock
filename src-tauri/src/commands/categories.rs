// 分类相关的 Tauri commands。
//
// 命令名、参数形状严格遵循 target.md §7.1 / §7.3。
// 校验规则见 design.md §5.4：name trim 后非空，description 默认空串。
//
// 参数命名约定：invoke 的 key 必须与 Rust 命令参数名完全一致
// （Tauri 2 不自动转 camelCase）。这里所有参数都是单词，无歧义。

use std::sync::Mutex;

use rusqlite::Connection;

use crate::db::repo::{self, CategoryRow};
use crate::error::AppError;

/// 规范化分类名：trim 后非空校验，返回 trim 后的 owned String。
fn normalize_name(raw: &str) -> Result<String, AppError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(AppError::CategoryNameEmpty);
    }
    Ok(trimmed.to_string())
}

/// description：None 或 trim 后的空串都归一化为 ""，避免存 null/空白。
fn normalize_description(raw: Option<&str>) -> String {
    raw.map(|s| s.trim()).unwrap_or("").to_string()
}

/// 返回全部分类，按 sort_order / created_at 升序。
#[tauri::command]
pub fn list_categories(
    state: tauri::State<'_, Mutex<Connection>>,
) -> Result<Vec<CategoryRow>, AppError> {
    let conn = state.lock().expect("db mutex poisoned");
    Ok(repo::list_categories(&conn)?)
}

/// 新建分类。返回插入后的完整行（含生成的 id / 时间戳 / sort_order）。
#[tauri::command]
pub fn create_category(
    name: String,
    description: Option<String>,
    state: tauri::State<'_, Mutex<Connection>>,
) -> Result<CategoryRow, AppError> {
    let name = normalize_name(&name)?;
    let description = normalize_description(description.as_deref());
    let conn = state.lock().expect("db mutex poisoned");
    Ok(repo::insert_category(&conn, &name, &description)?)
}

/// 更新分类名/描述。分类不存在返回 NotFound。
#[tauri::command]
pub fn update_category(
    id: i64,
    name: String,
    description: Option<String>,
    state: tauri::State<'_, Mutex<Connection>>,
) -> Result<CategoryRow, AppError> {
    let name = normalize_name(&name)?;
    let description = normalize_description(description.as_deref());
    let conn = state.lock().expect("db mutex poisoned");
    repo::update_category(&conn, id, &name, &description)
}

/// 删除分类。分类下有片段时返回 CategoryNotEmpty（合约：message
/// 字面量 "Category is not empty"）。
#[tauri::command]
pub fn delete_category(
    id: i64,
    state: tauri::State<'_, Mutex<Connection>>,
) -> Result<(), AppError> {
    let conn = state.lock().expect("db mutex poisoned");
    repo::delete_category(&conn, id)
}
