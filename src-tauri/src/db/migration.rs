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
    drop_deprecated_snippet_columns(conn)?;
    for stmt in INDEXES {
        conn.execute(stmt, [])?;
    }
    seed_default_category(conn)?;
    Ok(())
}

/// 幂等：删除 snippets 表里已废弃的列（v1 迭代中陆续移除的字段）。
/// 对全新 DB 是空操作；对老 DB 执行 DROP COLUMN。
/// 内置 SQLite >= 3.35.0 支持 DROP COLUMN（rusqlite 0.31 bundled = 3.45.x）。
///
/// 已废弃清单：
///   - tags：v1 不实现 Tag 功能（spec §4）
///   - description / language：v1 简化 schema，只留 title/content/分类/收藏
fn drop_deprecated_snippet_columns(conn: &Connection) -> Result<(), rusqlite::Error> {
    const DEPRECATED: &[&str] = &["tags", "description", "language"];

    // PRAGMA table_info 返回每列一行；name 在第 2 列（index 1）。
    let existing: Vec<String> = {
        let mut stmt = conn.prepare("PRAGMA table_info(snippets)")?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(1))?;
        rows.filter_map(|r| r.ok()).collect()
    };

    for col in DEPRECATED {
        if existing.iter().any(|name| name == col) {
            conn.execute(&format!("ALTER TABLE snippets DROP COLUMN {col}"), [])?;
        }
    }
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
