// 组合式函数 - Toast 轻提示
// 全局单例，用于显示"已移至回收站"等短暂提示

import { ref } from "vue";

interface ToastState {
  visible: boolean;
  message: string;
}

const state = ref<ToastState>({ visible: false, message: "" });
let hideTimer: number | null = null;

export function useToast() {
  /** 显示一条 toast，duration 毫秒后自动消失 */
  function showToast(message: string, duration = 2000): void {
    state.value.message = message;
    state.value.visible = true;
    if (hideTimer !== null) clearTimeout(hideTimer);
    hideTimer = window.setTimeout(() => {
      state.value.visible = false;
    }, duration);
  }

  return { toastState: state, showToast };
}
