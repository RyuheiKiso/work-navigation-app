// 対応 §: ロードマップ §11.4.1 §16
// 監査ログ閲覧画面: 直近 N 件＋アクション／対象でのフィルタ。

import { useEffect, useState } from 'react';
import { listAudit, type AuditRow } from '../../adapter/api-client';
import { toApiError } from '../../adapter/api-error';
import { t } from '../../i18n';
import { LoadingState } from '../states/loading-state';
import { EmptyState } from '../states/empty-state';
import { ErrorPanel } from '../states/error-panel';

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
    <div style={{ padding: 24 }}>
      <h1>🛡️ 監査ログ</h1>
      <p style={{ color: '#6C757D' }}>
        §11.4.1 INV-07 によりこのログは追記不変。DB トリガで UPDATE／DELETE を物理拒否しています。
      </p>

      <section style={{ background: '#FFFFFF', padding: 16, borderRadius: 8, marginBottom: 16, display: 'flex', gap: 8, alignItems: 'center', boxShadow: '0 1px 3px rgba(13,17,23,0.10)' }}>
        <label style={{ flex: 1, fontSize: 13 }}>
          フィルタ（actor / action / target）
          <input value={filter} onChange={(e) => setFilter(e.target.value)} style={{ width: '100%', padding: 8, marginTop: 4 }} />
        </label>
        <button type="button" onClick={() => void refresh()} disabled={busy} style={{ minHeight: 36, padding: '8px 16px', background: '#17A2B8', color: '#FFFFFF', border: 'none', borderRadius: 6, alignSelf: 'flex-end' }}>
          {busy ? '...' : '🔄 更新'}
        </button>
      </section>

      {error && (
        <div style={{ marginBottom: 8 }}>
          <ErrorPanel
            message={error}
            onRetry={() => void refresh()}
            onDismiss={() => setError(null)}
          />
        </div>
      )}

      {!initialLoaded && busy && <LoadingState label={t('state_label.loading_audit')} />}

      <section style={{ background: '#FFFFFF', padding: 16, borderRadius: 8, boxShadow: '0 1px 3px rgba(13,17,23,0.10)' }}>
        <p style={{ marginTop: 0 }}>
          表示中: <strong>{filtered.length}</strong> 件 / 総取得 {rows.length} 件
        </p>
        <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: 13 }}>
          <thead>
            <tr style={{ background: '#F8F9FA' }}>
              <th style={{ padding: 6, textAlign: 'left', borderBottom: '1px solid #DEE2E6' }}>サーバ時刻</th>
              <th style={{ padding: 6, textAlign: 'left', borderBottom: '1px solid #DEE2E6' }}>端末時刻</th>
              <th style={{ padding: 6, textAlign: 'left', borderBottom: '1px solid #DEE2E6' }}>主体</th>
              <th style={{ padding: 6, textAlign: 'left', borderBottom: '1px solid #DEE2E6' }}>操作</th>
              <th style={{ padding: 6, textAlign: 'left', borderBottom: '1px solid #DEE2E6' }}>対象</th>
              <th style={{ padding: 6, textAlign: 'left', borderBottom: '1px solid #DEE2E6' }}>payload</th>
            </tr>
          </thead>
          <tbody>
            {filtered.map((r) => (
              <tr key={r.id}>
                <td style={{ padding: 6, borderBottom: '1px solid #F1F3F5', whiteSpace: 'nowrap' }}>
                  {new Date(r.server_time).toLocaleString()}
                </td>
                <td style={{ padding: 6, borderBottom: '1px solid #F1F3F5', whiteSpace: 'nowrap', color: '#6C757D' }}>
                  {r.terminal_time ? new Date(r.terminal_time).toLocaleString() : '—'}
                </td>
                <td style={{ padding: 6, borderBottom: '1px solid #F1F3F5' }}><code>{r.actor_id}</code></td>
                <td style={{ padding: 6, borderBottom: '1px solid #F1F3F5' }}><code>{r.action}</code></td>
                <td style={{ padding: 6, borderBottom: '1px solid #F1F3F5' }}>{r.target_id ?? '—'}</td>
                <td style={{ padding: 6, borderBottom: '1px solid #F1F3F5', maxWidth: 300, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
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
