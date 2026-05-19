import type React from 'react';
import { TextField } from '@mui/material';
import type { GridColDef } from '@mui/x-data-grid';
import type { Operation } from '@wnav/shared/types';
import { resolveLocale } from '@wnav/shared/i18n';
import { GenericMasterListPage } from '@/components/GenericMasterListPage';
import { LocalizedTextField } from '@/components/LocalizedTextField';
import { queryKeys } from '@/api/queryKeys';

interface OperationForm {
  operationCode: string;
  processId: string;
  nameJa: string;
  nameEn: string;
  nameZh: string;
}

// SCR-MA-002 オペレーション一覧。プロセスに紐付くオペレーションの CRUD（FR-MA-002）。
export function OperationListPage(): React.ReactElement {
  const columns: GridColDef<Operation>[] = [
    { field: 'operationCode', headerName: 'オペレーションコード', width: 200 },
    { field: 'nameJson', headerName: '名称', flex: 1, valueGetter: (_, row) => resolveLocale(row.nameJson, 'ja') },
    { field: 'processId', headerName: '親プロセス ID', width: 240 },
  ];

  return (
    <GenericMasterListPage<Operation, OperationForm>
      title="オペレーション一覧"
      subtitle="工程のオペレーションマスタ管理"
      endpoint="/master/operations"
      queryKeyBuilder={(asOf) => queryKeys.master.operations(asOf)}
      columns={columns}
      initialCreateForm={{ operationCode: '', processId: '', nameJa: '', nameEn: '', nameZh: '' }}
      labelOf={(item) => resolveLocale(item.nameJson, 'ja')}
      validateAndBuildPayload={(form) => {
        if (!form.operationCode) return { ok: false, message: 'オペレーションコードは必須です' };
        if (form.operationCode.length > 64) return { ok: false, message: 'オペレーションコードは 64 文字以内' };
        if (!form.nameJa) return { ok: false, message: 'オペレーション名称（ja）は必須です' };
        if (!form.processId) return { ok: false, message: '親プロセス ID は必須です' };
        return {
          ok: true,
          payload: {
            operationCode: form.operationCode,
            nameJson: { ja: form.nameJa, en: form.nameEn, zh: form.nameZh },
            processId: form.processId,
          },
        };
      }}
      renderCreateForm={({ value, onChange }) => (
        <>
          <TextField
            label="オペレーションコード"
            value={value.operationCode}
            onChange={(e) => onChange({ ...value, operationCode: e.target.value })}
            required
            inputProps={{ maxLength: 64, 'aria-label': 'オペレーションコード' }}
          />
          <TextField
            label="親プロセス ID"
            value={value.processId}
            onChange={(e) => onChange({ ...value, processId: e.target.value })}
            required
            inputProps={{ 'aria-label': '親プロセス ID' }}
          />
          <LocalizedTextField
            label="オペレーション名称"
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
