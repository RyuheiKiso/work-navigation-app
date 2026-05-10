// 対応 §: ロードマップ §7.1
// React エントリポイント。

// React API
import React from 'react';
import ReactDOM from 'react-dom/client';
// ルートコンポーネント
import { App } from './App';
import { ErrorBoundary } from './presentation/components/error-boundary';
// :focus-visible / prefers-reduced-motion 等のベースライン CSS
import './global.css';
// 設計トークンを CSS 変数として注入する（テーマ可変）。global.css より後に挿入し、上書き可能にする。
import { injectTokensStyle } from './tokens/css-vars';

injectTokensStyle();

// マウント先の要素を取得
const rootElement = document.getElementById('root');
// 型ガード: 取得失敗時は明示的にエラーを投げる
if (rootElement === null) {
  // DOM 構造の不整合は致命的
  throw new Error('id="root" の要素が見つかりません');
}

// React 18 の concurrent root を作成
const root = ReactDOM.createRoot(rootElement);
// レンダリング（StrictMode で副作用検査を有効化）
root.render(
  <React.StrictMode>
    <ErrorBoundary>
      <App />
    </ErrorBoundary>
  </React.StrictMode>
);
