// 対応 §: ロードマップ §10.2.1 §10.2.2 §3.4.1 §11.2 §9.5
// React Flow ベースのフローエディタ表示層。
// ドメイン状態と HSM 検証は `useFlowEditor` フックに委譲し、
// 試行版発行は `publishTrial` ユーティリティで API を叩く。
// 配色・余白・角丸・影は tokens/access 経由 — テーマ追従と表記の一貫性を強制する。

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
import {
  palette, radius, fontSize, fontWeight, lineHeight, space, elevation, fontStack
} from '../../tokens/access';

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
  if (status === 'error') return palette.danger.default;
  if (status === 'saving') return palette.fgMuted;
  return palette.success.default;
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
        fontFamily: fontStack,
        background: palette.bg,
        color: palette.fg
      }}
      dir={isRtl(locale) ? 'rtl' : 'ltr'}
    >
      <header
        style={{
          gridArea: 'toolbar',
          padding: `${space[3]} ${space[4]}`,
          borderBottom: `1px solid ${palette.border}`,
          background: palette.bg,
          display: 'flex',
          gap: space[3],
          alignItems: 'center'
        }}
      >
        <strong style={{ fontSize: fontSize.subtitle, fontWeight: fontWeight.semibold }}>
          {t('flow.title_prefix')}
          <input
            value={editor.flowName}
            onChange={(e) => editor.setFlowName(e.target.value)}
            style={{ fontSize: fontSize.body, padding: `${space[1]} ${space[2]}`, marginLeft: space[2] }}
            aria-label="フロー名"
          />
        </strong>
        <span style={{ color: palette.fgMuted }}>
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
            gap: space[1],
            padding: `${space[1]} ${space[3]}`,
            borderRadius: radius.pill,
            fontSize: fontSize.caption,
            background: online ? palette.success.subtle : palette.danger.subtle,
            color: online ? palette.success.strong : palette.danger.strong,
            border: `1px solid ${online ? palette.success.default : palette.danger.default}`
          }}
        >
          <span
            aria-hidden="true"
            style={{
              width: '8px',
              height: '8px',
              borderRadius: radius.pill,
              background: online ? palette.success.default : palette.danger.default
            }}
          />
          {online ? t('network.online') : t('network.offline')}
        </span>
        <label style={{ display: 'flex', gap: space[1], alignItems: 'center' }}>
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
          padding: space[3],
          borderRight: `1px solid ${palette.border}`,
          background: palette.surface,
          overflowY: 'auto'
        }}
      >
        <h3 style={{ fontSize: fontSize.caption, marginTop: 0 }}>ノード追加</h3>
        <div style={{ display: 'grid', gap: space[1] }}>
          {(['start', 'step', 'decision', 'parallel', 'end'] as const).map((k) => (
            <button
              key={k}
              type="button"
              onClick={() => editor.addNode(k)}
              style={{
                minHeight: '44px',
                padding: `${space[2]} ${space[3]}`,
                border: `1px solid ${palette.borderStrong}`,
                borderRadius: radius.medium,
                background: palette.surface,
                color: palette.fg,
                cursor: 'pointer',
                textAlign: 'left'
              }}
              aria-label={`${k} ノードを追加`}
            >
              + {k}
            </button>
          ))}
        </div>

        <h3 style={{ fontSize: fontSize.caption, marginTop: space[5] }}>業界テンプレ</h3>
        <div style={{ display: 'grid', gap: space[1] }}>
          {TEMPLATE_CATALOG.map(([label, path]) => (
            <button
              key={path}
              type="button"
              onClick={() => void handleLoadTemplate(path)}
              style={{
                minHeight: '36px',
                padding: `${space[1]} ${space[2]}`,
                border: `1px solid ${palette.info.default}`,
                borderRadius: radius.small,
                background: palette.info.subtle,
                color: palette.info.strong,
                cursor: 'pointer',
                textAlign: 'left',
                fontSize: fontSize.caption
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
              top: space[2],
              left: space[2],
              right: space[2],
              zIndex: 10,
              padding: `${space[2]} ${space[3]}`,
              borderRadius: radius.medium,
              background: palette.warning.subtle,
              color: palette.warning.strong,
              border: `1px solid ${palette.warning.default}`,
              display: 'flex',
              alignItems: 'center',
              gap: space[3],
              boxShadow: elevation[1]
            }}
          >
            <span style={{ flex: 1 }}>↩️ {t('setting_ui.draft_restored')}</span>
            <button
              type="button"
              onClick={() => editor.discardDraft()}
              style={{
                minHeight: '32px',
                padding: `${space[1]} ${space[3]}`,
                background: 'transparent',
                color: palette.warning.strong,
                border: `1px solid ${palette.warning.strong}`,
                borderRadius: radius.small,
                cursor: 'pointer',
                fontSize: fontSize.caption
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
          padding: space[3],
          borderLeft: `1px solid ${palette.border}`,
          background: palette.surface,
          overflowY: 'auto'
        }}
      >
        <h3 style={{ fontSize: fontSize.caption, marginTop: 0 }}>HSM 検証（§3.4.1）</h3>
        {editor.validation && (
          <div
            style={{
              padding: space[3],
              borderRadius: radius.medium,
              background: editor.validation.valid ? palette.success.subtle : palette.danger.subtle,
              color: editor.validation.valid ? palette.success.strong : palette.danger.strong,
              marginBottom: space[3]
            }}
          >
            {editor.validation.valid ? '✓ 全条件 OK' : '✗ 違反あり'}
          </div>
        )}
        {editor.validation && !editor.validation.valid && (
          <ul style={{ paddingLeft: space[4], fontSize: fontSize.caption }}>
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
            <h3 style={{ fontSize: fontSize.caption, marginTop: space[4] }}>ノード詳細</h3>
            <div style={{ fontSize: fontSize.caption, lineHeight: lineHeight.loose }}>
              <div><strong>ID:</strong> {editor.selectedNode.id}</div>
              <div><strong>種別:</strong> {editor.selectedNode.kind}</div>
              <div>
                <strong>ラベル:</strong>{' '}
                <input
                  value={editor.selectedNode.label}
                  onChange={(e) => editor.setNodeLabel(editor.selectedNode!.id, e.target.value)}
                  style={{ width: '100%', padding: `${space[1]} ${space[2]}` }}
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

        <div style={{ marginTop: space[5] }}>
          <button
            type="button"
            onClick={() => void handlePublishTrial()}
            disabled={!editor.validation?.valid}
            style={{
              minHeight: '44px',
              width: '100%',
              padding: space[3],
              background: editor.validation?.valid ? palette.brand.default : palette.neutral[400],
              color: palette.white,
              border: 'none',
              borderRadius: radius.medium,
              cursor: editor.validation?.valid ? 'pointer' : 'not-allowed',
              fontSize: fontSize.body,
              fontWeight: fontWeight.semibold
            }}
            aria-label={t('flow.aria_publish_trial')}
          >
            {t('flow.publish_trial_button')}
          </button>
          <button
            type="button"
            style={{
              minHeight: '36px',
              width: '100%',
              padding: space[2],
              marginTop: space[2],
              background: 'transparent',
              color: palette.info.strong,
              border: `1px solid ${palette.info.default}`,
              borderRadius: radius.small,
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
