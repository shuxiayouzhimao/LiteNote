import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";

// Vite 配置 —— Tauri 官方推荐设置
// 参考 https://v2.tauri.app/start/frontend/vite/
export default defineConfig({
  plugins: [vue()],

  // Tauri 需要一个固定端口，失败时直接报错而非自动切换
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      // 忽略 src-tauri 目录，避免 Rust 编译触发前端热重载
      ignored: ["**/src-tauri/**"],
    },
  },

  // 构建产物供 Tauri 打包使用
  build: {
    // WebView2 (Windows) 支持较新特性
    target: "chrome105",
    minify: "esbuild",
    sourcemap: false,
  },
});
