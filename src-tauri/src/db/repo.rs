// SQL 数据访问层。
//
// 所有 rusqlite 调用集中在这里，commands 层只做参数校验和调度。
// 未来若换 sqlx 或加 FTS5，只改本文件，commands 层不受影响。
//
// 字段名、排序、外键约束严格遵循 target.md §5 / §7.3。

use rusqlite::{params, Connection, Row};

use crate::error::AppError;

/// categories 表的一行。字段名 snake_case，序列化后直接给前端
/// （对应 spec §6.1 的 Category 接口）。
#[derive(Debug, serde::Serialize)]
pub struct CategoryRow {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub sort_order: i64,
    pub created_at: String,
    pub updated_at: String,
}

fn row_to_category(row: &Row<'_>) -> rusqlite::Result<CategoryRow> {
    Ok(CategoryRow {
        id: row.get("id")?,
        name: row.get("name")?,
        description: row.get("description")?,
        sort_order: row.get("sort_order")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

const SELECT_CATEGORY_BY_ID: &str = "
SELECT id, name, description, sort_order, created_at, updated_at
FROM categories WHERE id = ?1";

/// 按 sort_order ASC, created_at ASC 返回全部分类（target.md §7.3）。
pub fn list_categories(conn: &Connection) -> Result<Vec<CategoryRow>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, name, description, sort_order, created_at, updated_at
         FROM categories
         ORDER BY sort_order ASC, created_at ASC",
    )?;
    let rows = stmt.query_map([], row_to_category)?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

/// 统计某分类下的片段数，用于删除前的非空校验。
pub fn count_snippets_in_category(conn: &Connection, id: i64) -> Result<i64, rusqlite::Error> {
    conn.query_row(
        "SELECT COUNT(*) FROM snippets WHERE category_id = ?1",
        params![id],
        |row| row.get(0),
    )
}

/// 新建分类。sort_order 取当前最大值 +1（无分类则为 0）。
/// INSERT 后回查完整行返回。
pub fn insert_category(
    conn: &Connection,
    name: &str,
    description: &str,
) -> Result<CategoryRow, rusqlite::Error> {
    let now = chrono::Utc::now().to_rfc3339();
    let max_order: i64 = conn
        .query_row("SELECT COALESCE(MAX(sort_order), -1) FROM categories", [], |row| {
            row.get(0)
        })?;
    conn.execute(
        "INSERT INTO categories (name, description, sort_order, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?4)",
        params![name, description, max_order + 1, now],
    )?;
    let id = conn.last_insert_rowid();
    conn.query_row(SELECT_CATEGORY_BY_ID, params![id], row_to_category)
}

/// 更新分类。分类不存在返回 AppError::NotFound。
/// updated_at 刷新为当前时间。
pub fn update_category(
    conn: &Connection,
    id: i64,
    name: &str,
    description: &str,
) -> Result<CategoryRow, AppError> {
    // 先确认存在，避免静默 UPDATE 0 行。
    let exists: i64 = conn.query_row(
        "SELECT COUNT(*) FROM categories WHERE id = ?1",
        params![id],
        |row| row.get(0),
    )?;
    if exists == 0 {
        return Err(AppError::NotFound {
            kind: "category",
            id,
        });
    }
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE categories SET name = ?1, description = ?2, updated_at = ?3 WHERE id = ?4",
        params![name, description, now, id],
    )?;
    Ok(conn.query_row(SELECT_CATEGORY_BY_ID, params![id], row_to_category)?)
}

/// 删除分类。分类下有片段时返回 AppError::CategoryNotEmpty
/// （message 字面量必须是 "Category is not empty"，见 error.rs）。
pub fn delete_category(conn: &Connection, id: i64) -> Result<(), AppError> {
    if count_snippets_in_category(conn, id)? > 0 {
        return Err(AppError::CategoryNotEmpty);
    }
    let affected = conn.execute("DELETE FROM categories WHERE id = ?1", params![id])?;
    if affected == 0 {
        return Err(AppError::NotFound {
            kind: "category",
            id,
        });
    }
    Ok(())
}

// =====================================================================
// snippets 表
// =====================================================================

/// snippets 表的一行。`category_name` 来自 LEFT JOIN categories，
/// 在「全部」视图下展示片段归属用（spec §6.2 的可选字段）。
#[derive(Debug, serde::Serialize)]
pub struct SnippetRow {
    pub id: i64,
    pub category_id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category_name: Option<String>,
    pub title: String,
    pub content: String,
    pub favorite: i64,
    pub used_count: i64,
    pub created_at: String,
    pub updated_at: String,
    pub last_used_at: Option<String>,
}

fn row_to_snippet(row: &Row<'_>) -> rusqlite::Result<SnippetRow> {
    Ok(SnippetRow {
        id: row.get("id")?,
        category_id: row.get("category_id")?,
        category_name: row.get::<_, Option<String>>("category_name")?,
        title: row.get("title")?,
        content: row.get("content")?,
        favorite: row.get("favorite")?,
        used_count: row.get("used_count")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
        last_used_at: row.get("last_used_at")?,
    })
}

/// 列表查询的公共 SQL 片段：JOIN categories 取 category_name，
/// 按 spec §9.4 排序：favorite DESC, last_used_at DESC, used_count DESC, updated_at DESC。
/// last_used_at 为 NULL 时排到最后（用 MAX(NULL, '1970...') 兜底）。
const SNIPPET_SELECT_ORDER: &str = "
SELECT s.id, s.category_id, c.name AS category_name,
       s.title, s.content,
       s.favorite, s.used_count, s.created_at, s.updated_at, s.last_used_at
FROM snippets s
LEFT JOIN categories c ON c.id = s.category_id
{FILTER}
ORDER BY s.favorite DESC,
         COALESCE(s.last_used_at, '0') DESC,
         s.used_count DESC,
         s.updated_at DESC";

/// 全量片段（「全部」视图）。
pub fn list_snippets(conn: &Connection) -> Result<Vec<SnippetRow>, rusqlite::Error> {
    let sql = SNIPPET_SELECT_ORDER.replace("{FILTER}", "");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], row_to_snippet)?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

