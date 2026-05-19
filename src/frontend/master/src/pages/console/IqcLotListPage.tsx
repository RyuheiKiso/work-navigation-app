import type React from 'react';
import { useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import { Box, Stack, TextField, MenuItem, Chip } from '@mui/material';
import { DataGrid, type GridColDef } from '@mui/x-data-grid';
import type { IncomingInspection, QcStatus } from '@wnav/shared/types';
import { api } from '@/api/client';
import { queryKeys } from '@/api/queryKeys';
import { PageHeader } from '@/components/PageHeader';

const QC_STATUSES: QcStatus[] = [
  'PENDING',
  'SAMPLING',
  'INSPECTING',
  'PASSED',
  'FAILED',
  'REJECTED',
  'CONDITIONAL_PASS',
  'SCREENING_REQUIRED',
  'SCRAPPED',
  'RETURNED_TO_VENDOR',
];

const STATUS_COLOR: Record<QcStatus, 'default' | 'warning' | 'success' | 'error' | 'info'> = {
  PENDING: 'default',
  SAMPLING: 'info',
  INSPECTING: 'info',
  PASSED: 'success',
  FAILED: 'error',
  REJECTED: 'error',
  CONDITIONAL_PASS: 'warning',
  SCREENING_REQUIRED: 'warning',
  SCRAPPED: 'error',
  RETURNED_TO_VENDOR: 'error',
};

// SCR-MC-012 受入ロット一覧（quality_admin/system_admin、FR-IQC-001）。
export function IqcLotListPage(): React.ReactElement {
  const [qcStatus, setQcStatus] = useState<string>('');

  const query = useQuery({
    queryKey: queryKeys.console.iqcLots({ qcStatus }),
    queryFn: async (): Promise<IncomingInspection[]> => {
      const params = new URLSearchParams();
      if (qcStatus) params.set('qc_status', qcStatus);
      const r = await api.getList<IncomingInspection>(`/iqc/incoming-inspections?${params.toString()}`);
      return r.data;
    },
  });

  const columns: GridColDef<IncomingInspection>[] = [
    { field: 'id', headerName: '検査 ID', width: 280 },
    { field: 'lotId', headerName: 'ロット ID', width: 220 },
    { field: 'supplierId', headerName: '仕入先 ID', width: 220 },
    { field: 'materialId', headerName: '材料 ID', width: 220 },
    { field: 'receivedQty', headerName: '受入数量', width: 120 },
    {
      field: 'qcStatus',
      headerName: 'QC 状態',
      width: 200,
      renderCell: ({ value }) => (
        <Chip label={String(value)} size="small" color={STATUS_COLOR[value as QcStatus] ?? 'default'} />
      ),
    },
    { field: 'defectCount', headerName: '不良数', width: 100 },
    { field: 'judgedAt', headerName: '判定日時', width: 200 },
  ];

  return (
    <Box>
      <PageHeader title="受入ロット一覧" subtitle="qc_status による絞り込み" />
      <Stack direction="row" spacing={2} mb={2}>
        <TextField
          select
          label="QC 状態フィルタ"
          value={qcStatus}
          onChange={(e) => setQcStatus(e.target.value)}
          sx={{ minWidth: 220 }}
          inputProps={{ 'aria-label': 'QC 状態フィルタ' }}
        >
          <MenuItem value="">すべて</MenuItem>
          {QC_STATUSES.map((s) => (
            <MenuItem key={s} value={s}>
              {s}
            </MenuItem>
          ))}
        </TextField>
      </Stack>
      <Box sx={{ width: '100%' }}>
        <DataGrid
          rows={query.data ?? []}
          columns={columns}
          loading={query.isLoading}
          getRowId={(r) => r.id}
          autoHeight
          pageSizeOptions={[25, 50, 100]}
          aria-label="受入ロット一覧テーブル"
        />
      </Box>
    </Box>
  );
}
