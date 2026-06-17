// 数据库子系统。
//
// 职责：
//   1. 打开 SQLite 文件
//   2. 启用外键约束
//   3. 建表 + 建索引（按 target.md §5.1 / §5.2 / §5.4 字面量）
//   4. 首次启动插入默认分类
//
// v1 不引入 schema 版本号。表结构未来变更时再加 `user_version` PRAGMA。

pub mod migration;

use rusqlite::Connection;
use std::path::Path;

pub fn open_and_migrate(db_path: &Path) -> Result<Connection, rusqlite::Error> {
    let conn = Connection::open(db_path)?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    migration::run(&conn)?;
    Ok(conn)
}
