import type React from 'react';
import { useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import {
  Box,
  Stack,
  TextField,
  MenuItem,
  Button,
  Paper,
  Typography,
  Chip,
} from '@mui/material';
import { DataGrid, type GridColDef } from '@mui/x-data-grid';
import { DateTimePicker } from '@mui/x-date-pickers/DateTimePicker';
import dayjs, { type Dayjs } from 'dayjs';
import { Download } from '@mui/icons-material';
import type { WorkEvent, ActivityType } from '@wnav/shared/types';
import { api } from '@/api/client';
import { queryKeys } from '@/api/queryKeys';
import { PageHeader } from '@/components/PageHeader';
import { UnSyncedBadge } from '@/components/UnSyncedBadge';
import { MasterTimeMachine } from '@/audit/AsOfControl';

const ACTIVITY_TYPES: ActivityType[] = [
  'step_completed',
  'step_skipped',
  'evidence_attached',
  'sign_applied',
  'measurement_recorded',
  'work_started',
  'work_completed',
  'andon_raised',
];

// SCR-MC-004 監査ログ閲覧（quality_admin/system_admin、FR-AU-004）。
// 時点参照固定 + 未同期バッジ表示（src/CLAUDE.md §Local-First とサーバー受信時刻）。
export function AuditLogPage(): React.ReactElement {
  const [from, setFrom] = useState<Dayjs | null>(dayjs().subtract(7, 'day'));
  const [to, setTo] = useState<Dayjs | null>(dayjs());
  const [activityType, setActivityType] = useState<string>('');
  const [caseId, setCaseId] = useState('');
  const [asOfUtc, setAsOfUtc] = useState<string | null>(null);

  const filters = {
    from: from?.toISOString() ?? '',
    to: to?.toISOString() ?? '',
    activityType,
    caseId,
    asOfUtc: asOfUtc ?? '',
  };

  const query = useQuery({
    queryKey: queryKeys.console.auditLogs(filters),
    queryFn: async (): Promise<WorkEvent[]> => {
      const params = new URLSearchParams();
      if (from) params.set('from', from.toISOString());
      if (to) params.set('to', to.toISOString());
      if (activityType) params.set('activity', activityType);
      if (caseId) params.set('case_id', caseId);
      if (asOfUtc) params.set('as_of', asOfUtc);
      const r = await api.getList<WorkEvent>(`/audit-logs?${params.toString()}`);
      return r.data;
    },
  });

  const columns: GridColDef<WorkEvent>[] = [
    { field: 'timestampServer', headerName: 'サーバ受信日時', width: 200 },
    { field: 'timestampClient', headerName: '端末記録日時', width: 200 },
    { field: 'caseId', headerName: 'ケース ID', width: 280 },
    {
      field: 'activity',
      headerName: 'アクティビティ',
      width: 180,
      renderCell: ({ value }) => <Chip label={String(value)} size="small" />,
    },
    { field: 'resource', headerName: 'リソース', width: 200 },
    {
      field: 'synced',
      headerName: '同期',
      width: 200,
      renderCell: ({ row }) => (
        <UnSyncedBadge
          serverReceivedAt={row.timestampServer}
          clientRecordedAt={row.timestampClient}
        />
      ),
    },
  ];

  const exportCsv = async (): Promise<void> => {
    const rows = query.data ?? [];
    const csv = [
      ['eventId', 'caseId', 'activity', 'timestampServer', 'timestampClient', 'resource'].join(','),
      ...rows.map((r) => [r.eventId, r.caseId, r.activity, r.timestampServer ?? '', r.timestampClient, r.resource].join(',')),
    ].join('\n');
    const blob = new Blob([csv], { type: 'text/csv' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `audit-logs-${dayjs().format('YYYYMMDDHHmmss')}.csv`;
    a.click();
    URL.revokeObjectURL(url);
  };

  return (
    <Box>
      <PageHeader title="監査ログ閲覧" subtitle="XES 互換イベントの時点参照（FR-AU-004）" />
      <MasterTimeMachine onAsOfChange={setAsOfUtc} />
      <Paper sx={{ p: 2, mb: 2 }} elevation={1}>
        <Stack direction="row" spacing={2} flexWrap="wrap" useFlexGap>
          <DateTimePicker label="開始日時" value={from} onChange={setFrom} ampm={false} />
          <DateTimePicker label="終了日時" value={to} onChange={setTo} ampm={false} />
          <TextField
            select
            label="アクティビティ"
            value={activityType}
            onChange={(e) => setActivityType(e.target.value)}
            sx={{ minWidth: 200 }}
            inputProps={{ 'aria-label': 'アクティビティ種別' }}
          >
            <MenuItem value="">すべて</MenuItem>
            {ACTIVITY_TYPES.map((a) => (
              <MenuItem key={a} value={a}>
                {a}
              </MenuItem>
            ))}
          </TextField>
          <TextField
            label="ケース ID"
            value={caseId}
            onChange={(e) => setCaseId(e.target.value)}
            sx={{ minWidth: 280 }}
            inputProps={{ 'aria-label': 'ケース ID' }}
          />
          <Button variant="contained" onClick={() => query.refetch()} aria-label="検索">
            検索
          </Button>
          <Button startIcon={<Download />} onClick={() => void exportCsv()} aria-label="CSV エクスポート">
            CSV
          </Button>
        </Stack>
      </Paper>
      <Typography variant="caption" color="text.secondary" gutterBottom>
        {query.data?.length ?? 0} 件
      </Typography>
      <Box sx={{ width: '100%' }}>
        <DataGrid
          rows={query.data ?? []}
          columns={columns}
          loading={query.isLoading}
          getRowId={(r) => r.eventId}
          autoHeight
          pageSizeOptions={[25, 50, 100]}
          initialState={{ pagination: { paginationModel: { pageSize: 50, page: 0 } } }}
          aria-label="監査ログテーブル"
        />
      </Box>
    </Box>
  );
}
