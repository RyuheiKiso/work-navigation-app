import type React from 'react';
import { TextField } from '@mui/material';
import type { GridColDef } from '@mui/x-data-grid';
import type { Supplier } from '@wnav/shared/types';
import { resolveLocale } from '@wnav/shared/i18n';
import { GenericMasterListPage } from '@/components/GenericMasterListPage';
import { LocalizedTextField } from '@/components/LocalizedTextField';
import { queryKeys } from '@/api/queryKeys';

interface SupplierForm {
  supplierCode: string;
  contactEmail: string;
  nameJa: string;
  nameEn: string;
  nameZh: string;
}

// SCR-MA-013 仕入先マスタ（docs/05/05_WebAPP詳細設計/12_SupplierMasterEditor詳細設計.md）。
export function SupplierMasterPage(): React.ReactElement {
  const columns: GridColDef<Supplier>[] = [
    { field: 'supplierCode', headerName: '仕入先コード', width: 180 },
    { field: 'nameJson', headerName: '名称', flex: 1, valueGetter: (_, row) => resolveLocale(row.nameJson, 'ja') },
    { field: 'contactEmail', headerName: '連絡先メール', width: 240 },
  ];

  return (
    <GenericMasterListPage<Supplier, SupplierForm>
      title="仕入先マスタ"
      subtitle="仕入先の版管理"
      endpoint="/master/suppliers"
      queryKeyBuilder={(asOf) => queryKeys.master.suppliers(asOf)}
      columns={columns}
      initialCreateForm={{ supplierCode: '', contactEmail: '', nameJa: '', nameEn: '', nameZh: '' }}
      labelOf={(item) => resolveLocale(item.nameJson, 'ja')}
      validateAndBuildPayload={(form) => {
        if (!form.supplierCode) return { ok: false, message: '仕入先コードは必須です' };
        if (!form.nameJa) return { ok: false, message: '仕入先名称（ja）は必須です' };
        if (form.contactEmail && !/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(form.contactEmail)) {
          return { ok: false, message: 'メール形式が不正です' };
        }
        return {
          ok: true,
          payload: {
            supplierCode: form.supplierCode,
            contactEmail: form.contactEmail || null,
            nameJson: { ja: form.nameJa, en: form.nameEn, zh: form.nameZh },
          },
        };
      }}
      renderCreateForm={({ value, onChange }) => (
        <>
          <TextField
            label="仕入先コード"
            value={value.supplierCode}
            onChange={(e) => onChange({ ...value, supplierCode: e.target.value })}
            required
            inputProps={{ maxLength: 64, 'aria-label': '仕入先コード' }}
          />
          <TextField
            label="連絡先メール"
            type="email"
            value={value.contactEmail}
            onChange={(e) => onChange({ ...value, contactEmail: e.target.value })}
            inputProps={{ maxLength: 256, 'aria-label': '連絡先メール' }}
          />
          <LocalizedTextField
            label="仕入先名称"
            value={{ ja: value.nameJa, en: value.nameEn, zh: value.nameZh }}
            onChange={(v) => onChange({ ...value, nameJa: v.ja, nameEn: v.en, nameZh: v.zh })}
            required
            maxLength={200}
          />
        </>
      )}
    />
  );
}
