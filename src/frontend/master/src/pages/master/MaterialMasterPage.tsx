import type React from 'react';
import { TextField } from '@mui/material';
import type { GridColDef } from '@mui/x-data-grid';
import type { Material } from '@wnav/shared/types';
import { resolveLocale } from '@wnav/shared/i18n';
import { GenericMasterListPage } from '@/components/GenericMasterListPage';
import { LocalizedTextField } from '@/components/LocalizedTextField';
import { queryKeys } from '@/api/queryKeys';

interface MaterialForm {
  materialCode: string;
  materialType: string;
  unit: string;
  nameJa: string;
  nameEn: string;
  nameZh: string;
}

// SCR-MA-012 材料マスタ（docs/05/05_WebAPP詳細設計/11_MaterialMasterEditor詳細設計.md）。
export function MaterialMasterPage(): React.ReactElement {
  const columns: GridColDef<Material>[] = [
    { field: 'materialCode', headerName: '材料コード', width: 180 },
    { field: 'nameJson', headerName: '名称', flex: 1, valueGetter: (_, row) => resolveLocale(row.nameJson, 'ja') },
    { field: 'materialType', headerName: '種別', width: 140 },
    { field: 'unit', headerName: '単位', width: 100 },
  ];

  return (
    <GenericMasterListPage<Material, MaterialForm>
      title="材料マスタ"
      subtitle="材料の版管理（material_code・material_type）"
      endpoint="/master/materials"
      queryKeyBuilder={(asOf) => queryKeys.master.materials(asOf)}
      columns={columns}
      initialCreateForm={{ materialCode: '', materialType: 'raw', unit: 'kg', nameJa: '', nameEn: '', nameZh: '' }}
      initialEditForm={(item) => ({
        materialCode: item.materialCode,
        materialType: item.materialType,
        unit: item.unit,
        nameJa: item.nameJson.ja,
        nameEn: item.nameJson.en,
        nameZh: item.nameJson.zh,
      })}
      labelOf={(item) => resolveLocale(item.nameJson, 'ja')}
      validateAndBuildPayload={(form) => {
        if (!form.materialCode) return { ok: false, message: '材料コードは必須です' };
        if (!form.nameJa) return { ok: false, message: '材料名称（ja）は必須です' };
        return {
          ok: true,
          payload: {
            materialCode: form.materialCode,
            materialType: form.materialType,
            unit: form.unit,
            nameJson: { ja: form.nameJa, en: form.nameEn, zh: form.nameZh },
          },
        };
      }}
      renderCreateForm={({ value, onChange }) => (
        <>
          <TextField
            label="材料コード"
            value={value.materialCode}
            onChange={(e) => onChange({ ...value, materialCode: e.target.value })}
            required
            inputProps={{ maxLength: 64, 'aria-label': '材料コード' }}
          />
          <TextField
            label="材料種別"
            value={value.materialType}
            onChange={(e) => onChange({ ...value, materialType: e.target.value })}
            inputProps={{ maxLength: 32, 'aria-label': '材料種別' }}
          />
          <TextField
            label="単位"
            value={value.unit}
            onChange={(e) => onChange({ ...value, unit: e.target.value })}
            inputProps={{ maxLength: 16, 'aria-label': '単位' }}
          />
          <LocalizedTextField
            label="材料名称"
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
