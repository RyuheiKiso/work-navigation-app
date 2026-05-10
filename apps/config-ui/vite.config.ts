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
    setupFiles: ['./src/test-setup.ts'],
    coverage: {
      provider: 'v8',
      reporter: ['text', 'lcov', 'html'],
      include: ['src/**/*.{ts,tsx}'],
      exclude: [
        'src/**/*.test.{ts,tsx}',
        'src/**/*.property.test.{ts,tsx}',
        'src/test-setup.ts',
        'src/main.tsx'
      ],
      thresholds: {
        // Phase 2 ベースライン: 落ちないラインを置く。Phase 3 で 60% へ引き上げる予定。
        lines: 35,
        statements: 35,
        functions: 35,
        branches: 60
      }
    }
  }
});
