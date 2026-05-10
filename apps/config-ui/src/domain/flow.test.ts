// 対応 §: ロードマップ §13.1 §10.2.1
// Flow Aggregate の単体テスト。

import { describe, it, expect } from 'vitest';
import { Flow, type FlowNode, type FlowEdge } from './flow';

describe('Flow aggregate', () => {
  // 開始ノード必須
  it('rejects flow without start node', () => {
    // 開始ノードなしの定義
    const nodes: FlowNode[] = [{ id: 'n1', kind: 'step', label: 'ステップ' }];
    // 辺なし
    const edges: FlowEdge[] = [];
    // エラーを期待
    expect(() => Flow.create('f1', 'テスト', nodes, edges)).toThrow(/開始ノード/);
  });

  // ID 重複の検出
  it('rejects duplicate node ids', () => {
    // 重複 ID
    const nodes: FlowNode[] = [
      { id: 'n1', kind: 'start', label: '開始' },
      { id: 'n1', kind: 'step', label: 'ステップ' }
    ];
    // エラーを期待
    expect(() => Flow.create('f1', 'テスト', nodes, [])).toThrow(/重複/);
  });

  // 辺の参照整合性
  it('rejects edges that refer to missing nodes', () => {
    // ノード集合
    const nodes: FlowNode[] = [{ id: 'start', kind: 'start', label: '開始' }];
    // 存在しないノードを参照する辺
    const edges: FlowEdge[] = [{ from: 'start', to: 'missing' }];
    // エラーを期待
    expect(() => Flow.create('f1', 'テスト', nodes, edges)).toThrow(/to ノードが存在しません/);
  });

  // 妥当な構築
  it('creates a flow with valid inputs', () => {
    // 妥当ノード
    const nodes: FlowNode[] = [
      { id: 'start', kind: 'start', label: '開始' },
      { id: 'step1', kind: 'step', label: '手順 1' },
      { id: 'end', kind: 'end', label: '終了' }
    ];
    // 妥当な辺
    const edges: FlowEdge[] = [
      { from: 'start', to: 'step1' },
      { from: 'step1', to: 'end' }
    ];
    // 構築
    const f = Flow.create('f1', 'サンプル', nodes, edges);
    // ノード数
    expect(f.nodeCount).toBe(3);
    // 辺数
    expect(f.edgeCount).toBe(2);
    // バージョン初期値
    expect(f.version).toBe(1);
  });
});
