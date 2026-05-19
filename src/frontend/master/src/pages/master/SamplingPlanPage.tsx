import type React from 'react';
import { TextField, MenuItem } from '@mui/material';
import type { GridColDef } from '@mui/x-data-grid';
import type { SamplingPlan } from '@wnav/shared/types';
import { resolveLocale } from '@wnav/shared/i18n';
import { GenericMasterListPage } from '@/components/GenericMasterListPage';
import { LocalizedTextField } from '@/components/LocalizedTextField';
import { queryKeys } from '@/api/queryKeys';

interface SamplingPlanForm {
  planCode: string;
  aqlValue: number;
  inspectionLevel: 'I' | 'II' | 'III';
  nameJa: string;
  nameEn: string;
  nameZh: string;
}

// SCR-MA-014 サンプリング計画。AQL 値 + 検査水準（docs/05/05_WebAPP詳細設計/13_SamplingPlanEditor詳細設計.md）。
export function SamplingPlanPage(): React.ReactElement {
  const columns: GridColDef<SamplingPlan>[] = [
    { field: 'planCode', headerName: '計画コード', width: 180 },
    { field: 'nameJson', headerName: '名称', flex: 1, valueGetter: (_, row) => resolveLocale(row.nameJson, 'ja') },
    { field: 'aqlValue', headerName: 'AQL', width: 100 },
    { field: 'inspectionLevel', headerName: '検査水準', width: 120 },
  ];

  return (
    <GenericMasterListPage<SamplingPlan, SamplingPlanForm>
      title="サンプリング計画"
      subtitle="AQL 値・検査水準・JSONB スナップショット"
      endpoint="/master/sampling-plans"
      queryKeyBuilder={() => queryKeys.master.samplingPlans()}
      columns={columns}
      initialCreateForm={{ planCode: '', aqlValue: 0.65, inspectionLevel: 'II', nameJa: '', nameEn: '', nameZh: '' }}
      labelOf={(item) => resolveLocale(item.nameJson, 'ja')}
      validateAndBuildPayload={(form) => {
        if (!form.planCode) return { ok: false, message: '計画コードは必須です' };
        if (form.aqlValue <= 0 || form.aqlValue > 100) return { ok: false, message: 'AQL は 0 < x <= 100' };
        if (!form.nameJa) return { ok: false, message: '名称（ja）は必須です' };
        return {
          ok: true,
          payload: {
            planCode: form.planCode,
            aqlValue: form.aqlValue,
            inspectionLevel: form.inspectionLevel,
            nameJson: { ja: form.nameJa, en: form.nameEn, zh: form.nameZh },
            planSnapshot: JSON.stringify({ aql: form.aqlValue, level: form.inspectionLevel }),
          },
        };
      }}
      renderCreateForm={({ value, onChange }) => (
        <>
          <TextField
            label="計画コード"
            value={value.planCode}
            onChange={(e) => onChange({ ...value, planCode: e.target.value })}
            required
            inputProps={{ maxLength: 64, 'aria-label': '計画コード' }}
          />
          <TextField
            label="AQL"
            type="number"
            value={value.aqlValue}
            onChange={(e) => onChange({ ...value, aqlValue: Number(e.target.value) })}
            required
            inputProps={{ step: 0.01, min: 0.01, max: 100, 'aria-label': 'AQL 値' }}
          />
          <TextField
            select
            label="検査水準"
            value={value.inspectionLevel}
            onChange={(e) => onChange({ ...value, inspectionLevel: e.target.value as SamplingPlanForm['inspectionLevel'] })}
            inputProps={{ 'aria-label': '検査水準' }}
          >
            <MenuItem value="I">I</MenuItem>
            <MenuItem value="II">II</MenuItem>
            <MenuItem value="III">III</MenuItem>
          </TextField>
          <LocalizedTextField
            label="計画名称"
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
