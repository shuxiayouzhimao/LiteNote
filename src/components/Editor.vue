<!-- 编辑区 - 右侧沉浸式编辑：标题 + 内容 + Markdown 预览 + 收藏/导出 + 字数统计 -->
<script setup lang="ts">
import { ref, computed, watch, nextTick } from "vue";
import MarkdownIt from "markdown-it";
import hljs from "highlight.js";
import { useNotesStore } from "../stores/notes";

const store = useNotesStore();

const titleRef = ref<HTMLInputElement | null>(null);
const contentRef = ref<HTMLTextAreaElement | null>(null);
const exportMenuVisible = ref(false);

// Markdown 解析器实例（集成 highlight.js 代码语法高亮）
const md = new MarkdownIt({
  html: false,        // 禁用原始 HTML（安全）
  linkify: true,      // 自动识别链接
  breaks: true,       // 换行转 <br>
  typographer: true,  // 智能引号/破折号
  highlight: (str: string, lang: string): string => {
    if (lang && hljs.getLanguage(lang)) {
      try {
        return `<pre><code class="hljs language-${lang}">${hljs.highlight(str, { language: lang }).value}</code></pre>`;
      } catch {
        // 高亮失败时降级为纯文本
      }
    }
    // 自动检测语言或无语言标记
    try {
      return `<pre><code class="hljs">${hljs.highlightAuto(str).value}</code></pre>`;
    } catch {
      return `<pre><code>${md.utils.escapeHtml(str)}</code></pre>`;
    }
  },
});

// 视图模式: 'edit' | 'split' | 'preview'
const viewMode = ref<"edit" | "split" | "preview">("edit");

// 模式循环切换
function cycleMode() {
  const modes: Array<"edit" | "split" | "preview"> = ["edit", "split", "preview"];
  const idx = modes.indexOf(viewMode.value);
  viewMode.value = modes[(idx + 1) % modes.length];
}

// Markdown 格式化工具栏：在光标处插入/包裹语法
type MarkdownAction = "bold" | "italic" | "heading" | "code" | "link" | "list";

function insertMarkdown(action: MarkdownAction) {
  const ta = contentRef.value;
  if (!ta) return;

  const start = ta.selectionStart;
  const end = ta.selectionEnd;
  const selected = ta.value.substring(start, end);
  const before = ta.value.substring(0, start);
  const after = ta.value.substring(end);

  let replacement = "";
  let selStart = start;
  let selEnd = start;

  switch (action) {
    case "bold":
      replacement = selected ? `**${selected}**` : "**加粗文本**";
      selStart = start + 2;
      selEnd = selected ? start + 2 + selected.length : start + 6;
      break;
    case "italic":
      replacement = selected ? `*${selected}*` : "*斜体文本*";
      selStart = start + 1;
      selEnd = selected ? start + 1 + selected.length : start + 5;
      break;
    case "heading":
      replacement = selected
        ? selected.split("\n").map((l) => `# ${l}`).join("\n")
        : `# 标题`;
      selStart = start + 2;
      selEnd = selected ? start + replacement.length : start + 4;
      break;
    case "code":
      replacement = selected ? `\`${selected}\`` : "`代码`";
      selStart = start + 1;
      selEnd = selected ? start + 1 + selected.length : start + 3;
      break;
    case "link":
      replacement = selected ? `[${selected}](url)` : "[链接文本](url)";
      selStart = selected ? start + 1 : start + 1;
      selEnd = selected ? start + 1 + selected.length : start + 5;
      break;
    case "list":
      replacement = selected
        ? selected.split("\n").map((l) => `- ${l}`).join("\n")
        : "- 列表项";
      selStart = start + 2;
      selEnd = selected ? start + replacement.length : start + 5;
      break;
  }

  const newContent = before + replacement + after;

  // 通过 store 更新内容（触发自动保存计时器）
  if (note.value) {
    store.onContentChanged(note.value.title, newContent);
  }

  // 恢复光标位置
  void nextTick(() => {
    ta.focus();
    ta.setSelectionRange(selStart, selEnd);
  });
}

