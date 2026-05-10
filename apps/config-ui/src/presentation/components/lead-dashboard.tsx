// 対応 §: ロードマップ §3.2.1.2 §3.6 §10.1
// 班長監視ダッシュボード: 全 Task の状態を一覧表示。

import { useEffect, useState } from 'react';
import { listDashboardTasks, type DashboardTask } from '../../adapter/api-client';

export function LeadDashboard(): JSX.Element {
  const [tasks, setTasks] = useState<DashboardTask[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [auto, setAuto] = useState(true);

  async function refresh(): Promise<void> {
    setError(null);
    try {
      setTasks(await listDashboardTasks());
    } catch (e) {
      setError((e as Error).message);
    }
  }

  useEffect(() => {
    void refresh();
    if (!auto) return;
    const t = setInterval(() => void refresh(), 5000);
    return () => clearInterval(t);
  }, [auto]);

  // 状態ごとに集計
  const counts = tasks.reduce<Record<string, number>>((acc, t) => {
    acc[t.state] = (acc[t.state] ?? 0) + 1;
    return acc;
  }, {});

  const stateColor = (s: string): string => {
    switch (s) {
      case 'Idle': return '#6C757D';
      case 'Ready': return '#17A2B8';
      case 'Running': return '#28A745';
      case 'Suspended': return '#FFC107';
      case 'Exception': return '#FD7E14';
      case 'Completed': return '#0C5460';
      case 'Failed':
      case 'Aborted': return '#DC3545';
      default: return '#212529';
    }
  };

  return (
    <div style={{ padding: 24 }}>
      <h1>📊 班長監視ダッシュボード</h1>
      <p style={{ color: '#6C757D' }}>5 秒ごとに自動更新（§3.2.1.2 認知負荷を抑える段階的開示）</p>

      {error && <div style={{ padding: 8, background: '#F8D7DA', color: '#721C24', borderRadius: 4, marginBottom: 12 }} role="alert">{error}</div>}

      <section style={{ display: 'flex', gap: 8, marginBottom: 16 }}>
        <button type="button" onClick={() => setAuto(!auto)} style={{ padding: '8px 16px', background: auto ? '#28A745' : '#6C757D', color: '#FFFFFF', border: 'none', borderRadius: 6, cursor: 'pointer' }}>
          {auto ? '⏸ 自動更新 ON' : '▶ 自動更新 OFF'}
        </button>
        <button type="button" onClick={() => void refresh()} style={{ padding: '8px 16px', background: '#17A2B8', color: '#FFFFFF', border: 'none', borderRadius: 6, cursor: 'pointer' }}>
          🔄 今すぐ更新
        </button>
      </section>

      {/* 状態サマリ */}
      <section style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(140px, 1fr))', gap: 12, marginBottom: 16 }}>
        {Object.entries(counts).map(([s, n]) => (
          <div key={s} style={{ background: '#FFFFFF', padding: 12, borderRadius: 8, borderTop: `4px solid ${stateColor(s)}`, boxShadow: '0 1px 3px rgba(13,17,23,0.10)' }}>
            <div style={{ fontSize: 12, color: '#6C757D' }}>{s}</div>
            <div style={{ fontSize: 28, fontWeight: 700, color: stateColor(s) }}>{n}</div>
          </div>
        ))}
      </section>

      {/* 詳細リスト */}
      <section style={{ background: '#FFFFFF', padding: 16, borderRadius: 8, boxShadow: '0 1px 3px rgba(13,17,23,0.10)' }}>
        <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: 13 }}>
          <thead>
            <tr style={{ background: '#F8F9FA' }}>
              <th style={{ padding: 8, textAlign: 'left', borderBottom: '1px solid #DEE2E6' }}>タスク</th>
              <th style={{ padding: 8, textAlign: 'left', borderBottom: '1px solid #DEE2E6' }}>状態</th>
              <th style={{ padding: 8, textAlign: 'left', borderBottom: '1px solid #DEE2E6' }}>端末</th>
              <th style={{ padding: 8, textAlign: 'left', borderBottom: '1px solid #DEE2E6' }}>担当</th>
              <th style={{ padding: 8, textAlign: 'left', borderBottom: '1px solid #DEE2E6' }}>現在ステップ</th>
              <th style={{ padding: 8, textAlign: 'left', borderBottom: '1px solid #DEE2E6' }}>更新</th>
            </tr>
          </thead>
          <tbody>
            {tasks.map((t) => (
              <tr key={t.id}>
                <td style={{ padding: 6, borderBottom: '1px solid #F1F3F5' }}>{t.title ?? t.id}</td>
                <td style={{ padding: 6, borderBottom: '1px solid #F1F3F5' }}>
                  <span style={{ display: 'inline-block', padding: '2px 8px', background: stateColor(t.state), color: '#FFFFFF', borderRadius: 4, fontSize: 12 }}>
                    {t.state}
                  </span>
                </td>
                <td style={{ padding: 6, borderBottom: '1px solid #F1F3F5' }}><code>{t.device_id}</code></td>
                <td style={{ padding: 6, borderBottom: '1px solid #F1F3F5' }}>{t.responsible_user ?? '—'}</td>
                <td style={{ padding: 6, borderBottom: '1px solid #F1F3F5' }}>{t.current_step_id ?? '—'}</td>
                <td style={{ padding: 6, borderBottom: '1px solid #F1F3F5', whiteSpace: 'nowrap', color: '#6C757D' }}>
                  {new Date(t.updated_at).toLocaleString()}
                </td>
              </tr>
            ))}
            {tasks.length === 0 && (
              <tr><td colSpan={6} style={{ padding: 16, textAlign: 'center', color: '#6C757D' }}>タスク未登録</td></tr>
            )}
          </tbody>
        </table>
      </section>
    </div>
  );
}
