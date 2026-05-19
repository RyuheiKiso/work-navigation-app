import type React from 'react';
import { useQuery } from '@tanstack/react-query';
import { Box, Paper, Stack, Typography, Button, Alert } from '@mui/material';
import { DataGrid, type GridColDef } from '@mui/x-data-grid';
import { Refresh } from '@mui/icons-material';
import { api } from '@/api/client';
import { PageHeader } from '@/components/PageHeader';
import { StatusLight, type LightLevel } from '@/components/StatusLight';

interface VerifyResult {
  ok: boolean;
  lastVerifiedAt: string;
  verifiedBlockCount: number;
  brokenBlocks: Array<{
    id: string;
    blockNumber: number;
    expectedHash: string;
    actualHash: string;
  }>;
}

// SCR-MC-008 ハッシュチェーン検証（quality_admin/system_admin、FR-AU-006）。
// クライアント側で再検証する設計だが、ここでは結果サマリのみ表示。
export function HashChainVerifierPage(): React.ReactElement {
  const query = useQuery({
    queryKey: ['console', 'hash-chain'],
    queryFn: async (): Promise<VerifyResult> => {
      const r = await api.get<VerifyResult>('/hash-chain/verify');
      return r.data;
    },
  });

  const v = query.data;
  const level: LightLevel = !v ? 'gray' : v.ok ? 'green' : 'red';

  const columns: GridColDef<VerifyResult['brokenBlocks'][number]>[] = [
    { field: 'blockNumber', headerName: 'ブロック番号', width: 140 },
    { field: 'id', headerName: 'ブロック ID', width: 320 },
    { field: 'expectedHash', headerName: '期待ハッシュ', flex: 1 },
    { field: 'actualHash', headerName: '実ハッシュ', flex: 1 },
  ];

  return (
    <Box>
      <PageHeader
        title="ハッシュチェーン検証"
        subtitle="SHA-256 チェーンの整合性確認"
        actions={
          <Button startIcon={<Refresh />} onClick={() => query.refetch()} aria-label="再検証">
            再検証
          </Button>
        }
      />
      <Stack spacing={3}>
        <Paper sx={{ p: 4 }} elevation={1}>
          <Stack direction="row" spacing={4} alignItems="center">
            <StatusLight
              level={level}
              label={v ? (v.ok ? 'チェーン健全' : 'チェーン破断検知') : '未検証'}
            />
            {v && (
              <>
                <Typography variant="body2">最終検証: {v.lastVerifiedAt}</Typography>
                <Typography variant="body2">検証済ブロック: {v.verifiedBlockCount}</Typography>
                <Typography variant="body2" color="error.main">
                  破断: {v.brokenBlocks.length} 件
                </Typography>
              </>
            )}
          </Stack>
        </Paper>
        {v && v.brokenBlocks.length > 0 && (
          <Alert severity="error" role="alert">
            ハッシュチェーンに破断が検出されました。直ちに品質管理者へ報告し、調査を開始してください。
          </Alert>
        )}
        {v && (
          <Paper sx={{ p: 2 }} elevation={1}>
            <Typography variant="h3" gutterBottom>
              破断ブロック一覧
            </Typography>
            <DataGrid
              rows={v.brokenBlocks}
              columns={columns}
              getRowId={(r) => r.id}
              autoHeight
              pageSizeOptions={[10, 25]}
              aria-label="破断ブロックテーブル"
            />
          </Paper>
        )}
      </Stack>
    </Box>
  );
}
