// 应用层错误类型。
//
// 所有 #[tauri::command] 返回 `Result<T, AppError>`，由 serde 序列化为前端
// 可读的对象 `{ type: "...", message: "..." }`。约束：
//   - `CategoryNotEmpty` 的 message 字面量必须是 "Category is not empty"
//     （target.md §7.3 的合约）。
//   - 所有 rusqlite 错误经 `Db(String)` 包装，避免把 SQLite 内部细节暴露。

use serde::Serialize;

#[derive(Debug, thiserror::Error, Serialize)]
#[serde(tag = "type", content = "message")]
pub enum AppError {
    #[error("Category is not empty")]
    CategoryNotEmpty,

    #[error("Category name cannot be empty")]
    CategoryNameEmpty,

    #[error("Snippet title cannot be empty")]
    SnippetTitleEmpty,

    #[error("Snippet content cannot be empty")]
    SnippetContentEmpty,

    #[error("Not found: {kind} id={id}")]
    NotFound { kind: &'static str, id: i64 },

    #[error("Database error: {0}")]
    Db(String),
}

impl From<rusqlite::Error> for AppError {
    fn from(err: rusqlite::Error) -> Self {
        AppError::Db(err.to_string())
    }
}
