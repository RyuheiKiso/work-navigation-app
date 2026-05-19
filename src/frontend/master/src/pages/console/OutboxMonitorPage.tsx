import type React from 'react';
import { useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import {
  Box,
  Button,
  Stack,
  Typography,
  Paper,
  Switch,
  FormControlLabel,
  Chip,
  Alert,
} from '@mui/material';
import { DataGrid, type GridColDef } from '@mui/x-data-grid';
import { Replay, DeleteForever } from '@mui/icons-material';
import type { OutboxEvent } from '@wnav/shared/types';
import { api } from '@/api/client';
import { queryKeys } from '@/api/queryKeys';
import { PageHeader } from '@/components/PageHeader';

// SCR-MC-007 Outbox / DLQ 監視（system_admin、FR-SY-007）。
export function OutboxMonitorPage(): React.ReactElement {
  const queryClient = useQueryClient();
  const [autoRefresh, setAutoRefresh] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const dlqQuery = useQuery({
    queryKey: queryKeys.console.outbox(),
    queryFn: async (): Promise<OutboxEvent[]> => {
      const r = await api.getList<OutboxEvent>('/outbox/dlq');
      return r.data;
    },
    refetchInterval: autoRefresh ? 10_000 : false,
  });

  const retryMutation = useMutation({
    mutationFn: async (id: string) => api.post(`/outbox/dlq/${id}/retry`, {}),
    onSettled: () => queryClient.invalidateQueries({ queryKey: queryKeys.console.outbox() }),
    onError: (e: unknown) => setError(e instanceof Error ? e.message : '再投入に失敗しました'),
  });

  const discardMutation = useMutation({
    mutationFn: async (id: string) => api.delete(`/outbox/dlq/${id}`),
    onSettled: () => queryClient.invalidateQueries({ queryKey: queryKeys.console.outbox() }),
    onError: (e: unknown) => setError(e instanceof Error ? e.message : '破棄に失敗しました'),
  });

  const columns: GridColDef<OutboxEvent>[] = [
    { field: 'id', headerName: 'イベント ID', width: 280 },
    {
      field: 'eventType',
      headerName: '種別',
      width: 200,
      renderCell: ({ value }) => <Chip label={String(value)} size="small" />,
    },
    { field: 'retryCount', headerName: '失敗回数', width: 120 },
    { field: 'lastFailedAt', headerName: '最終失敗', width: 200 },
    { field: 'lastError', headerName: 'エラー内容', flex: 1 },
    {
      field: 'actions',
      headerName: '操作',
      width: 260,
      sortable: false,
      renderCell: ({ row }) => (
        <Stack direction="row" spacing={1}>
          <Button
            size="small"
            startIcon={<Replay />}
            onClick={() => retryMutation.mutate(row.id)}
            disabled={retryMutation.isPending}
            aria-label={`${row.id} を再投入`}
          >
            再投入
          </Button>
          <Button
            size="small"
            color="error"
            startIcon={<DeleteForever />}
            onClick={() => discardMutation.mutate(row.id)}
            disabled={discardMutation.isPending}
            aria-label={`${row.id} を破棄`}
          >
            破棄
          </Button>
        </Stack>
      ),
    },
  ];

  return (
    <Box>
      <PageHeader
        title="Outbox / DLQ 監視"
        subtitle="未配信イベントの再投入・破棄"
        actions={
          <FormControlLabel
            control={<Switch checked={autoRefresh} onChange={(e) => setAutoRefresh(e.target.checked)} />}
            label="10 秒ごとに自動更新"
          />
        }
      />
      {error && (
        <Alert severity="error" sx={{ mb: 2 }} onClose={() => setError(null)}>
          {error}
        </Alert>
      )}
      <Paper sx={{ p: 2, mb: 2 }} elevation={1}>
        <Typography variant="h3">
          DLQ 総件数: {dlqQuery.data?.length ?? 0}
        </Typography>
      </Paper>
      <Box sx={{ width: '100%' }}>
        <DataGrid
          rows={dlqQuery.data ?? []}
          columns={columns}
          loading={dlqQuery.isLoading}
          getRowId={(r) => r.id}
          autoHeight
          pageSizeOptions={[25, 50, 100]}
          aria-label="DLQ テーブル"
        />
      </Box>
    </Box>
  );
}
