// 対応 §: ロードマップ §3.6 §3.6.2 §3.6.4 §9.2 §9.5 §11.2 §10.4 §10.6
// ナビゲーションシェルの表示層。
// ドメイン状態と API 連携は `useTaskNavigation` フックに委譲する。

import { useState } from 'react';
import { t, getLocale, setLocale, isRtl, type LocaleKey } from '../../i18n';
import { logout } from '../../adapter/api-client';
import { useTaskNavigation } from '../hooks/use-task-navigation';
import { useOnlineStatus } from '../hooks/use-online-status';
import { ConfirmDialog } from './confirm-dialog';

export interface NavigationShellProps {
  user: { user_id: string; display_name: string };
  onLogout(): void;
}

export function NavigationShell(props: NavigationShellProps): JSX.Element {
  const [locale, setLocaleState] = useState<LocaleKey>(getLocale());
  const nav = useTaskNavigation();
  const online = useOnlineStatus();
  const [confirmingAndon, setConfirmingAndon] = useState(false);

  return (
    <div
      style={{
        display: 'grid',
        gridTemplateColumns: '260px 1fr 320px',
        gridTemplateRows: 'auto auto 1fr auto',
        gridTemplateAreas:
          '"breadcrumb breadcrumb breadcrumb" "tasklist progress andon" "tasklist main side" "actions actions actions"',
        height: '100vh',
        fontFamily: 'Inter, "Noto Sans JP", system-ui, sans-serif',
        background: nav.storage.status === 'blocked' ? '#F8D7DA' : '#F8F9FA'
      }}
      dir={isRtl(locale) ? 'rtl' : 'ltr'}
    >
      <header
        style={{
          gridArea: 'breadcrumb',
          padding: '12px 16px',
          background: '#FFFFFF',
          borderBottom: '1px solid #DEE2E6',
          display: 'flex',
          gap: 12,
          alignItems: 'center'
        }}
      >
        <strong>📍 {props.user.display_name}（{props.user.user_id}）</strong>
        <span style={{ color: '#6C757D' }}>
          | 状態: {nav.selectedTaskState} | タスク: {nav.selectedTaskId ?? '未選択'}
        </span>
        <span
          role="status"
          aria-label={t('network.aria_label')}
          aria-live="polite"
          style={{
            marginLeft: 'auto',
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
        <label>
          🌐{' '}
          <select
            value={locale}
            onChange={(e) => {
              const l = e.target.value as LocaleKey;
              setLocale(l);
              setLocaleState(l);
            }}
          >
            {(['ja', 'en', 'zh', 'ko', 'de', 'es', 'vi', 'th', 'id', 'fr', 'pt', 'ar', 'he'] as const).map((l) => (
              <option key={l} value={l}>
                {l.toUpperCase()}
              </option>
            ))}
          </select>
        </label>
        <button
          type="button"
          onClick={() => {
            logout();
            props.onLogout();
          }}
          style={{
            padding: '6px 12px',
            background: 'transparent',
            border: '1px solid #6C757D',
            borderRadius: 6,
            cursor: 'pointer'
          }}
        >
          ログアウト
        </button>
      </header>

      <aside
        style={{
          gridArea: 'tasklist',
          padding: 12,
          background: '#FFFFFF',
          borderRight: '1px solid #DEE2E6',
          overflowY: 'auto'
        }}
      >
        <h3 style={{ fontSize: 14, marginTop: 0 }}>📋 当日のタスク</h3>
        {nav.tasks.length === 0 && (
          <p style={{ fontSize: 12, color: '#6C757D' }}>タスクがありません</p>
        )}
        {nav.tasks.map((task) => (
          <button
            key={task.id}
            type="button"
            onClick={() => nav.selectTask(task.id, task.state)}
            style={{
              display: 'block',
              width: '100%',
              padding: 8,
              marginBottom: 6,
              textAlign: 'left',
              border: '1px solid #DEE2E6',
              borderRadius: 6,
              background: nav.selectedTaskId === task.id ? '#FFF3CD' : '#FFFFFF',
              cursor: 'pointer',
              fontSize: 13
            }}
          >
            <div><strong>{task.title ?? task.id}</strong></div>
            <div style={{ color: '#6C757D', fontSize: 11 }}>
              {task.state} | {task.device_id}
            </div>
          </button>
        ))}
      </aside>

      <section
        style={{
          gridArea: 'progress',
          padding: 16,
          background: '#FFFFFF',
          borderBottom: '1px solid #DEE2E6'
        }}
      >
        <div style={{ fontSize: 14, marginBottom: 8 }}>
          残 <strong style={{ fontSize: 24, color: '#0C5460' }}>{nav.remaining}</strong> ステップ
        </div>
        <div
          style={{ height: 16, background: '#E9ECEF', borderRadius: 8, overflow: 'hidden' }}
          role="progressbar"
          aria-valuenow={Math.round(nav.progress * 100)}
          aria-valuemin={0}
          aria-valuemax={100}
        >
          <div
            style={{
              width: `${nav.progress * 100}%`,
              height: '100%',
              background: '#28A745',
              transition: 'width 240ms cubic-bezier(0.2, 0.0, 0.0, 1.0)'
            }}
          />
        </div>
        <div style={{ fontSize: 12, marginTop: 4, color: '#6C757D' }}>
          進捗 {Math.round(nav.progress * 100)}% ／ 経過 {nav.elapsedSec}s ／ 標準 {nav.stdSec}s
          {nav.overrun && <span style={{ color: '#DC3545', marginLeft: 8 }}>⚠ 超過</span>}
        </div>
      </section>

      <section
        style={{
          gridArea: 'andon',
          padding: 16,
          background: nav.andon
            ? nav.andon.severity >= 4 ? '#DC3545' : nav.andon.severity >= 3 ? '#FFC107' : '#17A2B8'
            : '#D4EDDA',
          color: nav.andon && nav.andon.severity >= 4 ? '#FFFFFF' : '#155724',
          borderBottom: '1px solid #DEE2E6'
        }}
        aria-live="assertive"
      >
        {nav.andon ? (
          <div>
            <strong>🚨 アンドン Lv.{nav.andon.severity}</strong>
            <p style={{ margin: '4px 0' }}>{nav.andon.message}</p>
          </div>
        ) : (
          <div>✓ 異常なし</div>
        )}
      </section>

      <main style={{ gridArea: 'main', padding: 24, overflowY: 'auto' }}>
        {nav.error && (
          <div
            style={{ padding: 10, marginBottom: 12, background: '#F8D7DA', color: '#721C24', borderRadius: 6 }}
            role="alert"
          >
            ⚠ {nav.error}
          </div>
        )}
        {nav.selectedTaskState === 'Idle' || nav.selectedTaskState === 'Ready' ? (
          <div style={{ padding: 24, background: '#FFFFFF', borderRadius: 16 }}>
            <h2>タスクを開始してください</h2>
            <button
              type="button"
              onClick={() => void nav.doStartTask()}
              disabled={nav.busy || !nav.selectedTaskId}
              style={{
                minHeight: 64,
                padding: '16px 32px',
                fontSize: 18,
                background: '#28A745',
                color: '#FFFFFF',
                border: 'none',
                borderRadius: 12,
                cursor: 'pointer'
              }}
            >
              ▶ 開始
            </button>
          </div>
        ) : nav.current ? (
          <article
            style={{
              padding: 24,
              background: '#FFFFFF',
              borderRadius: 16,
              boxShadow: '0 4px 6px rgba(13,17,23,0.07)'
            }}
          >
            <h1 style={{ fontSize: 32, marginTop: 0 }}>{nav.current.label}</h1>
            <p style={{ fontSize: 18, color: '#6C757D' }}>
              完了条件: {t(`completion.${nav.current.completion_criteria}`)} | 標準時間: {nav.current.standard_time_seconds}s
            </p>
            <p style={{ marginTop: 24, padding: 12, background: '#FFF3CD', borderRadius: 8 }}>
              ➡ 次の動作: <strong>完了</strong>
            </p>
          </article>
        ) : nav.steps.length > 0 ? (
          <div style={{ padding: 24, fontSize: 24, color: '#28A745' }}>🎉 全ステップ完了</div>
        ) : (
          <div style={{ padding: 24, color: '#6C757D' }}>このタスクにはステップが設定されていません</div>
        )}
      </main>

      <aside
        style={{
          gridArea: 'side',
          padding: 16,
          background: '#FFFFFF',
          borderLeft: '1px solid #DEE2E6',
          overflowY: 'auto'
        }}
      >
        <h3 style={{ fontSize: 14, marginTop: 0 }}>ステップマップ</h3>
        <ol>
          {nav.steps.map((s, i) => (
            <li
              key={s.id}
              style={{
                padding: 6,
                background: i === nav.cursor ? '#FFF3CD' : 'transparent',
                color: s.done ? '#28A745' : i < nav.cursor ? '#ADB5BD' : '#212529'
              }}
            >
              {s.done ? '✓' : i === nav.cursor ? '●' : '○'} {s.label}
            </li>
          ))}
        </ol>

        <h3 style={{ fontSize: 14 }}>ストレージ</h3>
        <div
          style={{
            padding: 8,
            background: nav.storage.status === 'normal' ? '#D4EDDA' : nav.storage.status === 'warning' ? '#FFF3CD' : '#F8D7DA',
            borderRadius: 4
          }}
        >
          使用率 {Math.round(nav.storage.utilization * 100)}% — {nav.storage.status}
        </div>

        <h3 style={{ fontSize: 14 }}>音声コマンド</h3>
        <input
          ref={nav.voiceInputRef}
          placeholder="開始 / 完了 / 中断"
          style={{ width: '100%', padding: 6, marginBottom: 6 }}
          onKeyDown={(e) => {
            if (e.key === 'Enter') nav.handleVoiceCommand();
          }}
        />
        <button
          type="button"
          onClick={nav.handleVoiceCommand}
          style={{ minHeight: 36, width: '100%', background: '#17A2B8', color: '#FFFFFF', border: 'none', borderRadius: 6 }}
        >
          🎙 認識
        </button>
      </aside>

      <footer
        style={{
          gridArea: 'actions',
          padding: 16,
          background: '#FFFFFF',
          borderTop: '1px solid #DEE2E6',
          display: 'flex',
          gap: 12
        }}
      >
        <button
          type="button"
          onClick={() => void nav.doCompleteCurrent()}
          disabled={nav.busy || !nav.current || nav.selectedTaskState !== 'Running'}
          style={{
            minHeight: 64,
            flex: 2,
            background: !nav.current || nav.selectedTaskState !== 'Running' ? '#ADB5BD' : '#28A745',
            color: '#FFFFFF',
            border: 'none',
            borderRadius: 12,
            fontSize: 20,
            cursor: 'pointer'
          }}
        >
          ✓ 完了する
        </button>
        <button
          type="button"
          onClick={() => void (nav.andon ? nav.doResume() : nav.doSuspend())}
          disabled={nav.busy || !nav.selectedTaskId}
          style={{
            minHeight: 64,
            flex: 1,
            background: '#FFC107',
            color: '#212529',
            border: 'none',
            borderRadius: 12,
            fontSize: 18
          }}
        >
          {nav.andon ? '▶ 再開' : '⏸ 中断'}
        </button>
        <button
          type="button"
          onClick={() => setConfirmingAndon(true)}
          style={{
            minHeight: 64,
            flex: 1,
            background: '#DC3545',
            color: '#FFFFFF',
            border: 'none',
            borderRadius: 12,
            fontSize: 18
          }}
        >
          🚨 アンドン
        </button>
      </footer>
      <ConfirmDialog
        open={confirmingAndon}
        title={t('confirm.andon_title')}
        description={t('confirm.andon_description')}
        confirmLabel={t('confirm.andon_confirm')}
        cancelLabel={t('confirm.cancel')}
        variant="danger"
        onConfirm={() => {
          setConfirmingAndon(false);
          nav.fireAndon();
        }}
        onCancel={() => setConfirmingAndon(false)}
      />
    </div>
  );
}
