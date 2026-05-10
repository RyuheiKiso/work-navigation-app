// 対応 §: ロードマップ §10.2.1（マスタ編集） §10.3.6 RACI
// 製品／設備／部材の最小 CRUD 画面（汎用）。

import { useEffect, useState } from 'react';
import {
  listProducts, upsertProduct, deleteProduct,
  listEquipments, upsertEquipment, deleteEquipment,
  listParts, upsertPart, deletePart,
  type MasterRow
} from '../../adapter/api-client';
import { t } from '../../i18n';
import { toApiError } from '../../adapter/api-error';
import { ConfirmDialog } from './confirm-dialog';
import { LoadingState } from '../states/loading-state';
import { EmptyState } from '../states/empty-state';
import { ErrorPanel } from '../states/error-panel';

function localize(e: unknown): string {
  return t(toApiError(e).i18nKey());
}

export interface MasterEditorProps {
  kind: 'products' | 'equipments' | 'parts';
}

function meta(kind: MasterEditorProps['kind']): { title: string; extraLabel: string } {
  if (kind === 'products') return { title: t('master.products_title'), extraLabel: t('master.extra_products') };
  if (kind === 'equipments') return { title: t('master.equipments_title'), extraLabel: t('master.extra_equipments') };
  return { title: t('master.parts_title'), extraLabel: t('master.extra_parts') };
}

