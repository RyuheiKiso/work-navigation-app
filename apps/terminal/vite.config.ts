// 対応 §: ロードマップ §7.1 §13.2
// Vite + React の最小設定。vitest 用の test 設定も同居させる。

// Vite の defineConfig で型補完を効かせる
import { defineConfig } from 'vite';
// React プラグイン
import react from '@vitejs/plugin-react';

// 設定本体
export default defineConfig({
  // React JSX 変換のためのプラグインを有効化する
  plugins: [react()],
  // 開発サーバ設定（Tauri から expects するポート）
  server: {
    // ホットリロード用の固定ポート
    port: 1420,
    // ネットワーク非依存（オフライン前提、§10.6）
    strictPort: true
  },
  // Vitest 設定（vite-node 統合）
  test: {
    // jsdom で DOM を模倣する
    environment: 'jsdom',
    // グローバル `expect` を有効化する
    globals: true,
    // jest-dom のカスタムマッチャを登録（toBeInTheDocument 等）
    setupFiles: ['./src/test-setup.ts']
  }
});
