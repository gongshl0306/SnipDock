// 导入 / 导出（备份）相关的 Tauri commands。
//
// 数据格式：单文件 JSON，schema 见下方 BackupFile：
//   {
//     "version": 1,
//     "exported_at": "<RFC3339>",
//     "categories": [ { name, description } ... ],
//     "snippets":   [ { category_name, title, content, favorite } ... ]
//   }
//
// 设计要点：
// - 不导出 id / 时间戳：这些都是导入端 DB 重新生成的本地标识。
//   片段通过 category_name 关联回分类。
// - 导入是合并语义，不清空原数据：
//     分类按 name 唯一；存在则复用 id，不存在则新建。
//     片段按 (匹配后 category_id, title, content) 三元组去重，
//       避免反复导入造成爆量。
// - 整个导入包在一个事务里，失败回滚。
// - 错误以 AppError::Db(message) 表达 IO / JSON / SQL 失败。

use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::error::AppError;

/// 导出文件根结构。version 用于将来格式演进；当前固定为 1。
#[derive(Debug, Serialize, Deserialize)]
pub struct BackupFile {
    pub version: u32,
    pub exported_at: String,
    pub categories: Vec<BackupCategory>,
    pub snippets: Vec<BackupSnippet>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BackupCategory {
    pub name: String,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BackupSnippet {
    /// 关联分类的 name；导入时根据该字段找/建分类。
    pub category_name: String,
    pub title: String,
    pub content: String,
    #[serde(default)]
    pub favorite: i64,
}

/// 导入完成后返回给前端的统计。
#[derive(Debug, Serialize)]
pub struct ImportResult {
    pub created_categories: usize,
    pub reused_categories: usize,
    pub created_snippets: usize,
    pub skipped_snippets: usize,
}

/// 导出统计。
#[derive(Debug, Serialize)]
pub struct ExportResult {
    pub path: String,
    pub categories: usize,
    pub snippets: usize,
}

/// 把当前数据库整体导出到 path。
#[tauri::command]
pub fn export_data(
    path: String,
    state: State<'_, Mutex<Connection>>,
) -> Result<ExportResult, AppError> {
    let conn = state.lock().expect("db mutex poisoned");

    // 取分类（按 sort_order 稳定输出，便于人肉 diff）。
    let mut cat_stmt = conn.prepare(
        "SELECT name, description FROM categories
         ORDER BY sort_order ASC, created_at ASC",
    )?;
    let categories: Vec<BackupCategory> = cat_stmt
        .query_map([], |row| {
            Ok(BackupCategory {
                name: row.get::<_, String>("name")?,
                description: row.get::<_, String>("description")?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    // 取片段，按 category 关联出 name。
    let mut snip_stmt = conn.prepare(
        "SELECT c.name AS category_name, s.title, s.content, s.favorite
         FROM snippets s
         INNER JOIN categories c ON c.id = s.category_id
         ORDER BY c.sort_order ASC, s.id ASC",
    )?;
    let snippets: Vec<BackupSnippet> = snip_stmt
        .query_map([], |row| {
            Ok(BackupSnippet {
                category_name: row.get::<_, String>("category_name")?,
                title: row.get::<_, String>("title")?,
                content: row.get::<_, String>("content")?,
                favorite: row.get::<_, i64>("favorite")?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    let cat_count = categories.len();
    let snip_count = snippets.len();
    let backup = BackupFile {
        version: 1,
        exported_at: chrono::Utc::now().to_rfc3339(),
        categories,
        snippets,
    };

    let json = serde_json::to_string_pretty(&backup)
        .map_err(|e| AppError::Db(format!("序列化失败：{e}")))?;

    // 创建父目录（如果用户在新位置另存）。
    let target = PathBuf::from(&path);
    if let Some(dir) = target.parent() {
        if !dir.as_os_str().is_empty() {
            fs::create_dir_all(dir)
                .map_err(|e| AppError::Db(format!("创建目录失败：{e}")))?;
        }
    }
    fs::write(&target, json).map_err(|e| AppError::Db(format!("写文件失败：{e}")))?;

    Ok(ExportResult {
        path,
        categories: cat_count,
        snippets: snip_count,
    })
}

/// 从 path 读取备份并合并到当前数据库。
#[tauri::command]
pub fn import_data(
    path: String,
    state: State<'_, Mutex<Connection>>,
) -> Result<ImportResult, AppError> {
    let raw = fs::read_to_string(&path)
        .map_err(|e| AppError::Db(format!("读文件失败：{e}")))?;
    let backup: BackupFile = serde_json::from_str(&raw)
        .map_err(|e| AppError::Db(format!("JSON 解析失败：{e}")))?;

    if backup.version != 1 {
        return Err(AppError::Db(format!(
            "不支持的备份版本：{}",
            backup.version
        )));
    }

    let mut conn = state.lock().expect("db mutex poisoned");
    let tx = conn
        .transaction()
        .map_err(|e| AppError::Db(format!("开启事务失败：{e}")))?;

    let mut created_categories = 0usize;
    let mut reused_categories = 0usize;
    let mut created_snippets = 0usize;
    let mut skipped_snippets = 0usize;

    let now = chrono::Utc::now().to_rfc3339();

    // 1) 处理分类：按 name 找/建，结果填到 name -> id 映射表。
    let mut name_to_id: std::collections::HashMap<String, i64> =
        std::collections::HashMap::new();

    for cat in &backup.categories {
        let trimmed = cat.name.trim();
        if trimmed.is_empty() {
            continue;
        }
        let existing: Option<i64> = tx
            .query_row(
                "SELECT id FROM categories WHERE name = ?1",
                params![trimmed],
                |r| r.get(0),
            )
            .ok();
        if let Some(id) = existing {
            name_to_id.insert(trimmed.to_string(), id);
            reused_categories += 1;
        } else {
            let max_order: i64 = tx.query_row(
                "SELECT COALESCE(MAX(sort_order), -1) FROM categories",
                [],
                |r| r.get(0),
            )?;
            tx.execute(
                "INSERT INTO categories (name, description, sort_order, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?4)",
                params![trimmed, cat.description, max_order + 1, now],
            )?;
            name_to_id.insert(trimmed.to_string(), tx.last_insert_rowid());
            created_categories += 1;
        }
    }

    // 2) 处理片段：找到目标分类 id（若 backup 里某片段引用的分类没在 categories 段出现，
    //    现尝试在已有库里按 name 找；都找不到就把它落到「默认」分类）。
    for snip in &backup.snippets {
        let title = snip.title.trim();
        let content = snip.content.trim();
        if title.is_empty() || content.is_empty() {
            skipped_snippets += 1;
            continue;
        }

        let cat_name = snip.category_name.trim();
        let category_id = if let Some(id) = name_to_id.get(cat_name) {
            *id
        } else if !cat_name.is_empty() {
            // 没在 backup.categories 里出现，但本地可能已有同名分类。
            let local: Option<i64> = tx
                .query_row(
                    "SELECT id FROM categories WHERE name = ?1",
                    params![cat_name],
                    |r| r.get(0),
                )
                .ok();
            match local {
                Some(id) => {
                    name_to_id.insert(cat_name.to_string(), id);
                    id
                }
                None => fallback_default_category(&tx, &now, &mut name_to_id)?,
            }
        } else {
            fallback_default_category(&tx, &now, &mut name_to_id)?
        };

        // 去重：(category_id, title, content) 已存在则跳过。
        let exists: i64 = tx.query_row(
            "SELECT COUNT(*) FROM snippets
             WHERE category_id = ?1 AND title = ?2 AND content = ?3",
            params![category_id, title, content],
            |r| r.get(0),
        )?;
        if exists > 0 {
            skipped_snippets += 1;
            continue;
        }

        let favorite = if snip.favorite != 0 { 1 } else { 0 };
        tx.execute(
            "INSERT INTO snippets
               (category_id, title, content,
                favorite, used_count, created_at, updated_at, last_used_at)
             VALUES (?1, ?2, ?3, ?4, 0, ?5, ?5, NULL)",
            params![category_id, title, content, favorite, now],
        )?;
        created_snippets += 1;
    }

    tx.commit()
        .map_err(|e| AppError::Db(format!("提交事务失败：{e}")))?;

    Ok(ImportResult {
        created_categories,
        reused_categories,
        created_snippets,
        skipped_snippets,
    })
}

/// 取「默认」分类的 id；如果连「默认」都不存在（用户删了），就新建一个。
/// 用于导入时找不到目标分类的兜底。
fn fallback_default_category(
    tx: &rusqlite::Transaction<'_>,
    now: &str,
    cache: &mut std::collections::HashMap<String, i64>,
) -> Result<i64, AppError> {
    const NAME: &str = "默认";
    if let Some(id) = cache.get(NAME) {
        return Ok(*id);
    }
    let existing: Option<i64> = tx
        .query_row(
            "SELECT id FROM categories WHERE name = ?1",
            params![NAME],
            |r| r.get(0),
        )
        .ok();
    let id = if let Some(id) = existing {
        id
    } else {
        let max_order: i64 = tx.query_row(
            "SELECT COALESCE(MAX(sort_order), -1) FROM categories",
            [],
            |r| r.get(0),
        )?;
        tx.execute(
            "INSERT INTO categories (name, description, sort_order, created_at, updated_at)
             VALUES (?1, '', ?2, ?3, ?3)",
            params![NAME, max_order + 1, now],
        )?;
        tx.last_insert_rowid()
    };
    cache.insert(NAME.to_string(), id);
    Ok(id)
}
