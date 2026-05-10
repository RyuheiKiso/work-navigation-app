// 対応 §: ロードマップ §10.2.1 §10.2.2 §3.4.1 §11.2 §9.5
// React Flow ベースのフローエディタ表示層。
// ドメイン状態と HSM 検証は `useFlowEditor` フックに委譲し、
// 試行版発行は `publishTrial` ユーティリティで API を叩く。

import { useEffect, useState } from 'react';
import ReactFlow, { Background, Controls, MiniMap } from 'reactflow';
import 'reactflow/dist/style.css';
import { type Flow } from '../../domain/flow';
import { t, getLocale, setLocale, type LocaleKey, isRtl, LOCALES } from '../../i18n';
import { useFlowEditor } from '../hooks/use-flow-editor';
import { publishTrial } from '../utils/publish-trial';
import type { AutosaveStatus } from '../hooks/use-autosave';
import { useOnlineStatus } from '../hooks/use-online-status';
import { showToast } from './toast';

const TEMPLATE_CATALOG: ReadonlyArray<readonly [string, string]> = [
  ['自動車：組立', '/templates/automotive/assembly-line.yaml'],
  ['自動車：SMED', '/templates/automotive/setup-changeover-smed.yaml'],
  ['自動車：QHold', '/templates/automotive/quality-hold.yaml'],
  ['医薬：バッチ記録', '/templates/pharma/batch-record.yaml'],
  ['医薬：変更管理', '/templates/pharma/change-control.yaml'],
  ['医薬：OOS 調査', '/templates/pharma/oos-investigation.yaml'],
  ['食品：加熱 CCP', '/templates/food/cooking-ccp.yaml'],
  ['食品：冷蔵監視', '/templates/food/cold-chain.yaml'],
  ['電子：SMT', '/templates/electronics/smt-assembly.yaml'],
  ['電子：CR 入退室', '/templates/electronics/cleanroom-entry.yaml']
];

const LOCALE_OPTIONS = Object.keys(LOCALES) as readonly LocaleKey[];

export interface FlowCanvasProps {
  initialFlow: Flow;
  onPublishTrial?: (flow: Flow) => void;
}

function autosaveLabel(status: AutosaveStatus, savedAt: number | null, now: number): string {
  if (status === 'saving') return t('setting_ui.autosave_saving');
  if (status === 'error') return t('setting_ui.autosave_failed');
  if (savedAt === null) return t('setting_ui.autosave_idle');
  const diffSec = Math.max(0, Math.floor((now - savedAt) / 1000));
  if (diffSec < 5) return t('setting_ui.autosave_just_now');
  if (diffSec < 60) return t('setting_ui.autosave_seconds_ago', { n: diffSec });
  const diffMin = Math.floor(diffSec / 60);
  return t('setting_ui.autosave_minutes_ago', { n: diffMin });
}

function autosaveColor(status: AutosaveStatus): string {
  if (status === 'error') return '#DC3545';
  if (status === 'saving') return '#6C757D';
  return '#28A745';
}

