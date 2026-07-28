/**
 * notes store 单元测试
 * 测试状态管理逻辑，mock 所有 Tauri API 调用
 */
import { describe, it, expect, vi, beforeEach } from "vitest";
import { setActivePinia, createPinia } from "pinia";

// ---------- Mock API 层 ----------
// 使用 vi.mock 工厂函数（会被 hoist，不能引用外部变量）
vi.mock("../../api", () => ({
  listNotes: vi.fn(),
  searchNotes: vi.fn(),
  getNote: vi.fn(),
  createNote: vi.fn(),
  updateNote: vi.fn(),
  deleteNote: vi.fn(),
  restoreNote: vi.fn(),
  permanentDelete: vi.fn(),
  clearTrash: vi.fn(),
  toggleFavorite: vi.fn(),
  togglePin: vi.fn(),
  getStats: vi.fn(),
  exportNote: vi.fn(),
  loadConfig: vi.fn(),
  saveConfig: vi.fn(),
}));

// 获取 mock 模块引用（在 vi.mock 之后 import）
import * as mockApi from "../../api";

import { useNotesStore } from "../notes";

// 测试用笔记工厂函数
function makeNote(overrides: Partial<ReturnType<typeof useNotesStore>["notes"] extends Array<infer T> ? T : never> = {}) {
  return {
    id: 1,
    title: "测试笔记",
    content: "Hello World",
    is_favorite: false,
    is_deleted: false,
    is_pinned: false,
    created_at: "2026-01-01 12:00:00",
    updated_at: "2026-01-01 12:00:00",
    ...overrides,
  };
}

