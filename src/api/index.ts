// API 层 - 封装所有对 Rust 后端的 invoke 调用
// 前端其他部分只通过这里访问后端，便于统一管理

import { invoke } from "@tauri-apps/api/core";
import { save } from "@tauri-apps/plugin-dialog";
import type { Note, Stats, AppConfig, FilterType, ExportFormat } from "./types";

// ========== 笔记查询 ==========

/** 列出笔记 */
export function listNotes(filter: FilterType): Promise<Note[]> {
  return invoke("list_notes", { filter });
}

/** 搜索笔记 */
export function searchNotes(keyword: string): Promise<Note[]> {
  return invoke("search_notes", { keyword });
}

/** 获取单条笔记 */
export function getNote(id: number): Promise<Note | null> {
  return invoke("get_note", { id });
}

// ========== 笔记增删改 ==========

/** 新建笔记，返回新 id */
export function createNote(title = "无标题", content = ""): Promise<number> {
  return invoke("create_note", { title, content });
}

/** 更新笔记 */
export function updateNote(id: number, title: string, content: string): Promise<boolean> {
  return invoke("update_note", { id, title, content });
}

/** 删除笔记（移入回收站） */
export function deleteNote(id: number): Promise<boolean> {
  return invoke("delete_note", { id });
}

/** 恢复笔记 */
export function restoreNote(id: number): Promise<boolean> {
  return invoke("restore_note", { id });
}

/** 永久删除 */
export function permanentDelete(id: number): Promise<boolean> {
  return invoke("permanent_delete", { id });
}

/** 清空回收站，返回删除数量 */
export function clearTrash(): Promise<number> {
  return invoke("clear_trash");
}

// ========== 状态切换 ==========

/** 切换收藏，返回新状态 */
export function toggleFavorite(id: number): Promise<boolean> {
  return invoke("toggle_favorite", { id });
}

/** 切换置顶，返回新状态 */
export function togglePin(id: number): Promise<boolean> {
  return invoke("toggle_pin", { id });
}

// ========== 统计 ==========

/** 获取统计信息 */
export function getStats(): Promise<Stats> {
  return invoke("get_stats");
}

// ========== 导出 ==========

/**
 * 导出笔记：先弹出保存对话框选路径，再让后端写文件
 * 返回导出的文件路径，取消则返回 null
 */
export async function exportNote(
  id: number,
  title: string,
  format: ExportFormat,
): Promise<string | null> {
  // 清理文件名中的非法字符
  const safeTitle = title.replace(/[/\\:*?"<>|]/g, "_") || "无标题";
  const path = await save({
    defaultPath: `${safeTitle}.${format}`,
    filters: [
      {
        name: format === "md" ? "Markdown 文件" : "文本文件",
        extensions: [format],
      },
    ],
  });
  if (!path) return null;
  return invoke("export_note", { id, format, path });
}

// ========== 配置读写 ==========

/** 读取配置 */
export function loadConfig(): Promise<AppConfig> {
  return invoke("load_config");
}

/** 保存配置 */
export function saveConfig(config: AppConfig): Promise<void> {
  return invoke("save_config", { config });
}
