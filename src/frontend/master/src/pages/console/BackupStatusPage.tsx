import type React from 'react';
import { useQuery } from '@tanstack/react-query';
import {
  Box,
  Paper,
  Stack,
  Typography,
  Chip,
} from '@mui/material';
import { DataGrid, type GridColDef } from '@mui/x-data-grid';
import { api } from '@/api/client';
import { PageHeader } from '@/components/PageHeader';
import { StatusLight, type LightLevel } from '@/components/StatusLight';

interface BackupItem {
  id: string;
  executedAt: string;
  sizeBytes: number;
  type: 'FULL' | 'DIFF';
  status: LightLevel;
}

interface BackupStatus {
  pgDumpHistory: BackupItem[];
  walArchive: { status: LightLevel; lastArchivedAt: string; unarchivedBytes: number };
}

// SCR-MC-006 バックアップ状況（system_admin、NFR-AVL）。
export function BackupStatusPage(): React.ReactElement {
  const query = useQuery({
    queryKey: ['console', 'backup-status'],
    queryFn: async (): Promise<BackupStatus> => {
      try {
        const r = await api.get<BackupStatus>('/system/backup-status');
        return r.data;
      } catch {
        return {
          pgDumpHistory: [],
          walArchive: { status: 'gray', lastArchivedAt: '', unarchivedBytes: 0 },
        };
      }
    },
  });

  const data = query.data;

  const columns: GridColDef<BackupItem>[] = [
    { field: 'executedAt', headerName: '実行日時', width: 200 },
    {
      field: 'sizeBytes',
      headerName: 'サイズ',
      width: 140,
      valueFormatter: (value: number | undefined) => (value !== undefined ? `${(value / (1024 * 1024)).toFixed(1)} MB` : ''),
    },
    {
      field: 'type',
      headerName: '種別',
      width: 100,
      renderCell: ({ value }) => <Chip label={String(value)} size="small" color={value === 'FULL' ? 'primary' : 'default'} />,
    },
    {
      field: 'status',
      headerName: '状態',
      width: 160,
      renderCell: ({ row }) => <StatusLight level={row.status} label={row.status === 'green' ? '成功' : '失敗'} />,
    },
  ];

  return (
    <Box>
      <PageHeader title="バックアップ状況" subtitle="pg_dump 履歴・WAL アーカイブ状態" />
      <Stack spacing={3}>
        <Paper sx={{ p: 2 }} elevation={1}>
          <Typography variant="h3" gutterBottom>
            WAL アーカイブ
          </Typography>
          {data && (
            <Stack direction="row" spacing={4} alignItems="center">
              <StatusLight level={data.walArchive.status} label="WAL アーカイブ状態" />
              <Typography variant="body2">
                最終アーカイブ: {data.walArchive.lastArchivedAt || '—'}
              </Typography>
              <Typography variant="body2">
                未アーカイブ量: {(data.walArchive.unarchivedBytes / (1024 * 1024)).toFixed(1)} MB
              </Typography>
            </Stack>
          )}
        </Paper>
        <Paper sx={{ p: 2 }} elevation={1}>
          <Typography variant="h3" gutterBottom>
            pg_dump 履歴
          </Typography>
          <DataGrid
            rows={data?.pgDumpHistory ?? []}
            columns={columns}
            loading={query.isLoading}
            getRowId={(r) => r.id}
            autoHeight
            pageSizeOptions={[10, 25]}
            aria-label="pg_dump 履歴"
          />
        </Paper>
      </Stack>
    </Box>
  );
}
