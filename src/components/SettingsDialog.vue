<!-- 设置对话框 - 主题切换 / 开机自启 / 清空回收站 / 关于 -->
<script setup lang="ts">
import { ref } from "vue";
import { useConfigStore } from "../stores/config";
import { useNotesStore } from "../stores/notes";
import { showConfirm } from "../composables/useConfirm";
import { checkUpdate, installUpdate } from "../api";

const configStore = useConfigStore();
const notesStore = useNotesStore();

const emit = defineEmits<{
  (e: "close"): void;
  (e: "toast", msg: string): void;
}>();

// 主题切换
function onThemeChange(theme: "light" | "dark") {
  void configStore.setTheme(theme);
}

// 开机自启切换
function onAutoStartChange(e: Event) {
  const checked = (e.target as HTMLInputElement).checked;
  void configStore.setAutoStart(checked);
}

// 窗口不透明度滑块：拖动时实时应用
function onOpacityInput(e: Event) {
  const val = parseFloat((e.target as HTMLInputElement).value);
  configStore.setOpacity(val);
}

// 清空回收站（带二次确认）
async function onClearTrash() {
  const ok = await showConfirm({
    title: "清空回收站",
    message: "确定要清空回收站吗？此操作不可恢复。",
    confirmText: "确定清空",
    danger: true,
  });
  if (!ok) return;
  const count = await notesStore.clearTrash();
  emit("toast", `已清空回收站（${count} 篇）`);
}

// 检查更新
const checkingUpdate = ref(false);
async function onCheckUpdate() {
  checkingUpdate.value = true;
  const result = await checkUpdate();
  checkingUpdate.value = false;
  if (result.hasUpdate && result.version) {
    const ok = await showConfirm({
      title: "发现新版本",
      message: `新版本 v${result.version} 可用，是否立即更新？`,
      confirmText: "立即更新",
    });
    if (ok) {
      try {
        await installUpdate();
      } catch {
        emit("toast", "更新失败，请稍后重试");
      }
    }
  } else {
    emit("toast", "已是最新版本");
  }
}
</script>

<template>
  <!-- 遮罩 -->
  <div class="overlay" @click.self="emit('close')">
    <div class="dialog">
      <div class="dialog-header">
        <span class="dialog-title">⚙️ 设置</span>
        <button class="close-btn" @click="emit('close')">✕</button>
      </div>

      <div class="dialog-body">
        <!-- 主题 -->
        <div class="row">
          <span class="label">外观主题</span>
          <div class="theme-switch">
            <button
              class="theme-opt"
              :class="{ active: configStore.theme === 'light' }"
              @click="onThemeChange('light')"
            >
              ☀️ 浅色
            </button>
            <button
              class="theme-opt"
              :class="{ active: configStore.theme === 'dark' }"
              @click="onThemeChange('dark')"
            >
              🌙 深色
            </button>
          </div>
        </div>

        <div class="sep"></div>

        <!-- 窗口透明度 -->
        <div class="row opacity-row">
          <span class="label">窗口透明度</span>
          <div class="opacity-control">
            <input
              type="range"
              class="opacity-slider"
              min="0.3"
              max="1"
              step="0.05"
              :value="configStore.windowOpacity"
              @input="onOpacityInput"
            />
            <span class="opacity-value">{{ Math.round(configStore.windowOpacity * 100) }}%</span>
          </div>
        </div>

        <div class="sep"></div>

        <!-- 开机自启 -->
        <div class="row">
          <span class="label">开机自动启动</span>
          <label class="switch">
            <input type="checkbox" :checked="configStore.autoStart" @change="onAutoStartChange" />
            <span class="slider"></span>
          </label>
        </div>

        <div class="sep"></div>

        <!-- 清空回收站 -->
        <div class="row">
          <span class="label">回收站管理</span>
          <button class="danger-btn" @click="onClearTrash">清空回收站</button>
        </div>
      </div>

      <!-- 检查更新 -->
      <div class="sep"></div>
      <div class="row">
        <span class="label">软件更新</span>
        <button class="update-btn" @click="onCheckUpdate">检查更新</button>
      </div>

      <!-- 关于 -->
      <div class="about">
        LiteNote 灵光记事本 v1.0.0<br />
        基于 Tauri + Vue 3 · 极轻量现代化桌面记事本
      </div>
    </div>
  </div>