export function FlowCanvas(props: FlowCanvasProps): JSX.Element {
  const editor = useFlowEditor(props.initialFlow);
  const online = useOnlineStatus();
  const [locale, setLocaleState] = useState<LocaleKey>(getLocale());
  const [now, setNow] = useState<number>(() => Date.now());

  // 経過秒の見え方だけ刻むため軽量タイマー。状態には触らないので autosave への影響なし。
  useEffect(() => {
    const id = window.setInterval(() => setNow(Date.now()), 5000);
    return () => window.clearInterval(id);
  }, []);

  async function handleLoadTemplate(path: string): Promise<void> {
    try {
      await editor.loadTemplate(path);
    } catch (e) {
      // §20.1 エラーメッセージ規約: 人を責めない
      showToast('danger', `テンプレ読込に失敗しました（${(e as Error).message}）`);
    }
  }

  async function handlePublishTrial(): Promise<void> {
    if (!editor.validation?.valid) {
      showToast('warning', '検証エラーがあるため試行版は発行できません（§10.2.3 ブロック）');
      return;
    }
    if (!editor.currentFlow) return;
    try {
      const r = await publishTrial(editor.currentFlow);
      showToast(
        'success',
        `✓ 試行版発行完了\nflow=${r.flow_id} v${r.version}\nstatus=${r.status}\npilot=${r.pilot_device_ids.join(', ')}`
      );
      props.onPublishTrial?.(editor.currentFlow);
    } catch (e) {
      showToast('danger', `発行に失敗しました: ${(e as Error).message}`);
    }
  }

  return (
    <div
      style={{
        display: 'grid',
        gridTemplateColumns: '240px 1fr 280px',
        gridTemplateRows: 'auto 1fr',
        gridTemplateAreas: '"toolbar toolbar toolbar" "left canvas right"',
        height: '100vh',
        fontFamily: 'Inter, "Noto Sans JP", "Noto Sans KR", "Noto Sans SC", system-ui, sans-serif'
      }}
      dir={isRtl(locale) ? 'rtl' : 'ltr'}
    >
      <header
        style={{
          gridArea: 'toolbar',
          padding: '12px 16px',
          borderBottom: '1px solid #DEE2E6',
          background: '#F8F9FA',
          display: 'flex',
          gap: 12,
          alignItems: 'center'
        }}
      >
        <strong style={{ fontSize: 18 }}>
          {t('flow.title_prefix')}
          <input
            value={editor.flowName}
            onChange={(e) => editor.setFlowName(e.target.value)}
            style={{ fontSize: 16, padding: '4px 8px', marginLeft: 8 }}
            aria-label="フロー名"
          />
        </strong>
        <span style={{ color: '#6C757D' }}>
          {t('flow.version_label')} {props.initialFlow.version} ／{' '}
          {t('flow.nodes_label')} {editor.nodes.length} ／{' '}
          {t('flow.edges_label')} {editor.edges.length}
        </span>
        <span
          style={{ color: autosaveColor(editor.autosaveStatus), marginLeft: 'auto' }}
          role="status"
          aria-live="polite"
          aria-label={t('setting_ui.autosave_label')}
        >
          {autosaveLabel(editor.autosaveStatus, editor.lastSavedAt, now)}
        </span>
        <span
          role="status"
          aria-label={t('network.aria_label')}
          aria-live="polite"
          style={{
            display: 'inline-flex',
            alignItems: 'center',
            gap: 6,
            padding: '4px 10px',
            borderRadius: 999,
            fontSize: 12,
            background: online ? '#D4EDDA' : '#F8D7DA',
            color: online ? '#155724' : '#721C24',
            border: '1px solid ' + (online ? '#C3E6CB' : '#F5C6CB')
          }}
        >
          <span
            aria-hidden="true"
            style={{
              width: 8,
              height: 8,
              borderRadius: '50%',
              background: online ? '#28A745' : '#DC3545'
            }}
          />
          {online ? t('network.online') : t('network.offline')}
        </span>
        <label style={{ display: 'flex', gap: 4, alignItems: 'center' }}>
          🌐
          <select
            value={locale}
            onChange={(e) => {
              const l = e.target.value as LocaleKey;
              setLocale(l);
              setLocaleState(l);
            }}
          >
            {LOCALE_OPTIONS.map((l) => (
              <option key={l} value={l}>{l.toUpperCase()}</option>
            ))}
          </select>
        </label>
      </header>

      <aside
        style={{
          gridArea: 'left',
          padding: 12,
          borderRight: '1px solid #DEE2E6',
          background: '#FFFFFF',
          overflowY: 'auto'
        }}
      >
        <h3 style={{ fontSize: 14, marginTop: 0 }}>ノード追加</h3>
        <div style={{ display: 'grid', gap: 6 }}>
          {(['start', 'step', 'decision', 'parallel', 'end'] as const).map((k) => (
            <button
              key={k}
              type="button"
              onClick={() => editor.addNode(k)}
              style={{
                minHeight: 44,
                padding: '8px 12px',
                border: '1px solid #6C757D',
                borderRadius: 8,
                background: '#FFFFFF',
                cursor: 'pointer',
                textAlign: 'left'
              }}
              aria-label={`${k} ノードを追加`}
            >
              + {k}
            </button>
          ))}
        </div>

        <h3 style={{ fontSize: 14, marginTop: 24 }}>業界テンプレ</h3>
        <div style={{ display: 'grid', gap: 6 }}>
          {TEMPLATE_CATALOG.map(([label, path]) => (
            <button
              key={path}
              type="button"
              onClick={() => void handleLoadTemplate(path)}
              style={{
                minHeight: 36,
                padding: '6px 10px',
                border: '1px solid #17A2B8',
                borderRadius: 6,
                background: '#D1ECF1',
                cursor: 'pointer',
                textAlign: 'left',
                fontSize: 12
              }}
            >
              📄 {label}
            </button>
          ))}
        </div>
      </aside>

      <section style={{ gridArea: 'canvas', position: 'relative' }}>
        {editor.restoredFromDraft && (
          <div
            role="status"
            aria-live="polite"
            style={{
              position: 'absolute',
              top: 8,
              left: 8,
              right: 8,
              zIndex: 10,
              padding: '8px 12px',
              borderRadius: 8,
              background: '#FFF3CD',
              color: '#856404',
              border: '1px solid #FFEEBA',
              display: 'flex',
              alignItems: 'center',
              gap: 12,
              boxShadow: '0 2px 4px rgba(0,0,0,0.08)'
            }}
          >
            <span style={{ flex: 1 }}>↩️ {t('setting_ui.draft_restored')}</span>
            <button
              type="button"
              onClick={() => editor.discardDraft()}
              style={{
                minHeight: 32,
                padding: '4px 12px',
                background: 'transparent',
                color: '#856404',
                border: '1px solid #856404',
                borderRadius: 6,
                cursor: 'pointer',
                fontSize: 12
              }}
            >
              {t('setting_ui.discard_draft')}
            </button>
          </div>
        )}
        <ReactFlow
          nodes={editor.nodes}
          edges={editor.edges}
          onNodesChange={editor.onNodesChange}
          onEdgesChange={editor.onEdgesChange}
          onConnect={editor.onConnect}
          onNodeClick={(_, node) => editor.setSelectedNodeId(node.id)}
          onPaneClick={() => editor.setSelectedNodeId(null)}
          fitView
        >
          <MiniMap pannable zoomable />
          <Controls />
          <Background gap={16} />
        </ReactFlow>
      </section>

      <aside
        style={{
          gridArea: 'right',
          padding: 12,
          borderLeft: '1px solid #DEE2E6',
          background: '#FFFFFF',
          overflowY: 'auto'
        }}
      >
        <h3 style={{ fontSize: 14, marginTop: 0 }}>HSM 検証（§3.4.1）</h3>
        {editor.validation && (
          <div
            style={{
              padding: 12,
              borderRadius: 8,
              background: editor.validation.valid ? '#D4EDDA' : '#F8D7DA',
              color: editor.validation.valid ? '#155724' : '#721C24',
              marginBottom: 12
            }}
          >
            {editor.validation.valid ? '✓ 全条件 OK' : '✗ 違反あり'}
          </div>
        )}
        {editor.validation && !editor.validation.valid && (
          <ul style={{ paddingLeft: 18, fontSize: 13 }}>
            {editor.validation.unreachable.length > 0 && (
              <li>不到達: {editor.validation.unreachable.join(', ')}</li>
            )}
            {editor.validation.deadlocked.length > 0 && (
              <li>デッドロック: {editor.validation.deadlocked.join(', ')}</li>
            )}
            {editor.validation.cycles.length > 0 && (
              <li>サイクル: {editor.validation.cycles.length} 件</li>
            )}
            {editor.validation.orphanedEnds.length > 0 && (
              <li>孤児 end: {editor.validation.orphanedEnds.join(', ')}</li>
            )}
            {editor.validation.duplicateEdges.length > 0 && (
              <li>重複辺: {editor.validation.duplicateEdges.length} 件</li>
            )}
            {editor.validation.nondeterministicChoices.length > 0 && (
              <li>非決定的分岐: {editor.validation.nondeterministicChoices.join(', ')}</li>
            )}
          </ul>
        )}

        {editor.selectedNode && (
          <>
            <h3 style={{ fontSize: 14, marginTop: 16 }}>ノード詳細</h3>
            <div style={{ fontSize: 13, lineHeight: 1.8 }}>
              <div><strong>ID:</strong> {editor.selectedNode.id}</div>
              <div><strong>種別:</strong> {editor.selectedNode.kind}</div>
              <div>
                <strong>ラベル:</strong>{' '}
                <input
                  value={editor.selectedNode.label}
                  onChange={(e) => editor.setNodeLabel(editor.selectedNode!.id, e.target.value)}
                  style={{ width: '100%', padding: '4px 6px' }}
                />
              </div>
              <div>
                <strong>完了条件:</strong>{' '}
                <select
                  value={editor.selectedNode.completion ?? ''}
                  onChange={(e) =>
                    editor.setNodeCompletion(
                      editor.selectedNode!.id,
                      e.target.value === '' ? undefined : (e.target.value as 'manual' | 'photo')
                    )
                  }
                  style={{ width: '100%' }}
                >
                  <option value="">（未指定）</option>
                  <option value="manual">{t('completion.manual')}</option>
                  <option value="photo">{t('completion.photo')}</option>
                </select>
              </div>
            </div>
          </>
        )}

        <div style={{ marginTop: 24 }}>
          <button
            type="button"
            onClick={() => void handlePublishTrial()}
            disabled={!editor.validation?.valid}
            style={{
              minHeight: 44,
              width: '100%',
              padding: '10px',
              background: editor.validation?.valid ? '#28A745' : '#ADB5BD',
              color: '#FFFFFF',
              border: 'none',
              borderRadius: 8,
              cursor: editor.validation?.valid ? 'pointer' : 'not-allowed',
              fontSize: 14
            }}
            aria-label={t('flow.aria_publish_trial')}
          >
            {t('flow.publish_trial_button')}
          </button>
          <button
            type="button"
            style={{
              minHeight: 36,
              width: '100%',
              padding: '6px',
              marginTop: 8,
              background: 'transparent',
              color: '#0C5460',
              border: '1px solid #17A2B8',
              borderRadius: 6,
              cursor: 'pointer'
            }}
          >
            ↶ {t('setting_ui.rollback_link')}
          </button>
        </div>
      </aside>
    </div>
  );
}
