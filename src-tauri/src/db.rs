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

// ========== 单元测试 ==========
#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    /// 每个测试使用独立的内存数据库
    fn setup() -> Connection {
        let conn = Connection::open_in_memory().expect("创建内存数据库失败");
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
        )
        .expect("建表失败");
        conn
    }

    #[test]
    fn test_create_note() {
        let conn = setup();
        let id = create_note(&conn, "测试标题", "测试内容").expect("创建笔记失败");
        assert!(id > 0, "新建笔记应返回正数 id");

        let note = get_note(&conn, id).expect("查询失败").expect("笔记应存在");
        assert_eq!(note.title, "测试标题");
        assert_eq!(note.content, "测试内容");
        assert!(!note.is_favorite);
        assert!(!note.is_deleted);
        assert!(!note.is_pinned);
    }

    #[test]
    fn test_create_note_default_empty() {
        let conn = setup();
        let id = create_note(&conn, "无标题", "").expect("创建失败");
        let note = get_note(&conn, id).expect("查询失败").expect("笔记应存在");
        assert_eq!(note.title, "无标题");
        assert_eq!(note.content, "");
    }

    #[test]
    fn test_update_note() {
        let conn = setup();
        let id = create_note(&conn, "原始标题", "原始内容").unwrap();

        let ok = update_note(&conn, id, "新标题", "新内容").unwrap();
        assert!(ok);

        let note = get_note(&conn, id).unwrap().unwrap();
        assert_eq!(note.title, "新标题");
        assert_eq!(note.content, "新内容");
    }

    #[test]
    fn test_update_nonexistent() {
        let conn = setup();
        let ok = update_note(&conn, 999, "标题", "内容").unwrap();
        assert!(!ok, "更新不存在的笔记应返回 false");
    }

    #[test]
    fn test_delete_and_restore() {
        let conn = setup();
        let id = create_note(&conn, "要删的笔记", "").unwrap();

        // 软删除
        let ok = delete_note(&conn, id).unwrap();
        assert!(ok);

        // 不应该出现在 'all' 列表中
        let all = list_notes(&conn, "all").unwrap();
        assert!(all.iter().all(|n| n.id != id));

        // 应该出现在 'trash' 列表中
        let trash = list_notes(&conn, "trash").unwrap();
        assert!(trash.iter().any(|n| n.id == id));

        // 恢复
        let ok = restore_note(&conn, id).unwrap();
        assert!(ok);

        let all = list_notes(&conn, "all").unwrap();
        assert!(all.iter().any(|n| n.id == id));
    }

    #[test]
    fn test_permanent_delete() {
        let conn = setup();
        let id = create_note(&conn, "永删笔记", "").unwrap();

        let ok = permanent_delete(&conn, id).unwrap();
        assert!(ok);

        let note = get_note(&conn, id).unwrap();
        assert!(note.is_none(), "永久删除后笔记应不存在");

        // 删除不存在的不应报错
        let ok = permanent_delete(&conn, 999).unwrap();
        assert!(!ok);
    }

    #[test]
    fn test_list_notes_filter() {
        let conn = setup();
        create_note(&conn, "普通笔记", "").unwrap();
        let id2 = create_note(&conn, "收藏笔记", "").unwrap();
        toggle_favorite(&conn, id2).unwrap();

        let all = list_notes(&conn, "all").unwrap();
        assert_eq!(all.len(), 2);

        let favs = list_notes(&conn, "favorite").unwrap();
        assert_eq!(favs.len(), 1);
        assert_eq!(favs[0].id, id2);

        let trash = list_notes(&conn, "trash").unwrap();
        assert!(trash.is_empty());
    }

    #[test]
    fn test_list_trash() {
        let conn = setup();
        let id1 = create_note(&conn, "笔记1", "").unwrap();
        create_note(&conn, "笔记2", "").unwrap();
        delete_note(&conn, id1).unwrap();

        let trash = list_notes(&conn, "trash").unwrap();
        assert_eq!(trash.len(), 1);
        assert_eq!(trash[0].id, id1);
    }

    #[test]
    fn test_search_notes() {
        let conn = setup();
        create_note(&conn, "Rust 学习", "所有权与借用").unwrap();
        create_note(&conn, "Vue 入门", "组件与响应式").unwrap();
        create_note(&conn, "Tauri 配置", "Rust 后端命令").unwrap();

        // 搜标题
        let results = search_notes(&conn, "Rust").unwrap();
        assert_eq!(results.len(), 2); // "Rust 学习" + "Tauri 配置"（内容含 Rust）

        // 搜内容
        let results = search_notes(&conn, "组件").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Vue 入门");

        // 无匹配
        let results = search_notes(&conn, "Python").unwrap();
        assert!(results.is_empty());

        // 搜索不搜回收站中的笔记
        let id = create_note(&conn, "已删的Rust笔记", "").unwrap();
        delete_note(&conn, id).unwrap();
        let results = search_notes(&conn, "已删").unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_toggle_favorite() {
        let conn = setup();
        let id = create_note(&conn, "笔记", "").unwrap();

        let fav = toggle_favorite(&conn, id).unwrap();
        assert!(fav);

        let note = get_note(&conn, id).unwrap().unwrap();
        assert!(note.is_favorite);

        let fav = toggle_favorite(&conn, id).unwrap();
        assert!(!fav);
    }

    #[test]
    fn test_toggle_pin() {
        let conn = setup();
        let id = create_note(&conn, "笔记", "").unwrap();

        let pinned = toggle_pin(&conn, id).unwrap();
        assert!(pinned);

        let pinned = toggle_pin(&conn, id).unwrap();
        assert!(!pinned);
    }

    #[test]
    fn test_pinned_first_in_list() {
        let conn = setup();
        let id1 = create_note(&conn, "普通", "").unwrap();
        let id2 = create_note(&conn, "置顶", "").unwrap();
        toggle_pin(&conn, id2).unwrap();

        let all = list_notes(&conn, "all").unwrap();
        assert_eq!(all.len(), 2);
        assert_eq!(all[0].id, id2, "置顶笔记应排第一");
        assert_eq!(all[1].id, id1);
    }

    #[test]
    fn test_clear_trash() {
        let conn = setup();
        let id1 = create_note(&conn, "a", "").unwrap();
        let id2 = create_note(&conn, "b", "").unwrap();
        delete_note(&conn, id1).unwrap();
        delete_note(&conn, id2).unwrap();

        let count = clear_trash(&conn).unwrap();
        assert_eq!(count, 2);

        let trash = list_notes(&conn, "trash").unwrap();
        assert!(trash.is_empty());
    }

    #[test]
    fn test_get_stats() {
        let conn = setup();
        create_note(&conn, "笔记1", "Hello World").unwrap();      // 10 chars (no whitespace)
        create_note(&conn, "笔记2", "Rust is great").unwrap();    // 12 chars (space excluded)
        create_note(&conn, "已删", "garbage").unwrap();
        // 软删除第三个
        delete_note(&conn, 3).unwrap();

        let stats = get_stats(&conn).unwrap();
        assert_eq!(stats.count, 2, "统计应排除回收站");
        // "HelloWorld" = 10, "Rustisgreat" = 11 → 21
        assert_eq!(stats.words, 21);
    }

    #[test]
    fn test_get_note_nonexistent() {
        let conn = setup();
        let result = get_note(&conn, 999).unwrap();
        assert!(result.is_none());
    }
}
