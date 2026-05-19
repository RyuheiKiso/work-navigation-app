/// <reference types="vitest" />
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const dirname = path.dirname(fileURLToPath(import.meta.url));

// バックエンド axum (port 8081) を Vite dev server (port 5173) からプロキシし CORS を回避
export default defineConfig({
  plugins: [react()],
  resolve: {
    // @wnav/shared は package.json exports（subpath 単位）で解決する
    alias: {
      '@': path.resolve(dirname, './src'),
    },
    preserveSymlinks: false,
  },
  server: {
    port: 5173,
    proxy: {
      '/api': {
        target: 'http://localhost:8081',
        changeOrigin: true,
      },
    },
  },
  test: {
    environment: 'jsdom',
    globals: true,
    setupFiles: ['./src/test-setup.ts'],
    css: false,
  },
  build: {
    outDir: 'dist',
    sourcemap: true,
    target: 'es2022',
  },
});
