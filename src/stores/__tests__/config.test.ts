/**
 * config store 单元测试
 * 测试主题切换、窗口不透明度、配置持久化逻辑
 */
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { setActivePinia, createPinia } from "pinia";

// ---------- Mock API ----------
vi.mock("../../api", () => ({
  loadConfig: vi.fn(),
  saveConfig: vi.fn(),
}));

// ---------- Mock Tauri autostart plugin ----------
vi.mock("@tauri-apps/plugin-autostart", () => ({
  enable: vi.fn().mockResolvedValue(undefined),
  disable: vi.fn().mockResolvedValue(undefined),
  isEnabled: vi.fn().mockResolvedValue(false),
}));

import * as mockApi from "../../api";

import { useConfigStore } from "../config";
import { enable, disable, isEnabled } from "@tauri-apps/plugin-autostart";

// 默认配置 fixture
function defaultConfig(overrides = {}) {
  return {
    theme: "light" as const,
    auto_start: false,
    sidebar_active: "all" as const,
    last_opened_note_id: null,
    window_width: 1100,
    window_height: 720,
    window_opacity: 0.9,
    ...overrides,
  };
}

describe("useConfigStore", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.clearAllMocks();
    vi.useFakeTimers();

    // 清理 DOM 状态
    document.documentElement.removeAttribute("data-theme");
    document.documentElement.style.removeProperty("--win-opacity");
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  // ========== loadConfig ==========

  describe("loadConfig", () => {
    it("应从 API 加载配置并应用到 DOM", async () => {
      const store = useConfigStore();
      const cfg = defaultConfig({ theme: "dark", window_opacity: 0.7 });
      mockApi.loadConfig.mockResolvedValue(cfg);
      vi.mocked(isEnabled).mockResolvedValue(false);

      await store.loadConfig();

      expect(store.theme).toBe("dark");
      expect(store.windowOpacity).toBe(0.7);
      expect(store.autoStart).toBe(false);
      expect(store.sidebarActive).toBe("all");
      expect(document.documentElement.getAttribute("data-theme")).toBe("dark");
      expect(document.documentElement.style.getPropertyValue("--win-opacity")).toBe("0.7");
    });

    it("应同步真实的自启状态（覆盖配置值）", async () => {
      const store = useConfigStore();
      mockApi.loadConfig.mockResolvedValue(defaultConfig({ auto_start: true }));
      vi.mocked(isEnabled).mockResolvedValue(true); // 系统层面确实开启了

      await store.loadConfig();

      expect(store.autoStart).toBe(true);
    });

    it("自启插件不可用时应容错", async () => {
      const store = useConfigStore();
      mockApi.loadConfig.mockResolvedValue(defaultConfig({ auto_start: false }));
      vi.mocked(isEnabled).mockRejectedValue(new Error("plugin not available"));

      // 不应抛出异常
      await expect(store.loadConfig()).resolves.toBeUndefined();
      expect(store.autoStart).toBe(false);
    });
  });

  // ========== save ==========

  describe("save", () => {
    it("应收集状态并调用 API 保存", async () => {
      const store = useConfigStore();
      store.theme = "dark";
      store.windowOpacity = 0.5;
      store.sidebarActive = "favorite";
      store.lastOpenedNoteId = 42;
      mockApi.saveConfig.mockResolvedValue(undefined);

      await store.save();

      // 窗口尺寸动态读取，使用 objectContaining 匹配核心字段
      expect(mockApi.saveConfig).toHaveBeenCalledWith(
        expect.objectContaining({
          theme: "dark",
          auto_start: false,
          sidebar_active: "favorite",
          last_opened_note_id: 42,
          window_opacity: 0.5,
        }),
      );
    });
  });

  // ========== toggleTheme ==========

  describe("toggleTheme", () => {
    it("应从 light 切换到 dark", async () => {
      const store = useConfigStore();
      store.theme = "light";
      mockApi.saveConfig.mockResolvedValue(undefined);

      await store.toggleTheme();

      expect(store.theme).toBe("dark");
      expect(document.documentElement.getAttribute("data-theme")).toBe("dark");
      expect(mockApi.saveConfig).toHaveBeenCalled();
    });

    it("应从 dark 切换到 light", async () => {
      const store = useConfigStore();
      store.theme = "dark";
      mockApi.saveConfig.mockResolvedValue(undefined);

      await store.toggleTheme();

      expect(store.theme).toBe("light");
      expect(document.documentElement.getAttribute("data-theme")).toBe("light");
    });
  });

  // ========== setOpacity ==========

  describe("setOpacity", () => {
    it("应钳制到 [0.3, 1.0] 区间", async () => {
      const store = useConfigStore();
      mockApi.saveConfig.mockResolvedValue(undefined);

      store.setOpacity(0.1);
      expect(store.windowOpacity).toBe(0.1);
      expect(document.documentElement.style.getPropertyValue("--win-opacity")).toBe("0.3");

      store.setOpacity(1.5);
      expect(document.documentElement.style.getPropertyValue("--win-opacity")).toBe("1");
    });

    it("应防抖 400ms 后再保存", () => {
      const store = useConfigStore();
      mockApi.saveConfig.mockResolvedValue(undefined);

      store.setOpacity(0.5);
      store.setOpacity(0.6);
      store.setOpacity(0.7);

      expect(mockApi.saveConfig).not.toHaveBeenCalled();

      vi.advanceTimersByTime(400);

      expect(mockApi.saveConfig).toHaveBeenCalledTimes(1);
    });
  });

  // ========== setAutoStart ==========

  describe("setAutoStart", () => {
    it("开启时应调用 enable()", async () => {
      const store = useConfigStore();
      mockApi.saveConfig.mockResolvedValue(undefined);

      await store.setAutoStart(true);

      expect(enable).toHaveBeenCalled();
      expect(store.autoStart).toBe(true);
      expect(mockApi.saveConfig).toHaveBeenCalled();
    });

    it("关闭时应调用 disable()", async () => {
      const store = useConfigStore();
      mockApi.saveConfig.mockResolvedValue(undefined);

      await store.setAutoStart(false);

      expect(disable).toHaveBeenCalled();
      expect(store.autoStart).toBe(false);
    });

    it("插件报错时应容错", async () => {
      const store = useConfigStore();
      vi.mocked(enable).mockRejectedValue(new Error("failed"));
      // 不期望抛出
      await expect(store.setAutoStart(true)).resolves.toBeUndefined();
    });
  });
});
