<!-- 笔记列表 - 中间栏（280px）：搜索框 + 卡片列表 + 新建按钮 + 统计 -->
<script setup lang="ts">
import { ref, computed } from "vue";
import { useNotesStore } from "../stores/notes";
import { formatTime, truncate } from "../composables/utils";
import type { Note } from "../api/types";

const store = useNotesStore();

const searchRef = ref<HTMLInputElement | null>(null);
const searchText = ref("");

// 右键菜单状态
const menuVisible = ref(false);
const menuX = ref(0);
const menuY = ref(0);
const menuNote = ref<Note | null>(null);

// 暴露给父组件：聚焦搜索框
defineExpose({
  focusSearch() {
    searchRef.value?.focus();
    searchRef.value?.select();
  },
  clearSearch() {
    searchText.value = "";
    void store.search("");
  },
});

const emit = defineEmits<{
  (e: "toast", msg: string): void;
}>();

// 搜索输入
function onSearchInput() {
  void store.search(searchText.value);
}

// 点击卡片
function onCardClick(note: Note) {
  if (store.isTrash) {
    emit("toast", "回收站笔记需先恢复才能编辑");
    return;
  }
  void store.selectNote(note.id);
}

// 右键菜单
function onContextMenu(e: MouseEvent, note: Note) {
  menuNote.value = note;
  menuX.value = e.clientX;
  menuY.value = e.clientY;
  menuVisible.value = true;
}

function closeMenu() {
  menuVisible.value = false;
  menuNote.value = null;
}

// 菜单操作
async function menuTogglePin() {
  if (menuNote.value) await store.togglePin(menuNote.value.id);
  closeMenu();
}
async function menuToggleFav() {
  if (menuNote.value) {
    const fav = await store.toggleFavorite(menuNote.value.id);
    emit("toast", fav ? "已收藏" : "已取消收藏");
  }
  closeMenu();
}
async function menuExport(fmt: "txt" | "md") {
  if (menuNote.value) {
    const path = await store.exportNote(menuNote.value.id, menuNote.value.title, fmt);
    if (path) emit("toast", "已导出");
  }
  closeMenu();
}
async function menuDelete() {
  if (menuNote.value) {
    await store.deleteNote(menuNote.value.id);
    emit("toast", "已移至回收站");
  }
  closeMenu();
}
async function menuRestore() {
  if (menuNote.value) {
    await store.restoreNote(menuNote.value.id);
    emit("toast", "已恢复笔记");
  }
  closeMenu();
}
async function menuPermanentDelete() {
  if (menuNote.value) {
    await store.permanentDelete(menuNote.value.id);
    emit("toast", "已永久删除");
  }
  closeMenu();
}

// 新建笔记
function onNewNote() {
  void store.createNote();
}

// 空状态文案
const emptyText = computed(() => {
  if (store.isTrash) return "🗑 回收站为空";
  if (store.currentFilter === "favorite") return "⭐ 暂无收藏";
  return "📝 暂无笔记，点击下方按钮新建";
});

// 底部统计文案
const statsText = computed(() => {
  const n = store.notes.length;
  if (store.isTrash) return `回收站共 ${n} 篇`;
  if (store.currentFilter === "favorite") return `收藏 ${n} 篇`;
  return `共 ${n} 篇笔记`;
});
</script>