</template>

<style scoped>
.overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.4);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.dialog {
  width: 420px;
  background: var(--window-bg-solid);
  border-radius: 12px;
  padding: 20px 24px;
  box-shadow: var(--shadow);
  border: 1px solid var(--divider);
}

.dialog-header {
  display: flex;
  align-items: center;
  margin-bottom: 16px;
}
.dialog-title {
  flex: 1;
  font-size: 18px;
  font-weight: bold;
  color: var(--text-primary);
}
.close-btn {
  width: 28px;
  height: 28px;
  border-radius: 6px;
  color: var(--text-secondary);
}
.close-btn:hover {
  background: var(--accent-light);
  color: var(--text-primary);
}

.dialog-body {
  display: flex;
  flex-direction: column;
}

.row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 0;
}
.label {
  font-size: 13px;
  color: var(--text-primary);
}

.sep {
  height: 1px;
  background: var(--divider);
}

/* 主题切换按钮组 */
.theme-switch {
  display: flex;
  gap: 4px;
  background: var(--search-bg);
  border-radius: 8px;
  padding: 3px;
}
.theme-opt {
  padding: 5px 12px;
  border-radius: 6px;
  font-size: 12px;
  color: var(--text-secondary);
}
.theme-opt.active {
  background: var(--accent);
  color: var(--text-on-accent);
}

/* 窗口透明度滑块 */
.opacity-row {
  align-items: center;
}
.opacity-control {
  display: flex;
  align-items: center;
  gap: 10px;
}
.opacity-value {
  font-size: 12px;
  color: var(--text-secondary);
  width: 36px;
  text-align: right;
}
.opacity-slider {
  -webkit-appearance: none;
  appearance: none;
  width: 140px;
  height: 6px;
  border-radius: 3px;
  background: var(--search-bg);
  outline: none;
  cursor: pointer;
}
.opacity-slider::-webkit-slider-thumb {
  -webkit-appearance: none;
  appearance: none;
  width: 16px;
  height: 16px;
  border-radius: 50%;
  background: var(--accent);
  border: 2px solid var(--card-bg);
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.2);
  transition: transform 0.1s ease;
}
.opacity-slider::-webkit-slider-thumb:hover {
  transform: scale(1.15);
}
.opacity-slider::-moz-range-thumb {
  width: 16px;
  height: 16px;
  border-radius: 50%;
  background: var(--accent);
  border: 2px solid var(--card-bg);
  cursor: pointer;
}

/* 开关 */
.switch {
  position: relative;
  display: inline-block;
  width: 42px;
  height: 24px;
}
.switch input {
  opacity: 0;
  width: 0;
  height: 0;
}
.slider {
  position: absolute;
  cursor: pointer;
  inset: 0;
  background: var(--text-placeholder);
  border-radius: 24px;
  transition: 0.3s;
}
.slider::before {
  content: "";
  position: absolute;
  height: 18px;
  width: 18px;
  left: 3px;
  bottom: 3px;
  background: #fff;
  border-radius: 50%;
  transition: 0.3s;
}
.switch input:checked + .slider {
  background: var(--accent);
}
.switch input:checked + .slider::before {
  transform: translateX(18px);
}

/* 危险按钮 */
.danger-btn {
  padding: 6px 14px;
  border-radius: 8px;
  font-size: 12px;
  color: var(--danger);
  border: 1px solid var(--danger);
}
.danger-btn:hover {
  background: var(--danger);
  color: #fff;
}

/* 检查更新按钮 */
.update-btn {
  padding: 6px 14px;
  border-radius: 8px;
  font-size: 12px;
  color: var(--accent);
  border: 1px solid var(--accent);
  transition: all 0.15s ease;
}
.update-btn:hover {
  background: var(--accent);
  color: var(--text-on-accent);
}

/* 关于 */
.about {
  margin-top: 16px;
  text-align: center;
  font-size: 11px;
  color: var(--text-secondary);
  line-height: 1.8;
}
</style>
