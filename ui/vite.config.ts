import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

// Tauri 桌面端通过 tauri.conf.json 的 build.frontendDist 指向这里的 dist/
export default defineConfig({
  plugins: [react()],
  // Tauri CLI 自己负责清屏，避免覆盖它的输出
  clearScreen: false,
  server: {
    port: 5173,
    // 端口被占用时直接失败，而不是换端口——devUrl 是写死的
    strictPort: true,
  },
  build: {
    outDir: 'dist',
    emptyOutDir: true,
    target: 'es2022',
    sourcemap: false,
    // 关闭 modulePreload polyfill：它会往 index.html 注入内联脚本，
    // 与 tauri.conf.json 里 `script-src 'self'` 的严格 CSP 冲突。
    // 所有 Tauri 使用的 WebView（WebKitGTK / WKWebView / WebView2）都支持 ES module。
    modulePreload: { polyfill: false },
  },
});
