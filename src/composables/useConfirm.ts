// 确认对话框 composable — 全局单例，任何组件可调用 showConfirm() 弹出确认框
import { ref } from "vue";

export interface ConfirmOptions {
  title: string;
  message: string;
  confirmText?: string;
  cancelText?: string;
  danger?: boolean;
}

// 模块级状态（全局单例，类似 useToast）
const visible = ref(false);
const options = ref<ConfirmOptions>({ title: "", message: "" });

let resolvePromise: ((value: boolean) => void) | null = null;

/** 弹出确认对话框，返回 Promise<boolean>（确定=true，取消=false） */
export function showConfirm(opts: ConfirmOptions): Promise<boolean> {
  options.value = opts;
  visible.value = true;
  return new Promise((resolve) => {
    resolvePromise = resolve;
  });
}

export function useConfirm() {
  function confirm() {
    if (resolvePromise) {
      resolvePromise(true);
      resolvePromise = null;
    }
    visible.value = false;
  }

  function cancel() {
    if (resolvePromise) {
      resolvePromise(false);
      resolvePromise = null;
    }
    visible.value = false;
  }

  return {
    visible,
    options,
    confirm,
    cancel,
    // 便捷方法
    showConfirm,
  };
}