// 编辑器内快捷键（仅当 textarea 聚焦时）
function onEditorKeydown(e: KeyboardEvent) {
  const ctrl = e.ctrlKey || e.metaKey;
  if (!ctrl) return;

  if (e.key.toLowerCase() === "b") {
    e.preventDefault();
    insertMarkdown("bold");
  } else if (e.key.toLowerCase() === "i") {
    e.preventDefault();
    insertMarkdown("italic");
  }
}

// 模式标签
const modeLabel = computed(() => {
  switch (viewMode.value) {
    case "edit": return "📝";
    case "split": return "🗂";
    case "preview": return "👁";
  }
});

const modeTooltip = computed(() => {
  switch (viewMode.value) {
    case "edit": return "纯文本编辑";
    case "split": return "分屏预览";
    case "preview": return "仅预览";
  }
});

// 暴露给父组件：聚焦标题、切换编辑模式
defineExpose({
  focusTitle() {
    void nextTick(() => {
      titleRef.value?.focus();
      titleRef.value?.select();
    });
  },
  cycleMode,
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

// Markdown 渲染为 HTML（用于预览）
const renderedHtml = computed(() => {
  const raw = note.value?.content ?? "";
  if (!raw.trim()) return "<p class='md-empty'>（空内容）</p>";
  return md.render(raw);
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

// 切换笔记时收起导出菜单、回到编辑模式
watch(note, () => {
  exportMenuVisible.value = false;
  viewMode.value = "edit";
});
</script>

<template>
  <div class="editor" :class="`mode-${viewMode}`" @click="exportMenuVisible = false">
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
        <button class="icon-btn mode-toggle" :title="modeTooltip" @click="cycleMode">
          {{ modeLabel }}
        </button>
        <span class="save-status">{{ saveText }}</span>
      </div>

      <!-- Markdown 格式化工具栏 -->
      <div class="md-toolbar">
        <button class="md-btn" title="加粗 (Ctrl+B)" @click="insertMarkdown('bold')"><b>B</b></button>
        <button class="md-btn" title="斜体 (Ctrl+I)" @click="insertMarkdown('italic')"><i>I</i></button>
        <button class="md-btn" title="标题" @click="insertMarkdown('heading')">H</button>
        <button class="md-btn" title="行内代码" @click="insertMarkdown('code')">&lt;/&gt;</button>
        <button class="md-btn" title="链接" @click="insertMarkdown('link')">🔗</button>
        <button class="md-btn" title="无序列表" @click="insertMarkdown('list')">📋</button>
      </div>

      <!-- 标题 -->
      <input ref="titleRef" v-model="title" class="title-input" placeholder="输入标题..." />

      <div class="divider"></div>

      <!-- 内容区：根据模式显示不同布局 -->
      <div class="editor-body">
        <!-- 纯文本 / 分屏：都显示编辑区 -->
        <textarea
          ref="contentRef"
          v-show="viewMode !== 'preview'"
          v-model="content"
          class="content-input"
          :class="{ 'split-pane': viewMode === 'split' }"
          placeholder="在这里开始写笔记...（支持 Markdown 语法）"
          @keydown="onEditorKeydown"
        ></textarea>

        <!-- 分屏/预览：显示渲染结果 -->
        <div
          v-if="viewMode !== 'edit'"
          class="markdown-preview"
          :class="{ 'split-pane': viewMode === 'split' }"
          v-html="renderedHtml"
        ></div>
      </div>

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
.mode-toggle {
  font-size: 15px;
}
.spacer {
  flex: 1;
}
.save-status {
  font-size: 11px;
  color: var(--text-secondary);
}

/* Markdown 格式化工具栏 */
.md-toolbar {
  display: flex;
  align-items: center;
  gap: 2px;
  padding: 4px 0;
}
.md-btn {
  width: 28px;
  height: 26px;
  border-radius: 5px;
  font-size: 12px;
  color: var(--text-secondary);
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.12s ease;
}
.md-btn:hover {
  background: var(--accent-light);
  color: var(--accent);
}
.md-btn b,
.md-btn i {
  font-size: 13px;
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

/* 编辑器主体 */
.editor-body {
  flex: 1;
  min-height: 0;
  display: flex;
  gap: 0;
}

/* 内容输入框 */
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

/* 分屏模式：编辑区和预览区各占一半 */
.content-input.split-pane {
  flex: 1;
  border-right: 1px solid var(--divider);
  padding-right: 16px;
}

.markdown-preview.split-pane {
  flex: 1;
  padding-left: 16px;
}

/* Markdown 预览区 */
.markdown-preview {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  font-size: 14px;
  line-height: 1.8;
  color: var(--text-primary);
  padding-bottom: 8px;
}

/* ===== Markdown 渲染内容样式（scoped 穿透） ===== */
.markdown-preview :deep(h1) {
  font-size: 1.6em;
  font-weight: 700;
  margin: 0.8em 0 0.4em;
  padding-bottom: 0.3em;
  border-bottom: 1px solid var(--divider);
}
.markdown-preview :deep(h2) {
  font-size: 1.35em;
  font-weight: 700;
  margin: 0.7em 0 0.35em;
  padding-bottom: 0.2em;
  border-bottom: 1px solid var(--divider);
}
.markdown-preview :deep(h3) {
  font-size: 1.15em;
  font-weight: 600;
  margin: 0.6em 0 0.3em;
}
.markdown-preview :deep(h4),
.markdown-preview :deep(h5),
.markdown-preview :deep(h6) {
  font-size: 1em;
  font-weight: 600;
  margin: 0.5em 0 0.25em;
}

.markdown-preview :deep(p) {
  margin: 0.4em 0;
}

.markdown-preview :deep(strong) {
  font-weight: 700;
  color: var(--text-primary);
}

.markdown-preview :deep(em) {
  font-style: italic;
}

.markdown-preview :deep(a) {
  color: var(--accent);
  text-decoration: none;
}
.markdown-preview :deep(a:hover) {
  text-decoration: underline;
}

/* 列表 */
.markdown-preview :deep(ul),
.markdown-preview :deep(ol) {
  padding-left: 1.5em;
  margin: 0.4em 0;
}
.markdown-preview :deep(li) {
  margin: 0.15em 0;
}
.markdown-preview :deep(li > p) {
  margin: 0;
}

/* 引用块 */
.markdown-preview :deep(blockquote) {
  margin: 0.5em 0;
  padding: 0.4em 0.8em;
  border-left: 3px solid var(--accent);
  background: var(--accent-light);
  border-radius: 0 6px 6px 0;
  color: var(--text-secondary);
}

/* 代码 */
.markdown-preview :deep(code) {
  font-family: "Cascadia Code", "Fira Code", "JetBrains Mono", Consolas, monospace;
  font-size: 0.9em;
  background: var(--search-bg);
  padding: 0.15em 0.4em;
  border-radius: 4px;
  color: var(--accent);
}

.markdown-preview :deep(pre) {
  background: var(--search-bg);
  padding: 12px 16px;
  border-radius: 8px;
  overflow-x: auto;
  margin: 0.6em 0;
}
.markdown-preview :deep(pre code) {
  background: none;
  padding: 0;
  color: var(--text-primary);
  font-size: 0.85em;
  line-height: 1.6;
}

/* 分割线 */
.markdown-preview :deep(hr) {
  border: none;
  border-top: 1px solid var(--divider);
  margin: 1em 0;
}

/* 表格 */
.markdown-preview :deep(table) {
  border-collapse: collapse;
  width: 100%;
  margin: 0.6em 0;
}
.markdown-preview :deep(th),
.markdown-preview :deep(td) {
  border: 1px solid var(--divider);
  padding: 6px 12px;
  text-align: left;
}
.markdown-preview :deep(th) {
  background: var(--accent-light);
  font-weight: 600;
}

/* 图片 */
.markdown-preview :deep(img) {
  max-width: 100%;
  border-radius: 6px;
  margin: 0.4em 0;
}

/* 任务列表 */
.markdown-preview :deep(input[type="checkbox"]) {
  margin-right: 0.4em;
  accent-color: var(--accent);
}

/* 空内容提示 */
.markdown-preview :deep(.md-empty) {
  color: var(--text-placeholder);
  font-style: italic;
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
