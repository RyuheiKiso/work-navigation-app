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
import { palette, radius, fontSize, fontWeight, space, elevation } from '../../tokens/access';

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
    <div style={{ padding: space[5], background: palette.bg, color: palette.fg }}>
      <h1>{m.title}</h1>

      <section style={{ background: palette.surface, padding: space[4], borderRadius: radius.medium, marginBottom: space[4], boxShadow: elevation[1] }}>
        <h3 style={{ marginTop: 0 }}>{t('master.new_or_update')}</h3>
        <form onSubmit={(e) => void handleSubmit(e)} style={{ display: 'grid', gridTemplateColumns: '1fr 1fr 1fr auto', gap: space[2], alignItems: 'end' }}>
          <label style={{ fontSize: fontSize.caption }}>
            {t('master.code_label')}
            <input value={code} onChange={(e) => setCode(e.target.value)} style={{ width: '100%', padding: space[2] }} />
          </label>
          <label style={{ fontSize: fontSize.caption }}>
            {t('master.name_label')}
            <input value={name} onChange={(e) => setName(e.target.value)} style={{ width: '100%', padding: space[2] }} />
          </label>
          <label style={{ fontSize: fontSize.caption }}>
            {m.extraLabel}
            <input value={extra} onChange={(e) => setExtra(e.target.value)} style={{ width: '100%', padding: space[2] }} />
          </label>
          <button type="submit" disabled={busy} style={{ minHeight: '36px', padding: `${space[2]} ${space[4]}`, background: palette.brand.default, color: palette.white, border: 'none', borderRadius: radius.small, fontWeight: fontWeight.semibold, cursor: busy ? 'not-allowed' : 'pointer' }}>
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

      <section style={{ background: palette.surface, padding: space[4], borderRadius: radius.medium, boxShadow: elevation[1] }}>
        <h3 style={{ marginTop: 0 }}>{t('master.registered_count', { n: rows.length })}</h3>
        <table style={{ width: '100%', borderCollapse: 'collapse' }}>
          <thead>
            <tr style={{ background: palette.surfaceAlt }}>
              <th style={{ padding: space[2], textAlign: 'left', borderBottom: `1px solid ${palette.border}` }}>{t('master.code_label')}</th>
              <th style={{ padding: space[2], textAlign: 'left', borderBottom: `1px solid ${palette.border}` }}>{t('master.name_label')}</th>
              <th style={{ padding: space[2], textAlign: 'left', borderBottom: `1px solid ${palette.border}` }}>{m.extraLabel}</th>
              <th style={{ padding: space[2], borderBottom: `1px solid ${palette.border}` }}>{t('master.edit')}/{t('master.delete')}</th>
            </tr>
          </thead>
          <tbody>
            {rows.map((r) => (
              <tr key={r.code}>
                <td style={{ padding: space[2], borderBottom: `1px solid ${palette.neutral[100]}` }}><code>{r.code}</code></td>
                <td style={{ padding: space[2], borderBottom: `1px solid ${palette.neutral[100]}` }}>{r.name}</td>
                <td style={{ padding: space[2], borderBottom: `1px solid ${palette.neutral[100]}`, color: palette.fgMuted }}>{r.extra ?? '—'}</td>
                <td style={{ padding: space[2], borderBottom: `1px solid ${palette.neutral[100]}`, textAlign: 'center' }}>
                  <button
                    type="button"
                    onClick={() => { setCode(r.code); setName(r.name); setExtra(r.extra ?? ''); }}
                    style={{ padding: `${space[1]} ${space[2]}`, marginRight: space[1], background: palette.info.default, color: palette.white, border: 'none', borderRadius: radius.small, cursor: 'pointer' }}
                  >
                    {t('master.edit')}
                  </button>
                  <button
                    type="button"
                    onClick={() => setConfirmingDelete(r.code)}
                    style={{ padding: `${space[1]} ${space[2]}`, background: palette.danger.default, color: palette.white, border: 'none', borderRadius: radius.small, cursor: 'pointer' }}
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
