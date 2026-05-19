import type React from 'react';
import { useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import { Box, Stack, TextField, MenuItem, Chip } from '@mui/material';
import { DataGrid, type GridColDef } from '@mui/x-data-grid';
import type { Rework, ReworkStatus } from '@wnav/shared/types';
import { api } from '@/api/client';
import { queryKeys } from '@/api/queryKeys';
import { PageHeader } from '@/components/PageHeader';

const REWORK_STATUSES: ReworkStatus[] = ['OPEN', 'IN_PROGRESS', 'PENDING_VERIFICATION', 'CLOSED', 'SCRAPPED', 'RETURNED'];

const STATUS_COLOR: Record<ReworkStatus, 'default' | 'info' | 'warning' | 'success' | 'error'> = {
  OPEN: 'default',
  IN_PROGRESS: 'info',
  PENDING_VERIFICATION: 'warning',
  CLOSED: 'success',
  SCRAPPED: 'error',
  RETURNED: 'error',
};

// SCR-MC-014 リワーク一覧（quality_admin / supervisor、FR-RW-015）。
export function ReworkListPage(): React.ReactElement {
  const [status, setStatus] = useState<string>('');
  const [assignee, setAssignee] = useState('');

  const query = useQuery({
    queryKey: queryKeys.console.rework({ status, assignee }),
    queryFn: async (): Promise<Rework[]> => {
      const params = new URLSearchParams();
      if (status) params.set('status', status);
      if (assignee) params.set('assigned_to', assignee);
      const r = await api.getList<Rework>(`/reworks?${params.toString()}`);
      return r.data;
    },
  });

  const columns: GridColDef<Rework>[] = [
    { field: 'reworkCaseId', headerName: 'リワークケース ID', width: 280 },
    { field: 'parentCaseId', headerName: '元ケース ID', width: 280 },
    { field: 'sopId', headerName: 'SOP ID', width: 280 },
    { field: 'reworkCount', headerName: '回数', width: 80 },
    {
      field: 'status',
      headerName: '状態',
      width: 200,
      renderCell: ({ value }) => (
        <Chip label={String(value)} size="small" color={STATUS_COLOR[value as ReworkStatus] ?? 'default'} />
      ),
    },
    { field: 'deadline', headerName: '期限', width: 200 },
    { field: 'assignedTo', headerName: '担当者', width: 200 },
  ];

  return (
    <Box>
      <PageHeader title="リワーク一覧" subtitle="status / 担当者 / 期限で絞り込み" />
      <Stack direction="row" spacing={2} mb={2}>
        <TextField
          select
          label="状態フィルタ"
          value={status}
          onChange={(e) => setStatus(e.target.value)}
          sx={{ minWidth: 220 }}
          inputProps={{ 'aria-label': '状態フィルタ' }}
        >
          <MenuItem value="">すべて</MenuItem>
          {REWORK_STATUSES.map((s) => (
            <MenuItem key={s} value={s}>
              {s}
            </MenuItem>
          ))}
        </TextField>
        <TextField
          label="担当者 ID"
          value={assignee}
          onChange={(e) => setAssignee(e.target.value)}
          sx={{ minWidth: 280 }}
          inputProps={{ 'aria-label': '担当者 ID' }}
        />
      </Stack>
      <Box sx={{ width: '100%' }}>
        <DataGrid
          rows={query.data ?? []}
          columns={columns}
          loading={query.isLoading}
          getRowId={(r) => r.id}
          autoHeight
          pageSizeOptions={[25, 50, 100]}
          aria-label="リワーク一覧テーブル"
        />
      </Box>
    </Box>
  );
}