describe("useNotesStore", () => {
  beforeEach(() => {
    // 每次测试创建全新 Pinia 实例 + 重置所有 mock
    setActivePinia(createPinia());
    vi.clearAllMocks();
    vi.useFakeTimers(); // 控制自动保存定时器
  });

  // ========== 列表加载 ==========

  describe("refreshList", () => {
    it("应该在无搜索关键词时按 filter 加载列表", async () => {
      const store = useNotesStore();
      const notes = [makeNote(), makeNote({ id: 2, title: "第二篇" })];
      mockApi.listNotes.mockResolvedValue(notes);
      mockApi.getStats.mockResolvedValue({ count: 2, words: 21 });

      await store.refreshList();

      expect(mockApi.listNotes).toHaveBeenCalledWith("all");
      expect(store.notes).toEqual(notes);
      expect(store.stats).toEqual({ count: 2, words: 21 });
    });

    it("应该在有搜索关键词时调用搜索 API", async () => {
      const store = useNotesStore();
      store.searchKeyword = "Hello";
      mockApi.searchNotes.mockResolvedValue([makeNote()]);
      mockApi.getStats.mockResolvedValue({ count: 1, words: 11 });

      await store.refreshList();

      expect(mockApi.searchNotes).toHaveBeenCalledWith("Hello");
      expect(mockApi.listNotes).not.toHaveBeenCalled();
    });
  });

  describe("switchFilter", () => {
    it("应该切换过滤器并刷新列表", async () => {
      const store = useNotesStore();
      mockApi.listNotes.mockResolvedValue([]);
      mockApi.getStats.mockResolvedValue({ count: 0, words: 0 });

      await store.switchFilter("favorite");

      expect(store.currentFilter).toBe("favorite");
      expect(store.searchKeyword).toBe("");
      expect(mockApi.listNotes).toHaveBeenCalledWith("favorite");
    });

    it("切换前应保存当前笔记", async () => {
      const store = useNotesStore();
      const note = makeNote();
      store.currentNote = note;
      store.saveStatus = "unsaved";
      mockApi.updateNote.mockResolvedValue(true);
      mockApi.listNotes.mockResolvedValue([note]);
      mockApi.getStats.mockResolvedValue({ count: 1, words: 5 });

      await store.switchFilter("trash");

      expect(mockApi.updateNote).toHaveBeenCalledWith(note.id, note.title, note.content);
    });
  });

  describe("search", () => {
    it("应该按关键词搜索", async () => {
      const store = useNotesStore();
      mockApi.searchNotes.mockResolvedValue([makeNote()]);

      await store.search("测试");

      expect(store.searchKeyword).toBe("测试");
      expect(mockApi.searchNotes).toHaveBeenCalledWith("测试");
    });

    it("空关键词应切回列表", async () => {
      const store = useNotesStore();
      mockApi.listNotes.mockResolvedValue([makeNote()]);

      await store.search("");

      expect(mockApi.listNotes).toHaveBeenCalledWith("all");
    });
  });

  // ========== 笔记选中 ==========

  describe("selectNote", () => {
    it("应该加载笔记到 currentNote", async () => {
      const store = useNotesStore();
      const note = makeNote();
      mockApi.getNote.mockResolvedValue(note);

      await store.selectNote(1);

      expect(mockApi.getNote).toHaveBeenCalledWith(1);
      expect(store.currentNote).toEqual(note);
      expect(store.saveStatus).toBe("");
    });

    it("相同 id 应跳过加载", async () => {
      const store = useNotesStore();
      const note = makeNote();
      store.currentNote = note;

      await store.selectNote(1);

      expect(mockApi.getNote).not.toHaveBeenCalled();
    });

    it("选中前应保存上一个笔记", async () => {
      const store = useNotesStore();
      const prev = makeNote({ id: 1, title: "旧笔记" });
      store.currentNote = prev;
      store.saveStatus = "unsaved";
      mockApi.getNote.mockResolvedValue(makeNote({ id: 2 }));
      mockApi.updateNote.mockResolvedValue(true);

      await store.selectNote(2);

      expect(mockApi.updateNote).toHaveBeenCalledWith(1, "旧笔记", "Hello World");
    });
  });

  describe("createNote", () => {
    it("应该创建笔记并加载到编辑区", async () => {
      const store = useNotesStore();
      const note = makeNote({ id: 99, title: "无标题", content: "" });
      mockApi.createNote.mockResolvedValue(99);
      mockApi.listNotes.mockResolvedValue([note]);
      mockApi.getNote.mockResolvedValue(note);
      mockApi.getStats.mockResolvedValue({ count: 1, words: 0 });

      const id = await store.createNote();

      expect(id).toBe(99);
      expect(store.currentFilter).toBe("all");
      expect(store.currentNote).toEqual(note);
    });
  });

  // ========== 自动保存 ==========

  describe("onContentChanged + autoSave", () => {
    it("标记未保存并启动 1 秒防抖定时器", async () => {
      const store = useNotesStore();
      store.currentNote = makeNote();
      mockApi.updateNote.mockResolvedValue(true);
      mockApi.listNotes.mockResolvedValue([makeNote({ title: "新标题" })]);
      mockApi.getStats.mockResolvedValue({ count: 1, words: 6 });

      store.onContentChanged("新标题", "新内容");

      expect(store.saveStatus).toBe("unsaved");
      expect(store.currentNote!.title).toBe("新标题");
      expect(store.currentNote!.content).toBe("新内容");

      // 此时不应保存
      expect(mockApi.updateNote).not.toHaveBeenCalled();

      // 推进 1 秒 → 应触发保存
      await vi.advanceTimersByTimeAsync(1000);

      expect(mockApi.updateNote).toHaveBeenCalledWith(1, "新标题", "新内容");
      expect(store.saveStatus).toBe("saved");
    });

    it("连续输入应重置定时器（防抖）", async () => {
      const store = useNotesStore();
      store.currentNote = makeNote();
      mockApi.updateNote.mockResolvedValue(true);
      mockApi.listNotes.mockResolvedValue([]);
      mockApi.getStats.mockResolvedValue({ count: 0, words: 0 });

      store.onContentChanged("A", "a");
      await vi.advanceTimersByTimeAsync(500);
      store.onContentChanged("AB", "ab");
      await vi.advanceTimersByTimeAsync(500);

      // 500ms 后不应保存（被重置了）
      expect(mockApi.updateNote).not.toHaveBeenCalled();

      // 再等 500ms → 触发
      await vi.advanceTimersByTimeAsync(500);
      expect(mockApi.updateNote).toHaveBeenCalledTimes(1);
    });
  });

  describe("manualSave", () => {
    it("应立即保存，清除定时器", async () => {
      const store = useNotesStore();
      store.currentNote = makeNote();
      mockApi.updateNote.mockResolvedValue(true);
      mockApi.listNotes.mockResolvedValue([makeNote()]);
      mockApi.getStats.mockResolvedValue({ count: 1, words: 5 });

      store.onContentChanged("改", "变");
      await store.manualSave();

      expect(mockApi.updateNote).toHaveBeenCalledTimes(1);
      expect(store.saveStatus).toBe("saved");
    });
  });

  // ========== 删除 / 恢复 ==========

  describe("deleteNote", () => {
    it("软删除后应从 currentNote 清除并刷新列表", async () => {
      const store = useNotesStore();
      store.currentNote = makeNote();
      mockApi.deleteNote.mockResolvedValue(true);
      mockApi.listNotes.mockResolvedValue([]);
      mockApi.getStats.mockResolvedValue({ count: 0, words: 0 });

      await store.deleteNote(1);

      expect(store.currentNote).toBeNull();
      expect(mockApi.deleteNote).toHaveBeenCalledWith(1);
      expect(mockApi.listNotes).toHaveBeenCalled();
    });
  });

  describe("permanentDelete", () => {
    it("永久删除当前笔记时应清空 currentNote", async () => {
      const store = useNotesStore();
      store.currentNote = makeNote();
      mockApi.permanentDelete.mockResolvedValue(true);
      mockApi.listNotes.mockResolvedValue([]);
      mockApi.getStats.mockResolvedValue({ count: 0, words: 0 });

      await store.permanentDelete(1);

      expect(store.currentNote).toBeNull();
    });
  });

  describe("restoreNote", () => {
    it("恢复后应刷新列表", async () => {
      const store = useNotesStore();
      mockApi.restoreNote.mockResolvedValue(true);
      mockApi.listNotes.mockResolvedValue([makeNote()]);
      mockApi.getStats.mockResolvedValue({ count: 1, words: 5 });

      await store.restoreNote(1);

      expect(mockApi.restoreNote).toHaveBeenCalledWith(1);
    });
  });

  // ========== 收藏 / 置顶 ==========

  describe("toggleFavorite", () => {
    it("应更新 currentNote 的收藏状态并刷新列表", async () => {
      const store = useNotesStore();
      store.currentNote = makeNote({ is_favorite: false });
      mockApi.toggleFavorite.mockResolvedValue(true);
      mockApi.listNotes.mockResolvedValue([]);
      mockApi.getStats.mockResolvedValue({ count: 0, words: 0 });

      const result = await store.toggleFavorite(1);

      expect(result).toBe(true);
      expect(store.currentNote!.is_favorite).toBe(true);
    });
  });

  describe("togglePin", () => {
    it("应刷新列表", async () => {
      const store = useNotesStore();
      mockApi.togglePin.mockResolvedValue(true);
      mockApi.listNotes.mockResolvedValue([]);
      mockApi.getStats.mockResolvedValue({ count: 0, words: 0 });

      await store.togglePin(1);

      expect(mockApi.togglePin).toHaveBeenCalledWith(1);
    });
  });

  // ========== 计算属性 ==========

  describe("计算属性", () => {
    it("isTrash: 在回收站视图时为 true", () => {
      const store = useNotesStore();
      store.currentFilter = "trash";
      expect(store.isTrash).toBe(true);

      store.currentFilter = "all";
      expect(store.isTrash).toBe(false);
    });

    it("currentNoteId: 返回当前笔记 id，无笔记返回 null", () => {
      const store = useNotesStore();
      expect(store.currentNoteId).toBeNull();

      store.currentNote = makeNote({ id: 42 });
      expect(store.currentNoteId).toBe(42);
    });
  });
});
