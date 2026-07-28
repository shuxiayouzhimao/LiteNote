// Pinia Store - 应用配置与主题管理

import { defineStore } from "pinia";
import { ref } from "vue";
import * as api from "../api";
import type { AppConfig, FilterType } from "../api/types";
import { enable, disable, isEnabled } from "@tauri-apps/plugin-autostart";

export const useConfigStore = defineStore("config", () => {
  const theme = ref<"light" | "dark">("light");
  const autoStart = ref(false);
  const sidebarActive = ref<FilterType>("all");
  const lastOpenedNoteId = ref<number | null>(null);
  // 窗口不透明度 (0.3 ~ 1.0)
  const windowOpacity = ref(0.9);

  /** 从后端加载配置并应用主题 */
  async function loadConfig(): Promise<void> {
    const cfg = await api.loadConfig();
    theme.value = cfg.theme;
    autoStart.value = cfg.auto_start;
    sidebarActive.value = cfg.sidebar_active;
    lastOpenedNoteId.value = cfg.last_opened_note_id;
    windowOpacity.value = cfg.window_opacity;
    applyTheme(theme.value);
    applyOpacity(windowOpacity.value);

    // 同步真实的自启状态
    try {
      autoStart.value = await isEnabled();
    } catch {
      // 插件不可用时忽略
    }
  }

  /** 保存配置到后端 */
  async function save(): Promise<void> {
    const cfg: AppConfig = {
      theme: theme.value,
      auto_start: autoStart.value,
      sidebar_active: sidebarActive.value,
      last_opened_note_id: lastOpenedNoteId.value,
      window_width: 1100,
      window_height: 720,
      window_opacity: windowOpacity.value,
    };
    await api.saveConfig(cfg);
  }

  /** 应用主题到 DOM（通过 data-theme 属性触发 CSS 变量切换） */
  function applyTheme(t: "light" | "dark"): void {
    document.documentElement.setAttribute("data-theme", t);
  }

  /** 应用窗口不透明度到 DOM（设置 --win-opacity CSS 变量，控制 --window-bg 的 alpha） */
  function applyOpacity(opacity: number): void {
    // 钳制到合法区间
    const clamped = Math.min(1, Math.max(0.3, opacity));
    document.documentElement.style.setProperty("--win-opacity", String(clamped));
  }

  /** 切换主题 */
  async function toggleTheme(): Promise<void> {
    theme.value = theme.value === "light" ? "dark" : "light";
    applyTheme(theme.value);
    await save();
  }

  /** 设置主题 */
  async function setTheme(t: "light" | "dark"): Promise<void> {
    theme.value = t;
    applyTheme(t);
    await save();
  }

  /** 设置开机自启 */
  async function setAutoStart(enable_: boolean): Promise<void> {
    try {
      if (enable_) {
        await enable();
      } else {
        await disable();
      }
      autoStart.value = enable_;
      await save();
    } catch (e) {
      console.error("设置开机自启失败:", e);
    }
  }

  /** 设置窗口不透明度（实时应用，防抖保存） */
  let opacitySaveTimer: number | null = null;
  function setOpacity(opacity: number): void {
    windowOpacity.value = opacity;
    applyOpacity(opacity);
    // 拖动滑块时频繁触发，防抖 400ms 后再持久化
    if (opacitySaveTimer !== null) clearTimeout(opacitySaveTimer);
    opacitySaveTimer = window.setTimeout(() => {
      void save();
    }, 400);
  }

  /** 记录最后打开的笔记 */
  async function setLastOpened(id: number | null): Promise<void> {
    lastOpenedNoteId.value = id;
    await save();
  }

  /** 记录侧边栏选中项 */
  async function setSidebarActive(filter: FilterType): Promise<void> {
    sidebarActive.value = filter;
    await save();
  }

  return {
    theme,
    autoStart,
    sidebarActive,
    lastOpenedNoteId,
    windowOpacity,
    loadConfig,
    save,
    applyTheme,
    applyOpacity,
    toggleTheme,
    setTheme,
    setAutoStart,
    setOpacity,
    setLastOpened,
    setSidebarActive,
  };
});
