// 类型定义 - 前后端共享的数据结构（与 Rust 端 serde 结构对应）

/** 笔记 */
export interface Note {
  id: number;
  title: string;
  content: string;
  is_favorite: boolean;
  is_deleted: boolean;
  is_pinned: boolean;
  created_at: string;
  updated_at: string;
}

/** 统计信息 */
export interface Stats {
  count: number; // 笔记总数
  words: number; // 总字数
}

/** 应用配置（与 Rust AppConfig 对应） */
export interface AppConfig {
  theme: "light" | "dark";
  auto_start: boolean;
  sidebar_active: FilterType;
  last_opened_note_id: number | null;
  window_width: number;
  window_height: number;
  /** 窗口不透明度 (0.3 ~ 1.0) */
  window_opacity: number;
}

/** 列表过滤类型 */
export type FilterType = "all" | "favorite" | "trash";

/** 导出格式 */
export type ExportFormat = "txt" | "md";
