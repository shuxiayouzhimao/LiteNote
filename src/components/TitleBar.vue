<!-- 自定义标题栏 - 无边框窗口拖动 + 主题切换 + 窗口控制按钮 -->
<script setup lang="ts">
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useConfigStore } from "../stores/config";

const configStore = useConfigStore();
const appWindow = getCurrentWindow();

// 最小化
function minimize() {
  void appWindow.minimize();
}

// 最大化 / 还原
async function toggleMaximize() {
  const isMax = await appWindow.isMaximized();
  if (isMax) {
    await appWindow.unmaximize();
  } else {
    await appWindow.maximize();
  }
}

// 关闭（触发后端 CloseRequested → 隐藏到托盘）
function close() {
  void appWindow.close();
}

// 切换主题
function toggleTheme() {
  void configStore.toggleTheme();
}
</script>

<template>
  <!-- data-tauri-drag-region 让该区域可拖动窗口 -->
  <div class="title-bar" data-tauri-drag-region>
    <span class="title" data-tauri-drag-region>LiteNote</span>
    <div class="spacer" data-tauri-drag-region></div>

    <button class="win-btn" :title="configStore.theme === 'light' ? '切换深色' : '切换浅色'" @click="toggleTheme">
      {{ configStore.theme === "light" ? "🌙" : "☀️" }}
    </button>
    <button class="win-btn" title="最小化" @click="minimize">─</button>
    <button class="win-btn" title="最大化" @click="toggleMaximize">□</button>
    <button class="win-btn close" title="关闭" @click="close">✕</button>
  </div>
</template>

<style scoped>
.title-bar {
  height: 40px;
  display: flex;
  align-items: center;
  padding: 0 8px 0 16px;
  flex-shrink: 0;
}

.title {
  font-size: 12px;
  font-weight: bold;
  color: var(--text-primary);
}

.spacer {
  flex: 1;
}

.win-btn {
  width: 32px;
  height: 28px;
  border-radius: 6px;
  color: var(--text-secondary);
  font-size: 13px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.15s ease;
}

.win-btn:hover {
  background: var(--accent-light);
  color: var(--text-primary);
}

.win-btn.close:hover {
  background: var(--danger);
  color: #fff;
}
</style>
