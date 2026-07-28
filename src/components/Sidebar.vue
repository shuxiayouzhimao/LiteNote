<!-- 侧边栏 - 极窄图标导航（60px）：全部 / 收藏 / 回收站 / 设置 -->
<script setup lang="ts">
import type { FilterType } from "../api/types";

const props = defineProps<{
  active: FilterType;
}>();

const emit = defineEmits<{
  (e: "navigate", key: FilterType): void;
  (e: "open-settings"): void;
}>();

// 导航项配置
const navItems: { icon: string; tip: string; key: FilterType }[] = [
  { icon: "📝", tip: "全部笔记", key: "all" },
  { icon: "⭐", tip: "收藏", key: "favorite" },
  { icon: "🗑", tip: "回收站", key: "trash" },
];
</script>

<template>
  <div class="sidebar">
    <!-- Logo -->
    <div class="logo">📓</div>

    <!-- 导航按钮 -->
    <nav class="nav">
      <button
        v-for="item in navItems"
        :key="item.key"
        class="nav-btn"
        :class="{ active: props.active === item.key }"
        :title="item.tip"
        @click="emit('navigate', item.key)"
      >
        {{ item.icon }}
      </button>
    </nav>

    <div class="spacer"></div>

    <!-- 分隔线 -->
    <div class="divider"></div>

    <!-- 设置按钮 -->
    <button class="nav-btn" title="设置" @click="emit('open-settings')">⚙️</button>
  </div>
</template>

<style scoped>
.sidebar {
  width: 60px;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 12px 0;
  background: var(--sidebar-bg);
  border-right: 1px solid var(--divider);
}

.logo {
  font-size: 24px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  margin-bottom: 8px;
}

.nav {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.nav-btn {
  width: 44px;
  height: 44px;
  border-radius: 10px;
  font-size: 20px;
  color: var(--sidebar-icon);
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s ease;
}

.nav-btn:hover {
  background: var(--accent-light);
}

.nav-btn.active {
  background: var(--accent-light);
  /* 用彩色圆点标识选中（emoji 无法直接染色，用左侧色条） */
  box-shadow: inset 3px 0 0 var(--accent);
}

.spacer {
  flex: 1;
}

.divider {
  width: 32px;
  height: 1px;
  background: var(--divider);
  margin: 4px 0 8px;
}
</style>
