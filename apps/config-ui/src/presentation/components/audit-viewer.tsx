// 対応 §: ロードマップ §11.4.1 §16
// 監査ログ閲覧画面: 直近 N 件＋アクション／対象でのフィルタ。

import { useEffect, useState } from 'react';
import { listAudit, type AuditRow } from '../../adapter/api-client';
import { toApiError } from '../../adapter/api-error';
import { t } from '../../i18n';
import { LoadingState } from '../states/loading-state';
import { EmptyState } from '../states/empty-state';
import { ErrorPanel } from '../states/error-panel';
import { palette, radius, fontSize, space, elevation } from '../../tokens/access';

export function AuditViewer(): JSX.Element {
  const [rows, setRows] = useState<AuditRow[]>([]);
  const [filter, setFilter] = useState('');
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [initialLoaded, setInitialLoaded] = useState(false);

  async function refresh(): Promise<void> {
    setBusy(true); setError(null);
    try {
      const r = await listAudit(200);
      setRows(r);
    } catch (e) {
      setError(t(toApiError(e).i18nKey()));
    } finally {
      setBusy(false);
      setInitialLoaded(true);
    }
  }

  useEffect(() => { void refresh(); }, []);

  const filtered = rows.filter((r) => {
    if (filter === '') return true;
    const f = filter.toLowerCase();
    return (
      r.actor_id.toLowerCase().includes(f) ||
      r.action.toLowerCase().includes(f) ||
      (r.target_id ?? '').toLowerCase().includes(f)
    );
  });

  return (
    <div style={{ padding: space[5], background: palette.bg, color: palette.fg }}>
      <h1>🛡️ 監査ログ</h1>
      <p style={{ color: palette.fgMuted }}>
        §11.4.1 INV-07 によりこのログは追記不変。DB トリガで UPDATE／DELETE を物理拒否しています。
      </p>

      <section style={{ background: palette.surface, padding: space[4], borderRadius: radius.medium, marginBottom: space[4], display: 'flex', gap: space[2], alignItems: 'center', boxShadow: elevation[1] }}>
        <label style={{ flex: 1, fontSize: fontSize.caption }}>
          フィルタ（actor / action / target）
          <input value={filter} onChange={(e) => setFilter(e.target.value)} style={{ width: '100%', padding: space[2], marginTop: space[1] }} />
        </label>
        <button type="button" onClick={() => void refresh()} disabled={busy} style={{ minHeight: '36px', padding: `${space[2]} ${space[4]}`, background: palette.info.default, color: palette.white, border: 'none', borderRadius: radius.small, alignSelf: 'flex-end', cursor: busy ? 'not-allowed' : 'pointer' }}>
          {busy ? '...' : '🔄 更新'}
        </button>
      </section>

      {error && (
        <div style={{ marginBottom: space[2] }}>
          <ErrorPanel
            message={error}
            onRetry={() => void refresh()}
            onDismiss={() => setError(null)}
          />
        </div>
      )}

      {!initialLoaded && busy && <LoadingState label={t('state_label.loading_audit')} />}

      <section style={{ background: palette.surface, padding: space[4], borderRadius: radius.medium, boxShadow: elevation[1] }}>
        <p style={{ marginTop: 0 }}>
          表示中: <strong>{filtered.length}</strong> 件 / 総取得 {rows.length} 件
        </p>
        <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: fontSize.caption }}>
          <thead>
            <tr style={{ background: palette.surfaceAlt }}>
              <th style={{ padding: space[1], textAlign: 'left', borderBottom: `1px solid ${palette.border}` }}>サーバ時刻</th>
              <th style={{ padding: space[1], textAlign: 'left', borderBottom: `1px solid ${palette.border}` }}>端末時刻</th>
              <th style={{ padding: space[1], textAlign: 'left', borderBottom: `1px solid ${palette.border}` }}>主体</th>
              <th style={{ padding: space[1], textAlign: 'left', borderBottom: `1px solid ${palette.border}` }}>操作</th>
              <th style={{ padding: space[1], textAlign: 'left', borderBottom: `1px solid ${palette.border}` }}>対象</th>
              <th style={{ padding: space[1], textAlign: 'left', borderBottom: `1px solid ${palette.border}` }}>payload</th>
            </tr>
          </thead>
          <tbody>
            {filtered.map((r) => (
              <tr key={r.id}>
                <td style={{ padding: space[1], borderBottom: `1px solid ${palette.neutral[100]}`, whiteSpace: 'nowrap' }}>
                  {new Date(r.server_time).toLocaleString()}
                </td>
                <td style={{ padding: space[1], borderBottom: `1px solid ${palette.neutral[100]}`, whiteSpace: 'nowrap', color: palette.fgMuted }}>
                  {r.terminal_time ? new Date(r.terminal_time).toLocaleString() : '—'}
                </td>
                <td style={{ padding: space[1], borderBottom: `1px solid ${palette.neutral[100]}` }}><code>{r.actor_id}</code></td>
                <td style={{ padding: space[1], borderBottom: `1px solid ${palette.neutral[100]}` }}><code>{r.action}</code></td>
                <td style={{ padding: space[1], borderBottom: `1px solid ${palette.neutral[100]}` }}>{r.target_id ?? '—'}</td>
                <td style={{ padding: space[1], borderBottom: `1px solid ${palette.neutral[100]}`, maxWidth: 300, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
                  <small>{r.payload ?? '—'}</small>
                </td>
              </tr>
            ))}
            {filtered.length === 0 && initialLoaded && !error && (
              <tr>
                <td colSpan={6} style={{ padding: 0 }}>
                  <EmptyState icon="🛡️" title={t('state_label.no_audit_title')} inline />
                </td>
              </tr>
            )}
          </tbody>
        </table>
      </section>
    </div>
  );
}
