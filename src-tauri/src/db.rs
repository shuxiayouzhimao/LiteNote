//! 数据库模块 - SQLite 操作
//! 负责建表、笔记的增删改查、搜索、收藏/置顶、统计等所有数据操作
//! 数据库文件存储于 %APPDATA%/LiteNote/data.db

use rusqlite::{Connection, Result as SqlResult};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;

/// 笔记数据结构（前后端共享，通过 serde 序列化为 JSON）
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Note {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub is_favorite: bool,
    pub is_deleted: bool,
    pub is_pinned: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// 统计信息
#[derive(Debug, Serialize, Deserialize)]
pub struct Stats {
    pub count: i64,  // 笔记总数（不含回收站）
    pub words: i64,  // 总字数
}

/// 数据库连接的全局状态（用 Mutex 保证线程安全）
pub struct DbState(pub Mutex<Connection>);

/// 打开（或创建）数据库连接并初始化表结构
pub fn init_db(db_path: &PathBuf) -> SqlResult<Connection> {
    let conn = Connection::open(db_path)?;

    // 提升并发/性能
    conn.execute_batch(
        "PRAGMA journal_mode = WAL;
         PRAGMA foreign_keys = ON;",
    )?;

    // 建表 + 索引
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS notes (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            title       TEXT    NOT NULL DEFAULT '无标题',
            content     TEXT    DEFAULT '',
            is_favorite INTEGER DEFAULT 0,
            is_deleted  INTEGER DEFAULT 0,
            is_pinned   INTEGER DEFAULT 0,
            created_at  TEXT    DEFAULT (datetime('now','localtime')),
            updated_at  TEXT    DEFAULT (datetime('now','localtime'))
        );
        CREATE INDEX IF NOT EXISTS idx_notes_updated  ON notes(updated_at DESC);
        CREATE INDEX IF NOT EXISTS idx_notes_deleted  ON notes(is_deleted);
        CREATE INDEX IF NOT EXISTS idx_notes_favorite ON notes(is_favorite, is_deleted);",
    )?;

    Ok(conn)
}

