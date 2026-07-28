<!-- 根组件 - 无边框毛玻璃三栏布局，整合所有子组件、快捷键、托盘/全局事件监听 -->
<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import Sidebar from "./components/Sidebar.vue";
import TitleBar from "./components/TitleBar.vue";
import NoteList from "./components/NoteList.vue";
import Editor from "./components/Editor.vue";
import SettingsDialog from "./components/SettingsDialog.vue";
import Toast from "./components/Toast.vue";
import { useNotesStore } from "./stores/notes";
import { useConfigStore } from "./stores/config";
import { useToast } from "./composables/useToast";
import type { FilterType } from "./api/types";

const notesStore = useNotesStore();
const configStore = useConfigStore();
const { showToast } = useToast();

const noteListRef = ref<InstanceType<typeof NoteList> | null>(null);
const editorRef = ref<InstanceType<typeof Editor> | null>(null);
const settingsVisible = ref(false);
const sidebarActive = ref<FilterType>("all");

let unlisteners: UnlistenFn[] = [];

// ========== 初始化 ==========
onMounted(async () => {
  // 1. 加载配置 + 主题
  await configStore.loadConfig();
  sidebarActive.value = configStore.sidebarActive;

  // 2. 加载笔记列表
  notesStore.currentFilter = sidebarActive.value;
  await notesStore.refreshList();

  // 3. 恢复上次打开的笔记
  const lastId = configStore.lastOpenedNoteId;
  if (lastId && !notesStore.isTrash) {
    const exists = notesStore.notes.find((n) => n.id === lastId);
    if (exists) await notesStore.selectNote(lastId);
  }

  // 4. 注册快捷键
  window.addEventListener("keydown", onKeyDown);

  // 5. 监听后端事件（托盘/全局快捷键新建笔记）
  unlisteners.push(await listen("tray://new-note", () => void handleNewNote()));
  unlisteners.push(await listen("shortcut://new-note", () => void handleNewNote()));
});

onUnmounted(() => {
  window.removeEventListener("keydown", onKeyDown);
  unlisteners.forEach((fn) => fn());
});

// ========== 侧边栏导航 ==========
async function onNavigate(key: FilterType) {
  sidebarActive.value = key;
  noteListRef.value?.clearSearch();
  await notesStore.switchFilter(key);
  await configStore.setSidebarActive(key);
}

function onOpenSettings() {
  settingsVisible.value = true;
}

// ========== 新建笔记 ==========
async function handleNewNote() {
  sidebarActive.value = "all";
  noteListRef.value?.clearSearch();
  await notesStore.createNote();
  await configStore.setLastOpened(notesStore.currentNoteId);
  editorRef.value?.focusTitle();
}

// ========== 快捷键 ==========
function onKeyDown(e: KeyboardEvent) {
  const ctrl = e.ctrlKey || e.metaKey;

  // Ctrl+N 新建
  if (ctrl && !e.shiftKey && e.key.toLowerCase() === "n") {
    e.preventDefault();
    void handleNewNote();
    return;
  }
  // Ctrl+Shift+N 新建（应用内也支持）
  if (ctrl && e.shiftKey && e.key.toLowerCase() === "n") {
    e.preventDefault();
    void handleNewNote();
    return;
  }
  // Ctrl+F 聚焦搜索
  if (ctrl && e.key.toLowerCase() === "f") {
    e.preventDefault();
    noteListRef.value?.focusSearch();
    return;
  }
  // Ctrl+S 手动保存
  if (ctrl && e.key.toLowerCase() === "s") {
    e.preventDefault();
    void notesStore.manualSave();
    showToast("已保存");
    return;
  }
  // Ctrl+W 关闭当前笔记
  if (ctrl && e.key.toLowerCase() === "w") {
    e.preventDefault();
    void notesStore.clearSelection();
    return;
  }
  // Delete 删除选中笔记（仅当焦点不在输入框时）
  if (e.key === "Delete" && !isEditingText(e)) {
    const id = notesStore.currentNoteId;
    if (id) {
      e.preventDefault();
      if (notesStore.isTrash) {
        void notesStore.permanentDelete(id).then(() => showToast("已永久删除"));
      } else {
        void notesStore.deleteNote(id).then(() => showToast("已移至回收站"));
      }
    }
    return;
  }
  // Esc 取消选中 / 退出搜索
  if (e.key === "Escape") {
    const target = e.target as HTMLElement;
    if (target.tagName === "INPUT" && target.className.includes("search")) {
      noteListRef.value?.clearSearch();
      target.blur();
    } else {
      void notesStore.clearSelection();
    }
    return;
  }
}

// 判断当前焦点是否在文本输入区（避免误删笔记）
function isEditingText(e: KeyboardEvent): boolean {
  const target = e.target as HTMLElement;
  return target.tagName === "TEXTAREA" || target.tagName === "INPUT";
}

// Toast 转发
function onToast(msg: string) {
  showToast(msg);
}
</script>

<template>
  <div class="app-shell">
    <div class="main-frame">
      <!-- 标题栏 -->
      <TitleBar />

      <!-- 三栏内容 -->
      <div class="content">
        <Sidebar :active="sidebarActive" @navigate="onNavigate" @open-settings="onOpenSettings" />
        <NoteList ref="noteListRef" @toast="onToast" />
        <Editor ref="editorRef" @toast="onToast" />
      </div>

      <!-- 底部状态栏 -->
      <div class="status-bar">笔记: {{ notesStore.stats.count }} | 字数: {{ notesStore.stats.words }}</div>
    </div>

    <!-- 设置对话框 -->
    <SettingsDialog v-if="settingsVisible" @close="settingsVisible = false" @toast="onToast" />

    <!-- Toast -->
    <Toast />
  </div>
</template>

<style scoped>
.app-shell {
  width: 100%;
  height: 100%;
  padding: 0;
  background: transparent;
}

/* 主框：圆角 + 毛玻璃背景 + 阴影 */
.main-frame {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--window-bg);
  /* 毛玻璃：backdrop-filter 让窗口后的内容模糊 */
  backdrop-filter: blur(30px) saturate(180%);
  -webkit-backdrop-filter: blur(30px) saturate(180%);
  border-radius: 12px;
  overflow: hidden;
  border: 1px solid var(--card-border);
}

.content {
  flex: 1;
  min-height: 0;
  display: flex;
}

.status-bar {
  height: 28px;
  display: flex;
  align-items: center;
  padding: 0 16px;
  font-size: 10px;
  color: var(--text-secondary);
  border-top: 1px solid var(--divider);
}
</style>
