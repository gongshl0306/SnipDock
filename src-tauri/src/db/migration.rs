// 数据库迁移：建表 + 建索引 + 默认分类初始化。
//
// 全部使用 `IF NOT EXISTS`，重复执行幂等。SQL 字面量与 target.md §5 一致，
// 只在前端契约（字段名、类型、默认值、外键）保持一致的范围内组织代码。

use chrono::Utc;
use rusqlite::{params, Connection};

const CREATE_CATEGORIES: &str = "
CREATE TABLE IF NOT EXISTS categories (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
)";

const CREATE_SNIPPETS: &str = "
CREATE TABLE IF NOT EXISTS snippets (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    category_id INTEGER NOT NULL,
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    tags TEXT NOT NULL DEFAULT '',
    description TEXT NOT NULL DEFAULT '',
    language TEXT NOT NULL DEFAULT 'text',
    favorite INTEGER NOT NULL DEFAULT 0,
    used_count INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    last_used_at TEXT,
    FOREIGN KEY (category_id) REFERENCES categories(id)
)";

// 索引名严格遵循 target.md §5.4
const INDEXES: &[&str] = &[
    "CREATE INDEX IF NOT EXISTS idx_snippets_category_id ON snippets(category_id)",
    "CREATE INDEX IF NOT EXISTS idx_snippets_updated_at ON snippets(updated_at)",
    "CREATE INDEX IF NOT EXISTS idx_snippets_used_count ON snippets(used_count)",
    "CREATE INDEX IF NOT EXISTS idx_snippets_last_used_at ON snippets(last_used_at)",
];

pub fn run(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute(CREATE_CATEGORIES, [])?;
    conn.execute(CREATE_SNIPPETS, [])?;
    for stmt in INDEXES {
        conn.execute(stmt, [])?;
    }
    seed_default_category(conn)?;
    Ok(())
}

fn seed_default_category(conn: &Connection) -> Result<(), rusqlite::Error> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM categories", [], |row| row.get(0))?;
    if count > 0 {
        return Ok(());
    }
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO categories (name, description, sort_order, created_at, updated_at)
         VALUES (?1, ?2, 0, ?3, ?3)",
        params!["默认", "默认分类", now],
    )?;
    Ok(())
}
