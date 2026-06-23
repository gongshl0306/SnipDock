// 片段相关的 Tauri commands。
//
// 命令名、参数形状遵循 target.md §7.2。v1 不引入 Tag 字段。
// 偏差（design.md §5.2 已确认）：不实现 search_snippets / copy_snippet。
//   - 搜索：前端 computed 内存过滤（M4）
//   - 复制：前端 navigator.clipboard + mark_snippet_used
//
// 校验规则（design.md §5.4）：title/content trim 后非空。

use std::sync::Mutex;

use rusqlite::Connection;

use crate::db::repo::{self, SnippetRow};
use crate::error::AppError;

/// 新建片段载荷。字段 snake_case 对应 spec §6.3。
/// Option 字段：前端可省略，由后端填默认值。
#[derive(Debug, serde::Deserialize)]
pub struct CreateSnippetPayload {
    pub category_id: i64,
    pub title: String,
    pub content: String,
    pub favorite: Option<i64>,
}

/// 更新片段载荷。spec §6.4。id 必填，其余覆盖。
#[derive(Debug, serde::Deserialize)]
pub struct UpdateSnippetPayload {
    pub id: i64,
    pub category_id: i64,
    pub title: String,
    pub content: String,
    pub favorite: Option<i64>,
}

fn normalize_title(raw: &str) -> Result<String, AppError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(AppError::SnippetTitleEmpty);
    }
    Ok(trimmed.to_string())
}

fn normalize_content(raw: &str) -> Result<String, AppError> {
    // content 允许内部空白，只 trim 首尾。纯空白算空。
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(AppError::SnippetContentEmpty);
    }
    Ok(raw.to_string())
}

/// 列出全部片段（「全部」视图）。已按 spec §9.4 排序。
#[tauri::command]
pub fn list_snippets(
    state: tauri::State<'_, Mutex<Connection>>,
) -> Result<Vec<SnippetRow>, AppError> {
    let conn = state.lock().expect("db mutex poisoned");
    Ok(repo::list_snippets(&conn)?)
}

/// 列出某分类下的片段。
#[tauri::command]
pub fn list_snippets_by_category(
    category_id: i64,
    state: tauri::State<'_, Mutex<Connection>>,
) -> Result<Vec<SnippetRow>, AppError> {
    let conn = state.lock().expect("db mutex poisoned");
    Ok(repo::list_snippets_by_category(&conn, category_id)?)
}

/// 列出已收藏的片段（跨原分类）。
#[tauri::command]
pub fn list_snippets_favorites(
    state: tauri::State<'_, Mutex<Connection>>,
) -> Result<Vec<SnippetRow>, AppError> {
    let conn = state.lock().expect("db mutex poisoned");
    Ok(repo::list_snippets_favorites(&conn)?)
}

/// 新建片段。返回插入后的完整行。
#[tauri::command]
pub fn create_snippet(
    payload: CreateSnippetPayload,
    state: tauri::State<'_, Mutex<Connection>>,
) -> Result<SnippetRow, AppError> {
    let title = normalize_title(&payload.title)?;
    let content = normalize_content(&payload.content)?;
    let favorite = payload.favorite.unwrap_or(0);

    let conn = state.lock().expect("db mutex poisoned");
    Ok(repo::insert_snippet(
        &conn,
        payload.category_id,
        &title,
        &content,
        favorite,
    )?)
}

/// 更新片段。不存在返回 NotFound。
#[tauri::command]
pub fn update_snippet(
    payload: UpdateSnippetPayload,
    state: tauri::State<'_, Mutex<Connection>>,
) -> Result<SnippetRow, AppError> {
    let id = payload.id;
    let title = normalize_title(&payload.title)?;
    let content = normalize_content(&payload.content)?;
    let favorite = payload.favorite.unwrap_or(0);

    let conn = state.lock().expect("db mutex poisoned");
    repo::update_snippet(&conn, id, payload.category_id, &title, &content, favorite)
}

/// 删除片段。不存在返回 NotFound。
#[tauri::command]
pub fn delete_snippet(
    id: i64,
    state: tauri::State<'_, Mutex<Connection>>,
) -> Result<(), AppError> {
    let conn = state.lock().expect("db mutex poisoned");
    repo::delete_snippet(&conn, id)
}

/// 标记片段被使用：used_count += 1, last_used_at = now。
/// 复制流程（design.md §6.4）在复制成功后调用。
#[tauri::command]
pub fn mark_snippet_used(
    id: i64,
    state: tauri::State<'_, Mutex<Connection>>,
) -> Result<SnippetRow, AppError> {
    let conn = state.lock().expect("db mutex poisoned");
    repo::mark_snippet_used(&conn, id)
}
