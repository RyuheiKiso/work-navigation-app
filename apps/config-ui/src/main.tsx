// 対応 §: ロードマップ §7.2
// 設定 UI のエントリポイント。

// React API
import React from 'react';
import ReactDOM from 'react-dom/client';
// ルート
import { App } from './App';
import { ErrorBoundary } from './presentation/components/error-boundary';
// :focus-visible / prefers-reduced-motion 等のベースライン CSS
import './global.css';

// マウント先取得
const rootElement = document.getElementById('root');
// 型ガード
if (rootElement === null) {
  // DOM 不整合は致命的
  throw new Error('id="root" の要素が見つかりません');
}

// concurrent root 構築
const root = ReactDOM.createRoot(rootElement);
// レンダリング
root.render(
  <React.StrictMode>
    <ErrorBoundary>
      <App />
    </ErrorBoundary>
  </React.StrictMode>
);
