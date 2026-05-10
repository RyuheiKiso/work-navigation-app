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
import { t } from '../../i18n';
import { palette, radius, fontSize, fontWeight, space, fontStack } from '../../tokens/access';

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
    { key: 'flow', label: t('shell.tab_flow'), icon: '📐' },
    { key: 'products', label: t('shell.tab_products'), icon: '📦' },
    { key: 'equipments', label: t('shell.tab_equipments'), icon: '🏭' },
    { key: 'parts', label: t('shell.tab_parts'), icon: '🔧' },
    { key: 'audit', label: t('shell.tab_audit'), icon: '🛡️' },
    { key: 'dashboard', label: t('shell.tab_dashboard'), icon: '📊' }
  ];

  return (
    <div style={{ display: 'grid', gridTemplateColumns: '200px 1fr', height: '100vh', fontFamily: fontStack }}>
      <nav style={{ background: palette.neutral[800], color: palette.white, padding: space[4], display: 'flex', flexDirection: 'column' }}>
        <h2 style={{ fontSize: fontSize.caption, color: palette.neutral[400], marginTop: 0 }}>work-navigation-app</h2>
        <p style={{ fontSize: fontSize.caption, color: palette.neutral[400] }}>
          👤 {props.user.display_name}<br />
          <small>{props.user.user_id}</small>
        </p>
        <div style={{ marginTop: space[4], display: 'grid', gap: space[1] }}>
          {tabs.map((tb) => {
            // active 状態は背景濃淡＋左端アクセントボーダーで二重表現 (§1.11 形+色)
            const active = tab === tb.key;
            return (
              <button
                key={tb.key}
                type="button"
                aria-current={active ? 'page' : undefined}
                onClick={() => setTab(tb.key)}
                style={{
                  padding: `${space[2]} ${space[3]}`,
                  textAlign: 'left',
                  background: active ? palette.neutral[700] : 'transparent',
                  color: palette.white,
                  border: `1px solid ${active ? palette.neutral[600] : 'transparent'}`,
                  borderLeft: `4px solid ${active ? palette.brand.default : 'transparent'}`,
                  borderRadius: radius.small,
                  cursor: 'pointer',
                  fontSize: fontSize.caption,
                  fontWeight: active ? fontWeight.semibold : fontWeight.regular
                }}
              >
                {tb.icon} {tb.label}
              </button>
            );
          })}
        </div>
        <div style={{ marginTop: 'auto' }}>
          <button
            type="button"
            onClick={() => { logout(); props.onLogout(); }}
            style={{
              width: '100%', padding: space[2], fontSize: fontSize.caption, background: 'transparent',
              color: palette.white, border: `1px solid ${palette.neutral[600]}`, borderRadius: radius.small, cursor: 'pointer'
            }}
          >
            {t('shell.logout')}
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