/// 当前本地时间字符串（YYYY-MM-DD HH:MM:SS）
fn now_string() -> String {
    chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

/// 从数据库行映射为 Note 结构
fn row_to_note(row: &rusqlite::Row) -> SqlResult<Note> {
    Ok(Note {
        id: row.get("id")?,
        title: row.get("title")?,
        content: row.get("content")?,
        is_favorite: row.get::<_, i64>("is_favorite")? != 0,
        is_deleted: row.get::<_, i64>("is_deleted")? != 0,
        is_pinned: row.get::<_, i64>("is_pinned")? != 0,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

// ========== 查询 ==========

/// 列出笔记，filter: "all" | "favorite" | "trash"
pub fn list_notes(conn: &Connection, filter: &str) -> SqlResult<Vec<Note>> {
    let sql = match filter {
        "favorite" => {
            "SELECT * FROM notes WHERE is_favorite = 1 AND is_deleted = 0
             ORDER BY is_pinned DESC, updated_at DESC"
        }
        "trash" => {
            "SELECT * FROM notes WHERE is_deleted = 1
             ORDER BY updated_at DESC"
        }
        _ => {
            "SELECT * FROM notes WHERE is_deleted = 0
             ORDER BY is_pinned DESC, updated_at DESC"
        }
    };
    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map([], row_to_note)?;
    rows.collect()
}

/// 搜索笔记（标题 + 内容）
pub fn search_notes(conn: &Connection, keyword: &str) -> SqlResult<Vec<Note>> {
    let pattern = format!("%{}%", keyword);
    let mut stmt = conn.prepare(
        "SELECT * FROM notes
         WHERE is_deleted = 0 AND (title LIKE ?1 OR content LIKE ?1)
         ORDER BY is_pinned DESC, updated_at DESC",
    )?;
    let rows = stmt.query_map([&pattern], row_to_note)?;
    rows.collect()
}

/// 获取单条笔记
pub fn get_note(conn: &Connection, id: i64) -> SqlResult<Option<Note>> {
    let mut stmt = conn.prepare("SELECT * FROM notes WHERE id = ?1")?;
    let mut rows = stmt.query_map([id], row_to_note)?;
    match rows.next() {
        Some(note) => Ok(Some(note?)),
        None => Ok(None),
    }
}

// ========== 增删改 ==========

/// 新建笔记，返回新 id
pub fn create_note(conn: &Connection, title: &str, content: &str) -> SqlResult<i64> {
    let now = now_string();
    conn.execute(
        "INSERT INTO notes (title, content, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?3)",
        rusqlite::params![title, content, now],
    )?;
    Ok(conn.last_insert_rowid())
}

/// 更新笔记标题和内容，刷新 updated_at
pub fn update_note(conn: &Connection, id: i64, title: &str, content: &str) -> SqlResult<bool> {
    let now = now_string();
    let affected = conn.execute(
        "UPDATE notes SET title = ?1, content = ?2, updated_at = ?3 WHERE id = ?4",
        rusqlite::params![title, content, now, id],
    )?;
    Ok(affected > 0)
}

/// 软删除（移入回收站，同时取消收藏）
pub fn delete_note(conn: &Connection, id: i64) -> SqlResult<bool> {
    let affected = conn.execute(
        "UPDATE notes SET is_deleted = 1, is_favorite = 0 WHERE id = ?1",
        [id],
    )?;
    Ok(affected > 0)
}

/// 从回收站恢复
pub fn restore_note(conn: &Connection, id: i64) -> SqlResult<bool> {
    let affected = conn.execute("UPDATE notes SET is_deleted = 0 WHERE id = ?1", [id])?;
    Ok(affected > 0)
}

/// 永久删除
pub fn permanent_delete(conn: &Connection, id: i64) -> SqlResult<bool> {
    let affected = conn.execute("DELETE FROM notes WHERE id = ?1", [id])?;
    Ok(affected > 0)
}

/// 清空回收站，返回删除数量
pub fn clear_trash(conn: &Connection) -> SqlResult<i64> {
    let affected = conn.execute("DELETE FROM notes WHERE is_deleted = 1", [])?;
    Ok(affected as i64)
}

// ========== 状态切换 ==========

/// 切换收藏状态，返回新状态
pub fn toggle_favorite(conn: &Connection, id: i64) -> SqlResult<bool> {
    conn.execute(
        "UPDATE notes SET is_favorite = CASE WHEN is_favorite = 1 THEN 0 ELSE 1 END WHERE id = ?1",
        [id],
    )?;
    let state: i64 = conn.query_row("SELECT is_favorite FROM notes WHERE id = ?1", [id], |r| r.get(0))?;
    Ok(state != 0)
}

/// 切换置顶状态，返回新状态
pub fn toggle_pin(conn: &Connection, id: i64) -> SqlResult<bool> {
    conn.execute(
        "UPDATE notes SET is_pinned = CASE WHEN is_pinned = 1 THEN 0 ELSE 1 END WHERE id = ?1",
        [id],
    )?;
    let state: i64 = conn.query_row("SELECT is_pinned FROM notes WHERE id = ?1", [id], |r| r.get(0))?;
    Ok(state != 0)
}

// ========== 统计 ==========

/// 获取笔记统计（总数 + 字数，均不含回收站）
pub fn get_stats(conn: &Connection) -> SqlResult<Stats> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM notes WHERE is_deleted = 0",
        [],
        |r| r.get(0),
    )?;

    // 统计所有笔记内容字数（去掉空格和换行）
    let mut stmt = conn.prepare("SELECT content FROM notes WHERE is_deleted = 0")?;
    let contents = stmt.query_map([], |r| r.get::<_, String>(0))?;
    let mut words: i64 = 0;
    for c in contents {
        let text = c?;
        words += text.chars().filter(|ch| !ch.is_whitespace()).count() as i64;
    }

    Ok(Stats { count, words })
}
