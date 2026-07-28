// Pinia Store - 笔记全局状态管理
// 承担原 Python 版 controller 的职责：列表加载、选中、自动保存、搜索、导出等

import { defineStore } from "pinia";
import { ref, computed } from "vue";
import * as api from "../api";
import type { Note, Stats, FilterType, ExportFormat } from "../api/types";

export const useNotesStore = defineStore("notes", () => {
  // ========== 状态 ==========
  const notes = ref<Note[]>([]);              // 当前列表
  const currentNote = ref<Note | null>(null); // 当前编辑的笔记
  const currentFilter = ref<FilterType>("all");
  const searchKeyword = ref("");
  const stats = ref<Stats>({ count: 0, words: 0 });
  const saveStatus = ref<"" | "unsaved" | "saved">(""); // 保存状态指示

  // 自动保存定时器（1 秒防抖）
  let autoSaveTimer: number | null = null;

  // ========== 计算属性 ==========
  const isTrash = computed(() => currentFilter.value === "trash");
  const currentNoteId = computed(() => currentNote.value?.id ?? null);

  // ========== 列表加载 ==========

  /** 刷新当前视图的笔记列表 */
  async function refreshList(): Promise<void> {
    if (searchKeyword.value.trim()) {
      notes.value = await api.searchNotes(searchKeyword.value.trim());
    } else {
      notes.value = await api.listNotes(currentFilter.value);
    }
    await refreshStats();
  }

  /** 切换过滤视图（全部/收藏/回收站） */
  async function switchFilter(filter: FilterType): Promise<void> {
    await saveCurrentIfNeeded();
    currentFilter.value = filter;
    searchKeyword.value = "";
    await refreshList();
  }

  /** 搜索（实时） */
  async function search(keyword: string): Promise<void> {
    searchKeyword.value = keyword;
    if (keyword.trim()) {
      notes.value = await api.searchNotes(keyword.trim());
    } else {
      notes.value = await api.listNotes(currentFilter.value);
    }
  }

  /** 刷新统计 */
  async function refreshStats(): Promise<void> {
    stats.value = await api.getStats();
  }

  // ========== 笔记选中 / 编辑 ==========

  /** 选中并加载笔记到编辑区 */
  async function selectNote(id: number): Promise<void> {
    if (currentNote.value?.id === id) return;
    await saveCurrentIfNeeded();
    const note = await api.getNote(id);
    if (note) {
      currentNote.value = note;
      saveStatus.value = "";
    }
  }

  /** 取消选中 */
  async function clearSelection(): Promise<void> {
    await saveCurrentIfNeeded();
    currentNote.value = null;
    saveStatus.value = "";
  }

  /** 新建笔记 */
  async function createNote(): Promise<number> {
    await saveCurrentIfNeeded();
    // 新建笔记须切回"全部"视图
    currentFilter.value = "all";
    searchKeyword.value = "";
    const id = await api.createNote();
    await refreshList();
    const note = await api.getNote(id);
    if (note) currentNote.value = note;
    saveStatus.value = "";
    return id;
  }

  // ========== 自动保存 ==========

  /** 编辑器内容变化：标记未保存并重启防抖定时器 */
  function onContentChanged(title: string, content: string): void {
    if (!currentNote.value) return;
    currentNote.value.title = title;
    currentNote.value.content = content;
    saveStatus.value = "unsaved";

    if (autoSaveTimer !== null) clearTimeout(autoSaveTimer);
    autoSaveTimer = window.setTimeout(() => {
      void doSave();
    }, 1000); // 停止输入 1 秒后自动保存
  }

  /** 执行保存 */
  async function doSave(): Promise<void> {
    if (!currentNote.value) return;
    const { id, title, content } = currentNote.value;
    await api.updateNote(id, title.trim() || "无标题", content);
    saveStatus.value = "saved";
    // 刷新列表（更新预览/时间），但不打断搜索状态
    if (!searchKeyword.value.trim()) {
      const savedId = id;
      notes.value = await api.listNotes(currentFilter.value);
      // 保持当前选中对象引用
      const found = notes.value.find((n) => n.id === savedId);
      if (found && currentNote.value) {
        currentNote.value.updated_at = found.updated_at;
      }
    }
    await refreshStats();
  }

  /** 手动保存（Ctrl+S） */
  async function manualSave(): Promise<void> {
    if (autoSaveTimer !== null) clearTimeout(autoSaveTimer);
    await doSave();
  }

  /** 切换/关闭前若有未保存内容则先保存 */
  async function saveCurrentIfNeeded(): Promise<void> {
    if (saveStatus.value === "unsaved" && currentNote.value) {
      if (autoSaveTimer !== null) clearTimeout(autoSaveTimer);
      await doSave();
    }
  }

  // ========== 删除 / 恢复 ==========

  async function deleteNote(id: number): Promise<void> {
    if (currentNote.value?.id === id) {
      currentNote.value = null;
    }
    await api.deleteNote(id);
    await refreshList();
  }

  async function restoreNote(id: number): Promise<void> {
    await api.restoreNote(id);
    await refreshList();
  }

  async function permanentDelete(id: number): Promise<void> {
    if (currentNote.value?.id === id) {
      currentNote.value = null;
    }
    await api.permanentDelete(id);
    await refreshList();
  }

  async function clearTrash(): Promise<number> {
    const count = await api.clearTrash();
    await refreshList();
    return count;
  }

  // ========== 收藏 / 置顶 ==========

  async function toggleFavorite(id: number): Promise<boolean> {
    const isFav = await api.toggleFavorite(id);
    if (currentNote.value?.id === id) {
      currentNote.value.is_favorite = isFav;
    }
    await refreshList();
    return isFav;
  }

  async function togglePin(id: number): Promise<void> {
    await api.togglePin(id);
    await refreshList();
  }

  // ========== 导出 ==========

  async function exportNote(id: number, title: string, format: ExportFormat): Promise<string | null> {
    return api.exportNote(id, title, format);
  }

  return {
    // 状态
    notes,
    currentNote,
    currentFilter,
    searchKeyword,
    stats,
    saveStatus,
    // 计算属性
    isTrash,
    currentNoteId,
    // 方法
    refreshList,
    switchFilter,
    search,
    refreshStats,
    selectNote,
    clearSelection,
    createNote,
    onContentChanged,
    manualSave,
    saveCurrentIfNeeded,
    deleteNote,
    restoreNote,
    permanentDelete,
    clearTrash,
    toggleFavorite,
    togglePin,
    exportNote,
  };
});
