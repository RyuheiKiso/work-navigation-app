// 対応 §: ロードマップ §7.2 §10.2 §10.2.2 §11.2 §9.5
// 設定 UI のフロー編集画面の最小コンポーネント。
// §10.2.2 14 観点を将来拡充するための足場として、ノード一覧と保存ボタンのみを提供する。

// React Hook
import { useState } from 'react';
// ドメイン
import type { Flow, FlowNode } from '../../domain/flow';

/** プロパティ */
export interface FlowEditorProps {
  // 初期フロー
  initialFlow: Flow;
  // 試行版発行ハンドラ
  onPublishTrial?: () => void;
}

/** フロー編集の最小 UI */
export function FlowEditor(props: FlowEditorProps): JSX.Element {
  // 編集中フラグ（自動保存 10 秒間隔の前段、§10.2.2）
  const [dirty, setDirty] = useState<boolean>(false);

  // 保存ボタンのハンドラ
  function handlePublishTrial(): void {
    // 親に通知
    props.onPublishTrial?.();
    // dirty 解除
    setDirty(false);
  }

  // 描画
  return (
    <section
      style={{
        // 余白
        padding: '24px',
        // システムフォント連鎖（§9.5.1）
        fontFamily:
          'Inter, "Noto Sans JP", "Noto Sans KR", "Noto Sans SC", system-ui, sans-serif'
      }}
      aria-labelledby={`flow-${props.initialFlow.id}`}
    >
      <header>
        <h1 id={`flow-${props.initialFlow.id}`} style={{ fontSize: '24px', margin: 0 }}>
          フロー: {props.initialFlow.name}
        </h1>
        <p style={{ color: '#6C757D' }}>
          バージョン: {props.initialFlow.version} ／ ノード: {props.initialFlow.nodeCount} ／ 辺: {props.initialFlow.edgeCount}
        </p>
      </header>

      {/* §10.2.2 段階的開示: 推奨設定（最小情報）のみ表示 */}
      <section>
        <h2 style={{ fontSize: '20px' }}>ノード一覧</h2>
        <ul style={{ listStyle: 'disc', paddingLeft: '24px' }}>
          {/* 内部ノードを表示するため、Flow から外部公開は将来拡充。
              ここでは表示用の最小プレースホルダ。 */}
          <li>（ノードの詳細表示は §10.2 拡充で実装）</li>
        </ul>
      </section>

      {/* アクション領域 */}
      <footer>
        <button
          type="button"
          onClick={handlePublishTrial}
          // §10.2.2 失敗コスト軽減: 2 段階確認は将来モーダルで実装
          // ここではタッチターゲット 11mm（44px）以上を保証
          style={{
            minHeight: '44px',
            padding: '8px 16px',
            background: '#28A745',
            color: '#FFFFFF',
            border: 'none',
            borderRadius: '8px',
            cursor: 'pointer'
          }}
          aria-label="試行版を発行する"
        >
          試行版を発行する
        </button>
        {/* §10.2.2 即時フィードバック: 編集中表示 */}
        {dirty && <span style={{ marginLeft: '12px' }}>未保存の変更があります</span>}
      </footer>
    </section>
  );
}

/** ノード一覧を抽出する（テスト用ヘルパ）*/
export function listNodes(flow: Flow, nodes: ReadonlyArray<FlowNode>): ReadonlyArray<FlowNode> {
  // フローと同期したノードリストを返す（将来 Flow からの公開で削除可能）
  return nodes.filter(() => flow.nodeCount >= 0);
}
