import { defineConfig } from "vitest/config";
import vue from "@vitejs/plugin-vue";
import { resolve } from "path";

export default defineConfig({
  plugins: [vue()],
  test: {
    // happy-dom 比 jsdom 更快，对 Vue 组件测试足够
    environment: "happy-dom",
    // 全局测试工具（无需手动 import describe/it/expect）
    globals: true,
    // 测试文件匹配
    include: ["src/**/*.{test,spec}.{ts,js}"],
    // 排除 node_modules
    exclude: ["node_modules", "src-tauri"],
    // 覆盖率（可选，后续 CI 中启用）
    coverage: {
      provider: "v8",
      reporter: ["text", "lcov"],
      include: ["src/**/*.ts"],
    },
  },
  resolve: {
    alias: {
      "@": resolve(__dirname, "src"),
    },
  },
});
