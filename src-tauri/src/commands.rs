//! Tauri Commands - 前端通过 invoke 调用的后端命令
//! 每个命令都是前后端交互的接口，负责调用 db 模块并处理错误

use crate::db::{self, DbState, Note, Stats};
use crate::config::AppConfig;
use std::fs;
use std::io::Write;
use tauri::State;

/// 统一的错误转换：把各种错误转成前端可读的 String
type CmdResult<T> = Result<T, String>;

fn map_err<E: std::fmt::Display>(e: E) -> String {
    e.to_string()
}

// ========== 笔记查询 ==========

#[tauri::command]
pub fn list_notes(state: State<DbState>, filter: String) -> CmdResult<Vec<Note>> {
    let conn = state.0.lock().map_err(map_err)?;
    db::list_notes(&conn, &filter).map_err(map_err)
}

#[tauri::command]
pub fn search_notes(state: State<DbState>, keyword: String) -> CmdResult<Vec<Note>> {
    let conn = state.0.lock().map_err(map_err)?;
    db::search_notes(&conn, &keyword).map_err(map_err)
}

#[tauri::command]
pub fn get_note(state: State<DbState>, id: i64) -> CmdResult<Option<Note>> {
    let conn = state.0.lock().map_err(map_err)?;
    db::get_note(&conn, id).map_err(map_err)
}

// ========== 笔记增删改 ==========

#[tauri::command]
pub fn create_note(state: State<DbState>, title: String, content: String) -> CmdResult<i64> {
    let conn = state.0.lock().map_err(map_err)?;
    db::create_note(&conn, &title, &content).map_err(map_err)
}

#[tauri::command]
pub fn update_note(
    state: State<DbState>,
    id: i64,
    title: String,
    content: String,
) -> CmdResult<bool> {
    let conn = state.0.lock().map_err(map_err)?;
    db::update_note(&conn, id, &title, &content).map_err(map_err)
}

#[tauri::command]
pub fn delete_note(state: State<DbState>, id: i64) -> CmdResult<bool> {
    let conn = state.0.lock().map_err(map_err)?;
    db::delete_note(&conn, id).map_err(map_err)
}

#[tauri::command]
pub fn restore_note(state: State<DbState>, id: i64) -> CmdResult<bool> {
    let conn = state.0.lock().map_err(map_err)?;
    db::restore_note(&conn, id).map_err(map_err)
}

#[tauri::command]
pub fn permanent_delete(state: State<DbState>, id: i64) -> CmdResult<bool> {
    let conn = state.0.lock().map_err(map_err)?;
    db::permanent_delete(&conn, id).map_err(map_err)
}

#[tauri::command]
pub fn clear_trash(state: State<DbState>) -> CmdResult<i64> {
    let conn = state.0.lock().map_err(map_err)?;
    db::clear_trash(&conn).map_err(map_err)
}

// ========== 状态切换 ==========

#[tauri::command]
pub fn toggle_favorite(state: State<DbState>, id: i64) -> CmdResult<bool> {
    let conn = state.0.lock().map_err(map_err)?;
    db::toggle_favorite(&conn, id).map_err(map_err)
}

#[tauri::command]
pub fn toggle_pin(state: State<DbState>, id: i64) -> CmdResult<bool> {
    let conn = state.0.lock().map_err(map_err)?;
    db::toggle_pin(&conn, id).map_err(map_err)
}

// ========== 统计 ==========

#[tauri::command]
pub fn get_stats(state: State<DbState>) -> CmdResult<Stats> {
    let conn = state.0.lock().map_err(map_err)?;
    db::get_stats(&conn).map_err(map_err)
}

// ========== 导出 ==========

/// 导出笔记到指定路径，format: "txt" | "md"
#[tauri::command]
pub fn export_note(
    state: State<DbState>,
    id: i64,
    format: String,
    path: String,
) -> CmdResult<String> {
    let conn = state.0.lock().map_err(map_err)?;
    let note = db::get_note(&conn, id)
        .map_err(map_err)?
        .ok_or_else(|| "笔记不存在".to_string())?;

    let text = if format == "md" {
        format!("# {}\n\n{}", note.title, note.content)
    } else {
        let underline = "=".repeat(note.title.chars().count().max(1));
        format!("{}\n{}\n\n{}", note.title, underline, note.content)
    };

    let mut file = fs::File::create(&path).map_err(map_err)?;
    file.write_all(text.as_bytes()).map_err(map_err)?;
    Ok(path)
}

// ========== 配置读写 ==========

/// 读取用户配置
#[tauri::command]
pub fn load_config(app: tauri::AppHandle) -> CmdResult<AppConfig> {
    let path = crate::config::config_path(&app).map_err(map_err)?;
    Ok(AppConfig::load(&path))
}

/// 保存用户配置
#[tauri::command]
pub fn save_config(app: tauri::AppHandle, config: AppConfig) -> CmdResult<()> {
    let path = crate::config::config_path(&app).map_err(map_err)?;
    config.save(&path).map_err(map_err)
}
