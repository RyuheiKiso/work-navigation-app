// 対応 §: ロードマップ §7.2 §10.2 §10.2.1 §10.2.2 §11.4.1 §3.6
// 設定 UI のアプリシェル: 左側メニュー + 中央コンテンツ。
// フローエディタ／マスタ／監査／班長ダッシュボード を切替表示。

import { useEffect, useMemo, useState } from 'react';
import { Flow, type FlowEdge, type FlowNode } from '../../domain/flow';
import { FlowCanvas } from './flow-canvas';
import { MasterEditor } from './master-editor';
import { AuditViewer } from './audit-viewer';
import { LeadDashboard } from './lead-dashboard';
import { logout, type DashboardTask } from '../../adapter/api-client';

export interface AppShellProps {
  user: { user_id: string; display_name: string };
  onLogout(): void;
}

type Tab = 'flow' | 'products' | 'equipments' | 'parts' | 'audit' | 'dashboard';

export function AppShell(props: AppShellProps): JSX.Element {
  const [tab, setTab] = useState<Tab>('flow');

  const sampleFlow = useMemo<Flow>(() => {
    const nodes: FlowNode[] = [
      { id: 'start', kind: 'start', label: '着手' },
      { id: 'inspect', kind: 'step', label: '部材確認', completion_criteria: 'photo' },
      { id: 'assemble', kind: 'step', label: '組立', completion_criteria: 'manual' },
      { id: 'check', kind: 'step', label: '完了検査', completion_criteria: 'photo' },
      { id: 'end', kind: 'end', label: '完了' }
    ];
    const edges: FlowEdge[] = [
      { from: 'start', to: 'inspect' },
      { from: 'inspect', to: 'assemble' },
      { from: 'assemble', to: 'check' },
      { from: 'check', to: 'end' }
    ];
    return Flow.create('sample-flow', 'サンプル組立フロー', nodes, edges, undefined, 1);
  }, []);

  const tabs: { key: Tab; label: string; icon: string }[] = [
    { key: 'flow', label: 'フロー編集', icon: '📐' },
    { key: 'products', label: '製品', icon: '📦' },
    { key: 'equipments', label: '設備', icon: '🏭' },
    { key: 'parts', label: '部材', icon: '🔧' },
    { key: 'audit', label: '監査ログ', icon: '🛡️' },
    { key: 'dashboard', label: '班長監視', icon: '📊' }
  ];

  return (
    <div style={{ display: 'grid', gridTemplateColumns: '200px 1fr', height: '100vh', fontFamily: 'Inter, "Noto Sans JP", system-ui, sans-serif' }}>
      <nav style={{ background: '#212529', color: '#FFFFFF', padding: 16, display: 'flex', flexDirection: 'column' }}>
        <h2 style={{ fontSize: 14, color: '#ADB5BD', marginTop: 0 }}>work-navigation-app</h2>
        <p style={{ fontSize: 12, color: '#ADB5BD' }}>
          👤 {props.user.display_name}<br />
          <small>{props.user.user_id}</small>
        </p>
        <div style={{ marginTop: 16, display: 'grid', gap: 4 }}>
          {tabs.map((tb) => (
            <button
              key={tb.key}
              type="button"
              onClick={() => setTab(tb.key)}
              style={{
                padding: '10px 12px',
                textAlign: 'left',
                background: tab === tb.key ? '#28A745' : 'transparent',
                color: '#FFFFFF',
                border: '1px solid ' + (tab === tb.key ? '#28A745' : '#495057'),
                borderRadius: 6,
                cursor: 'pointer',
                fontSize: 14
              }}
            >
              {tb.icon} {tb.label}
            </button>
          ))}
        </div>
        <div style={{ marginTop: 'auto' }}>
          <button
            type="button"
            onClick={() => { logout(); props.onLogout(); }}
            style={{
              width: '100%', padding: 8, fontSize: 13, background: 'transparent',
              color: '#FFFFFF', border: '1px solid #6C757D', borderRadius: 6, cursor: 'pointer'
            }}
          >
            ログアウト
          </button>
        </div>
      </nav>

      <section style={{ overflow: 'auto' }}>
        {tab === 'flow' && (
          <FlowCanvas
            initialFlow={sampleFlow}
            onPublishTrial={(_f) => { /* FlowCanvas 内で実 HTTP を叩く */ }}
          />
        )}
        {tab === 'products' && <MasterEditor kind="products" />}
        {tab === 'equipments' && <MasterEditor kind="equipments" />}
        {tab === 'parts' && <MasterEditor kind="parts" />}
        {tab === 'audit' && <AuditViewer />}
        {tab === 'dashboard' && <LeadDashboard />}
      </section>
    </div>
  );
}

// 子コンポーネントから参照させたい型を再エクスポート
export type { DashboardTask };