export function MasterEditor({ kind }: MasterEditorProps): JSX.Element {
  const [rows, setRows] = useState<MasterRow[]>([]);
  const [code, setCode] = useState('');
  const [name, setName] = useState('');
  const [extra, setExtra] = useState('');
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [confirmingDelete, setConfirmingDelete] = useState<string | null>(null);
  const [initialLoaded, setInitialLoaded] = useState(false);

  const m = meta(kind);

  const fetcher = async (): Promise<MasterRow[]> => {
    if (kind === 'products') return listProducts();
    if (kind === 'equipments') return listEquipments();
    return listParts();
  };
  const upserter = async (row: MasterRow): Promise<void> => {
    if (kind === 'products') return upsertProduct(row);
    if (kind === 'equipments') return upsertEquipment(row);
    return upsertPart(row);
  };
  const deleter = async (c: string): Promise<void> => {
    if (kind === 'products') return deleteProduct(c);
    if (kind === 'equipments') return deleteEquipment(c);
    return deletePart(c);
  };

  async function refresh(): Promise<void> {
    setError(null);
    try {
      setRows(await fetcher());
    } catch (e) {
      setError(localize(e));
    } finally {
      setInitialLoaded(true);
    }
  }

  useEffect(() => {
    setInitialLoaded(false);
    void refresh();
    /* eslint-disable-next-line */
  }, [kind]);

  async function handleSubmit(e: React.FormEvent): Promise<void> {
    e.preventDefault();
    if (!code.trim() || !name.trim()) {
      setError(t('master.code_and_name_required'));
      return;
    }
    setBusy(true); setError(null);
    try {
      await upserter({ code, name, extra: extra.trim() === '' ? null : extra });
      setCode(''); setName(''); setExtra('');
      await refresh();
    } catch (err) {
      setError(localize(err));
    } finally {
      setBusy(false);
    }
  }

  async function performDelete(c: string): Promise<void> {
    setBusy(true); setError(null);
    try { await deleter(c); await refresh(); } catch (e) { setError(localize(e)); } finally { setBusy(false); }
  }

  return (
    <div style={{ padding: 24 }}>
      <h1>{m.title}</h1>

      <section style={{ background: '#FFFFFF', padding: 16, borderRadius: 8, marginBottom: 16, boxShadow: '0 1px 3px rgba(13,17,23,0.10)' }}>
        <h3 style={{ marginTop: 0 }}>{t('master.new_or_update')}</h3>
        <form onSubmit={(e) => void handleSubmit(e)} style={{ display: 'grid', gridTemplateColumns: '1fr 1fr 1fr auto', gap: 8, alignItems: 'end' }}>
          <label style={{ fontSize: 13 }}>
            {t('master.code_label')}
            <input value={code} onChange={(e) => setCode(e.target.value)} style={{ width: '100%', padding: 8 }} />
          </label>
          <label style={{ fontSize: 13 }}>
            {t('master.name_label')}
            <input value={name} onChange={(e) => setName(e.target.value)} style={{ width: '100%', padding: 8 }} />
          </label>
          <label style={{ fontSize: 13 }}>
            {m.extraLabel}
            <input value={extra} onChange={(e) => setExtra(e.target.value)} style={{ width: '100%', padding: 8 }} />
          </label>
          <button type="submit" disabled={busy} style={{ minHeight: 36, padding: '8px 16px', background: '#28A745', color: '#FFFFFF', border: 'none', borderRadius: 6 }}>
            {t('master.save')}
          </button>
        </form>
        {error && (
          <div style={{ marginTop: 8 }}>
            <ErrorPanel
              message={error}
              onRetry={() => void refresh()}
              onDismiss={() => setError(null)}
            />
          </div>
        )}
      </section>

      {!initialLoaded && !error && <LoadingState label={t('state_label.loading_master')} />}

      <section style={{ background: '#FFFFFF', padding: 16, borderRadius: 8, boxShadow: '0 1px 3px rgba(13,17,23,0.10)' }}>
        <h3 style={{ marginTop: 0 }}>{t('master.registered_count', { n: rows.length })}</h3>
        <table style={{ width: '100%', borderCollapse: 'collapse' }}>
          <thead>
            <tr style={{ background: '#F8F9FA' }}>
              <th style={{ padding: 8, textAlign: 'left', borderBottom: '1px solid #DEE2E6' }}>{t('master.code_label')}</th>
              <th style={{ padding: 8, textAlign: 'left', borderBottom: '1px solid #DEE2E6' }}>{t('master.name_label')}</th>
              <th style={{ padding: 8, textAlign: 'left', borderBottom: '1px solid #DEE2E6' }}>{m.extraLabel}</th>
              <th style={{ padding: 8, borderBottom: '1px solid #DEE2E6' }}>{t('master.edit')}/{t('master.delete')}</th>
            </tr>
          </thead>
          <tbody>
            {rows.map((r) => (
              <tr key={r.code}>
                <td style={{ padding: 8, borderBottom: '1px solid #F1F3F5' }}><code>{r.code}</code></td>
                <td style={{ padding: 8, borderBottom: '1px solid #F1F3F5' }}>{r.name}</td>
                <td style={{ padding: 8, borderBottom: '1px solid #F1F3F5', color: '#6C757D' }}>{r.extra ?? '—'}</td>
                <td style={{ padding: 8, borderBottom: '1px solid #F1F3F5', textAlign: 'center' }}>
                  <button
                    type="button"
                    onClick={() => { setCode(r.code); setName(r.name); setExtra(r.extra ?? ''); }}
                    style={{ padding: '4px 8px', marginRight: 4, background: '#17A2B8', color: '#FFFFFF', border: 'none', borderRadius: 4, cursor: 'pointer' }}
                  >
                    {t('master.edit')}
                  </button>
                  <button
                    type="button"
                    onClick={() => setConfirmingDelete(r.code)}
                    style={{ padding: '4px 8px', background: '#DC3545', color: '#FFFFFF', border: 'none', borderRadius: 4, cursor: 'pointer' }}
                  >
                    {t('master.delete')}
                  </button>
                </td>
              </tr>
            ))}
            {rows.length === 0 && initialLoaded && !error && (
              <tr>
                <td colSpan={4} style={{ padding: 0 }}>
                  <EmptyState
                    icon="📦"
                    title={t('state_label.no_master_title')}
                    description={t('state_label.no_master_description')}
                    inline
                  />
                </td>
              </tr>
            )}
          </tbody>
        </table>
      </section>
      <ConfirmDialog
        open={confirmingDelete !== null}
        title={t('confirm.delete_title')}
        description={
          t('confirm.delete_description_prefix') +
          (confirmingDelete ?? '') +
          t('confirm.delete_description_suffix')
        }
        confirmLabel={t('confirm.delete_confirm')}
        cancelLabel={t('confirm.cancel')}
        variant="danger"
        onConfirm={() => {
          const code = confirmingDelete;
          setConfirmingDelete(null);
          if (code) void performDelete(code);
        }}
        onCancel={() => setConfirmingDelete(null)}
      />
    </div>
  );
}
