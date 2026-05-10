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
    setupFiles: ['./src/test-setup.ts'],
    // §13.2 テスト基盤: カバレッジ計測（v1 は閾値を緩めに、Phase 2 で 80% へ引き上げ予定）
    coverage: {
      provider: 'v8',
      reporter: ['text', 'lcov', 'html'],
      include: ['src/**/*.{ts,tsx}'],
      exclude: [
        'src/**/*.test.{ts,tsx}',
        'src/**/*.property.test.{ts,tsx}',
        'src/test-setup.ts',
        'src/main.tsx',
        'src/i18n/{ar,he,zh,ko,de,es,vi,th,id,fr,pt}.ts'
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
