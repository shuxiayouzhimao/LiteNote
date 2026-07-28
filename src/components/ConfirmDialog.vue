<!-- 通用确认对话框 — 全局单例，通过 useConfirm() 控制 -->
<script setup lang="ts">
import { useConfirm } from "../composables/useConfirm";

const { visible, options, confirm, cancel } = useConfirm();
</script>

<template>
  <Teleport to="body">
    <!-- 遮罩 -->
    <div v-if="visible" class="confirm-overlay" @click.self="cancel">
      <div class="confirm-box">
        <div class="confirm-title">{{ options.title }}</div>
        <div class="confirm-message">{{ options.message }}</div>
        <div class="confirm-actions">
          <button class="btn-cancel" @click="cancel">
            {{ options.cancelText || "取消" }}
          </button>
          <button
            class="btn-confirm"
            :class="{ danger: options.danger }"
            @click="confirm"
          >
            {{ options.confirmText || "确定" }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.confirm-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.35);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 10001;
}

.confirm-box {
  background: var(--window-bg-solid);
  border-radius: 12px;
  padding: 24px;
  width: 320px;
  box-shadow: var(--shadow);
  border: 1px solid var(--divider);
}

.confirm-title {
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 8px;
}

.confirm-message {
  font-size: 13px;
  color: var(--text-secondary);
  line-height: 1.7;
  margin-bottom: 20px;
}

.confirm-actions {
  display: flex;
  gap: 12px;
}

.btn-cancel,
.btn-confirm {
  flex: 1;
  padding: 9px;
  border-radius: 8px;
  font-size: 13px;
  font-weight: 500;
  transition: all 0.15s ease;
}

.btn-cancel {
  background: var(--search-bg);
  color: var(--text-primary);
}
.btn-cancel:hover {
  background: var(--divider);
}

.btn-confirm {
  background: var(--accent);
  color: var(--text-on-accent);
}
.btn-confirm:hover {
  background: var(--accent-hover);
}

.btn-confirm.danger {
  background: var(--danger);
}
.btn-confirm.danger:hover {
  background: var(--danger-hover);
}
</style>
