// 対応 §: ロードマップ §7.2 §13.2
// 設定 Web UI の Vite + React + Vitest 設定。

// defineConfig
import { defineConfig } from 'vite';
// React プラグイン
import react from '@vitejs/plugin-react';

// 設定本体
export default defineConfig({
  // React JSX 変換
  plugins: [react()],
  // 開発サーバ
  server: {
    // ポート（端末アプリと衝突しない 1421）
    port: 1421,
    // ポート固定
    strictPort: true
  },
  // Vitest
  test: {
    // jsdom
    environment: 'jsdom',
    // expect グローバル
    globals: true,
    // jest-dom のカスタムマッチャを登録（toBeInTheDocument 等）
    setupFiles: ['./src/test-setup.ts']
  }
});
