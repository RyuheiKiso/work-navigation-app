import type React from 'react';
import { TextField } from '@mui/material';
import type { GridColDef } from '@mui/x-data-grid';
import type { Product } from '@wnav/shared/types';
import { resolveLocale } from '@wnav/shared/i18n';
import { GenericMasterListPage } from '@/components/GenericMasterListPage';
import { LocalizedTextField } from '@/components/LocalizedTextField';
import { queryKeys } from '@/api/queryKeys';

interface ProductForm {
  productCode: string;
  nameJa: string;
  nameEn: string;
  nameZh: string;
}

// SCR-MA-003 製品一覧。製品マスタの CRUD（FR-MA-003）。
export function ProductListPage(): React.ReactElement {
  const columns: GridColDef<Product>[] = [
    { field: 'productCode', headerName: '製品コード', width: 200 },
    { field: 'nameJson', headerName: '名称', flex: 1, valueGetter: (_, row) => resolveLocale(row.nameJson, 'ja') },
  ];

  return (
    <GenericMasterListPage<Product, ProductForm>
      title="製品一覧"
      subtitle="製品マスタの管理"
      endpoint="/master/products"
      queryKeyBuilder={(asOf) => queryKeys.master.products(asOf)}
      columns={columns}
      initialCreateForm={{ productCode: '', nameJa: '', nameEn: '', nameZh: '' }}
      labelOf={(item) => resolveLocale(item.nameJson, 'ja')}
      validateAndBuildPayload={(form) => {
        if (!form.productCode) return { ok: false, message: '製品コードは必須です' };
        if (form.productCode.length > 64) return { ok: false, message: '製品コードは 64 文字以内' };
        if (!form.nameJa) return { ok: false, message: '製品名称（ja）は必須です' };
        return {
          ok: true,
          payload: {
            productCode: form.productCode,
            nameJson: { ja: form.nameJa, en: form.nameEn, zh: form.nameZh },
          },
        };
      }}
      renderCreateForm={({ value, onChange }) => (
        <>
          <TextField
            label="製品コード"
            value={value.productCode}
            onChange={(e) => onChange({ ...value, productCode: e.target.value })}
            required
            inputProps={{ maxLength: 64, 'aria-label': '製品コード' }}
          />
          <LocalizedTextField
            label="製品名称"
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
