<!-- 编辑区 - 右侧沉浸式编辑：标题 + 内容 + 收藏/导出 + 字数统计 -->
<script setup lang="ts">
import { ref, computed, watch, nextTick } from "vue";
import { useNotesStore } from "../stores/notes";

const store = useNotesStore();

const titleRef = ref<HTMLInputElement | null>(null);
const exportMenuVisible = ref(false);

// 暴露给父组件：聚焦标题（新建笔记后调用）
defineExpose({
  focusTitle() {
    void nextTick(() => {
      titleRef.value?.focus();
      titleRef.value?.select();
    });
  },
});

const emit = defineEmits<{
  (e: "toast", msg: string): void;
}>();

// 当前笔记（可能为 null → 显示空状态）
const note = computed(() => store.currentNote);

// 标题/内容双向绑定 —— 通过 computed 的 get/set 触发自动保存
const title = computed({
  get: () => note.value?.title ?? "",
  set: (v: string) => {
    if (note.value) store.onContentChanged(v, note.value.content);
  },
});
const content = computed({
  get: () => note.value?.content ?? "",
  set: (v: string) => {
    if (note.value) store.onContentChanged(note.value.title, v);
  },
});

// 字数统计
const wordCount = computed(() => {
  const text = note.value?.content ?? "";
  return text.replace(/\s/g, "").length;
});

// 保存状态文案
const saveText = computed(() => {
  if (store.saveStatus === "unsaved") return "● 未保存";
  if (store.saveStatus === "saved") return "✓ 已保存";
  return "";
});

// 收藏切换
async function onToggleFav() {
  if (note.value) {
    const fav = await store.toggleFavorite(note.value.id);
    emit("toast", fav ? "已收藏" : "已取消收藏");
  }
}

// 导出
async function onExport(fmt: "txt" | "md") {
  exportMenuVisible.value = false;
  if (note.value) {
    const path = await store.exportNote(note.value.id, note.value.title, fmt);
    if (path) emit("toast", "已导出");
  }
}

// 切换笔记时收起导出菜单
watch(note, () => {
  exportMenuVisible.value = false;
});
</script>

<template>
  <div class="editor" @click="exportMenuVisible = false">
    <!-- 有笔记：显示编辑器 -->
    <template v-if="note">
      <!-- 顶部操作栏 -->
      <div class="top-bar">
        <button class="icon-btn" :class="{ starred: note.is_favorite }" title="收藏 / 取消收藏" @click="onToggleFav">
          {{ note.is_favorite ? "★" : "☆" }}
        </button>
        <div class="export-wrap" @click.stop>
          <button class="icon-btn" title="导出笔记" @click="exportMenuVisible = !exportMenuVisible">📤</button>
          <div v-if="exportMenuVisible" class="export-menu">
            <div class="menu-item" @click="onExport('txt')">导出为 .txt</div>
            <div class="menu-item" @click="onExport('md')">导出为 .md</div>
          </div>
        </div>
        <div class="spacer"></div>
        <span class="save-status">{{ saveText }}</span>
      </div>

      <!-- 标题 -->
      <input ref="titleRef" v-model="title" class="title-input" placeholder="输入标题..." />

      <div class="divider"></div>

      <!-- 内容 -->
      <textarea v-model="content" class="content-input" placeholder="在这里开始写笔记..."></textarea>

      <!-- 底部字数 -->
      <div class="bottom-bar">字数: {{ wordCount }}</div>
    </template>

    <!-- 无笔记：空状态 -->
    <div v-else class="empty-state">
      <div class="empty-icon">📝</div>
      <div class="empty-text">选择一篇笔记开始编辑</div>
      <div class="empty-hint">或按 Ctrl+N 创建新笔记</div>
    </div>
  </div>
</template>

<style scoped>
.editor {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  padding: 16px 28px;
}

/* 顶部栏 */
.top-bar {
  display: flex;
  align-items: center;
  gap: 8px;
  height: 36px;
}
.icon-btn {
  width: 32px;
  height: 32px;
  border-radius: 8px;
  font-size: 16px;
  color: var(--text-secondary);
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.15s ease;
}
.icon-btn:hover {
  background: var(--accent-light);
  color: var(--text-primary);
}
.icon-btn.starred {
  color: var(--star);
}
.spacer {
  flex: 1;
}
.save-status {
  font-size: 11px;
  color: var(--text-secondary);
}

/* 导出菜单 */
.export-wrap {
  position: relative;
}
.export-menu {
  position: absolute;
  top: 36px;
  left: 0;
  background: var(--card-bg);
  border: 1px solid var(--divider);
  border-radius: 8px;
  padding: 4px;
  min-width: 130px;
  box-shadow: var(--shadow);
  z-index: 100;
}
.menu-item {
  padding: 7px 12px;
  border-radius: 5px;
  font-size: 13px;
  cursor: pointer;
  color: var(--text-primary);
}
.menu-item:hover {
  background: var(--accent-light);
}

/* 标题 */
.title-input {
  font-size: 20px;
  font-weight: bold;
  color: var(--text-primary);
  padding: 6px 0;
  margin-top: 4px;
}
.title-input::placeholder {
  color: var(--text-placeholder);
}

.divider {
  height: 1px;
  background: var(--divider);
  margin: 4px 0 8px;
}

/* 内容 */
.content-input {
  flex: 1;
  min-height: 0;
  font-size: 14px;
  line-height: 1.8;
  color: var(--text-primary);
  resize: none;
  overflow-y: auto;
}
.content-input::placeholder {
  color: var(--text-placeholder);
}

/* 底部字数 */
.bottom-bar {
  font-size: 11px;
  color: var(--text-secondary);
  padding-top: 6px;
}

/* 空状态 */
.empty-state {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
  color: var(--text-secondary);
}
.empty-icon {
  font-size: 48px;
  opacity: 0.6;
}
.empty-text {
  font-size: 15px;
}
.empty-hint {
  font-size: 12px;
  color: var(--text-placeholder);
}
</style>