/// 某分类下的片段。
pub fn list_snippets_by_category(
    conn: &Connection,
    category_id: i64,
) -> Result<Vec<SnippetRow>, rusqlite::Error> {
    let sql = SNIPPET_SELECT_ORDER.replace("{FILTER}", "WHERE s.category_id = ?1");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params![category_id], row_to_snippet)?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

/// 全部已收藏（favorite=1）的片段。跨原分类。「收藏」虚拟分类用。
pub fn list_snippets_favorites(conn: &Connection) -> Result<Vec<SnippetRow>, rusqlite::Error> {
    let sql = SNIPPET_SELECT_ORDER.replace("{FILTER}", "WHERE s.favorite = 1");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], row_to_snippet)?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

const SELECT_SNIPPET_BY_ID: &str = "
SELECT s.id, s.category_id, c.name AS category_name,
       s.title, s.content,
       s.favorite, s.used_count, s.created_at, s.updated_at, s.last_used_at
FROM snippets s
LEFT JOIN categories c ON c.id = s.category_id
WHERE s.id = ?1";

/// 新建片段。调用方保证 title/content 已 trim 非空。
/// 默认值：favorite=0, used_count=0, last_used_at=NULL, created_at=updated_at=now。
pub fn insert_snippet(
    conn: &Connection,
    category_id: i64,
    title: &str,
    content: &str,
    favorite: i64,
) -> Result<SnippetRow, rusqlite::Error> {
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO snippets
           (category_id, title, content,
            favorite, used_count, created_at, updated_at, last_used_at)
         VALUES (?1, ?2, ?3, ?4, 0, ?5, ?5, NULL)",
        params![category_id, title, content, favorite, now],
    )?;
    let id = conn.last_insert_rowid();
    conn.query_row(SELECT_SNIPPET_BY_ID, params![id], row_to_snippet)
}

/// 更新片段。分类不存在由外键约束兜底（返回 Db 错误）；
/// 片段本身不存在返回 AppError::NotFound。允许跨分类移动（category_id 可变）。
pub fn update_snippet(
    conn: &Connection,
    id: i64,
    category_id: i64,
    title: &str,
    content: &str,
    favorite: i64,
) -> Result<SnippetRow, AppError> {
    let exists: i64 = conn.query_row(
        "SELECT COUNT(*) FROM snippets WHERE id = ?1",
        params![id],
        |row| row.get(0),
    )?;
    if exists == 0 {
        return Err(AppError::NotFound {
            kind: "snippet",
            id,
        });
    }
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE snippets SET
           category_id = ?1, title = ?2, content = ?3,
           favorite = ?4, updated_at = ?5
         WHERE id = ?6",
        params![category_id, title, content, favorite, now, id],
    )?;
    Ok(conn.query_row(SELECT_SNIPPET_BY_ID, params![id], row_to_snippet)?)
}

/// 删除片段。不存在返回 NotFound。
pub fn delete_snippet(conn: &Connection, id: i64) -> Result<(), AppError> {
    let affected = conn.execute("DELETE FROM snippets WHERE id = ?1", params![id])?;
    if affected == 0 {
        return Err(AppError::NotFound {
            kind: "snippet",
            id,
        });
    }
    Ok(())
}

/// 标记片段被使用：used_count += 1，last_used_at = now。
/// updated_at 不变（使用计数不算内容更新）。不存在返回 NotFound。
pub fn mark_snippet_used(conn: &Connection, id: i64) -> Result<SnippetRow, AppError> {
    let now = chrono::Utc::now().to_rfc3339();
    let affected = conn.execute(
        "UPDATE snippets SET used_count = used_count + 1, last_used_at = ?1
         WHERE id = ?2",
        params![now, id],
    )?;
    if affected == 0 {
        return Err(AppError::NotFound {
            kind: "snippet",
            id,
        });
    }
    Ok(conn.query_row(SELECT_SNIPPET_BY_ID, params![id], row_to_snippet)?)
}