<template>
  <div class="note-list" @click="closeMenu">
    <!-- 搜索框 -->
    <div class="search-box">
      <span class="search-icon">🔍</span>
      <input
        ref="searchRef"
        v-model="searchText"
        class="search-input"
        placeholder="搜索笔记..."
        @input="onSearchInput"
      />
    </div>

    <!-- 卡片列表 -->
    <div class="cards">
      <template v-if="store.notes.length > 0">
        <div
          v-for="note in store.notes"
          :key="note.id"
          class="card"
          :class="{ selected: store.currentNoteId === note.id }"
          @click="onCardClick(note)"
          @contextmenu.prevent="onContextMenu($event, note)"
        >
          <div class="card-top">
            <span class="card-title">{{ note.title || "无标题" }}</span>
            <span v-if="note.is_pinned" class="badge">📌</span>
            <span v-if="note.is_favorite" class="badge">⭐</span>
          </div>
          <div class="card-preview">{{ truncate(note.content, 20) || "暂无内容" }}</div>
          <div class="card-time">{{ formatTime(note.updated_at) }}</div>
        </div>
      </template>

      <!-- 空状态 -->
      <div v-else class="empty">{{ emptyText }}</div>
    </div>

    <!-- 新建按钮（回收站不显示） -->
    <button v-if="!store.isTrash" class="new-btn" @click="onNewNote">＋ 新建笔记</button>

    <!-- 底部统计 -->
    <div class="stats">{{ statsText }}</div>

    <!-- 右键菜单 -->
    <Teleport to="body">
      <div
        v-if="menuVisible && menuNote"
        class="context-menu"
        :style="{ left: menuX + 'px', top: menuY + 'px' }"
        @click.stop
      >
        <template v-if="store.isTrash">
          <div class="menu-item" @click="menuRestore">🔄 恢复笔记</div>
          <div class="menu-item danger" @click="menuPermanentDelete">⛔ 永久删除</div>
        </template>
        <template v-else>
          <div class="menu-item" @click="menuTogglePin">
            📌 {{ menuNote.is_pinned ? "取消置顶" : "置顶" }}
          </div>
          <div class="menu-item" @click="menuToggleFav">
            {{ menuNote.is_favorite ? "⭐ 取消收藏" : "☆ 收藏" }}
          </div>
          <div class="menu-sep"></div>
          <div class="menu-item" @click="menuExport('txt')">📄 导出为 .txt</div>
          <div class="menu-item" @click="menuExport('md')">📝 导出为 .md</div>
          <div class="menu-sep"></div>
          <div class="menu-item danger" @click="menuDelete">🗑 删除</div>
        </template>
      </div>
    </Teleport>
  </div>
</template>

<style scoped>
.note-list {
  width: 280px;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  padding: 12px 8px 8px;
  border-right: 1px solid var(--divider);
  gap: 8px;
}

/* 搜索框 */
.search-box {
  display: flex;
  align-items: center;
  background: var(--search-bg);
  border: 1px solid var(--card-border);
  border-radius: 8px;
  padding: 0 10px;
  height: 34px;
  gap: 6px;
}
.search-box:focus-within {
  border-color: var(--accent);
}
.search-icon {
  font-size: 13px;
  opacity: 0.7;
}
.search-input {
  flex: 1;
  font-size: 13px;
  color: var(--text-primary);
}
.search-input::placeholder {
  color: var(--text-placeholder);
}

/* 卡片列表 */
.cards {
  flex: 1;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 2px;
}

.card {
  background: var(--card-bg);
  border: 1px solid var(--card-border);
  border-radius: 10px;
  padding: 10px 12px;
  cursor: pointer;
  transition: background 0.15s ease;
}
.card:hover {
  background: var(--card-bg-hover);
}
.card.selected {
  background: var(--card-bg-selected);
  border-color: var(--accent);
}

.card-top {
  display: flex;
  align-items: center;
  gap: 4px;
  margin-bottom: 3px;
}
.card-title {
  flex: 1;
  font-size: 13px;
  font-weight: bold;
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.badge {
  font-size: 11px;
}
.card-preview {
  font-size: 11px;
  color: var(--text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  margin-bottom: 4px;
}
.card-time {
  font-size: 10px;
  color: var(--text-placeholder);
}

/* 空状态 */
.empty {
  text-align: center;
  color: var(--text-secondary);
  font-size: 12px;
  padding: 40px 12px;
  line-height: 1.8;
}

/* 新建按钮 */
.new-btn {
  height: 36px;
  background: var(--accent);
  color: var(--text-on-accent);
  border-radius: 8px;
  font-size: 13px;
  font-weight: bold;
  transition: background 0.15s ease;
}
.new-btn:hover {
  background: var(--accent-hover);
}

/* 统计 */
.stats {
  text-align: center;
  font-size: 10px;
  color: var(--text-secondary);
}

/* 右键菜单 */
.context-menu {
  position: fixed;
  background: var(--card-bg);
  border: 1px solid var(--divider);
  border-radius: 8px;
  padding: 4px;
  min-width: 150px;
  box-shadow: var(--shadow);
  z-index: 9998;
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
.menu-item.danger {
  color: var(--danger);
}
.menu-sep {
  height: 1px;
  background: var(--divider);
  margin: 4px 8px;
}
</style>
