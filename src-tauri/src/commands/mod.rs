// Tauri command 模块。
//
// 每个领域（categories / snippets ...）一个子模块。
// lib.rs 通过 `use commands::{...}` 引入具体命令函数，
// generate_handler! 宏据此生成 __cmd__ 辅助项。

pub mod categories;
pub mod settings;
pub mod snippets;

pub use categories::{
    create_category, delete_category, list_categories, update_category,
};
pub use settings::{get_settings, set_toggle_shortcut};
pub use snippets::{
    create_snippet, delete_snippet, list_snippets, list_snippets_by_category,
    mark_snippet_used, update_snippet,
};
