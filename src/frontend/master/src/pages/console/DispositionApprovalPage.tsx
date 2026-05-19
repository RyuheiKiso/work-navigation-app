import type React from 'react';
import { useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import {
  Box,
  Stack,
  Paper,
  Typography,
  Alert,
  Chip,
  Button,
  TextField,
  MenuItem,
} from '@mui/material';
import { DataGrid, type GridColDef } from '@mui/x-data-grid';
import type { Disposition, DispositionType } from '@wnav/shared/types';
import { api } from '@/api/client';
import { queryKeys } from '@/api/queryKeys';
import { PageHeader } from '@/components/PageHeader';
import { SignaturePad } from '@/components/SignaturePad';
import { useAuth } from '@/auth/useAuth';

// SCR-MC-013 ディスポジション承認（quality_admin / supervisor、NFR-SEC-048 二者サイン）。
type SignerRole = 'quality_admin' | 'supervisor';

const DISPOSITION_TYPES: { value: DispositionType; label: string }[] = [
  { value: 'REWORK', label: 'リワーク' },
  { value: 'SCRAP', label: '廃却' },
  { value: 'RETURN_TO_VENDOR', label: '仕入先返却' },
  { value: 'USE_AS_IS', label: '特採使用' },
];

export function DispositionApprovalPage(): React.ReactElement {
  const { user } = useAuth();
  const queryClient = useQueryClient();
  const [selected, setSelected] = useState<Disposition | null>(null);
  const [signature, setSignature] = useState<string | null>(null);
  const [signerRole, setSignerRole] = useState<SignerRole>('quality_admin');
  const [error, setError] = useState<string | null>(null);

  const pending = useQuery({
    queryKey: queryKeys.console.dispositions('pending'),
    queryFn: async (): Promise<Disposition[]> => {
      const r = await api.getList<Disposition>('/dispositions');
      return r.data;
    },
  });

  const sign = useMutation({
    mutationFn: async () => {
      if (!user || !selected) throw new Error('未選択');
      if (!signature) throw new Error('署名が必要です');
      const signResp = await api.post<{ id: string }>('/electronic-signs', {
        context_type: signerRole === 'quality_admin' ? 'approval_sign' : 'quality_check_sign',
        context_id: selected.id,
        signature_base64: signature,
      });
      await api.post(`/dispositions/${selected.id}/sign`, {
        signer_id: user.id,
        signer_role: signerRole,
        electronic_sign_id: signResp.data.id,
      });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.console.dispositions('pending') });
      setSelected(null);
      setSignature(null);
    },
    onError: (e: unknown) => setError(e instanceof Error ? e.message : '署名に失敗しました'),
  });

  const columns: GridColDef<Disposition>[] = [
    { field: 'id', headerName: 'ディスポジション ID', width: 280 },
    { field: 'nonconformityId', headerName: '不適合 ID', width: 280 },
    {
      field: 'dispositionType',
      headerName: '措置種別',
      width: 160,
      renderCell: ({ value }) => <Chip label={String(value)} size="small" />,
    },
    { field: 'decisionReason', headerName: '判定理由', flex: 1 },
    {
      field: '__signs',
      headerName: '署名状況',
      width: 200,
      renderCell: ({ row }) => (
        <Stack direction="row" spacing={0.5}>
          <Chip
            label="品質"
            size="small"
            color={row.qualityAdminSignId ? 'success' : 'default'}
          />
          <Chip
            label="監督"
            size="small"
            color={row.supervisorSignId ? 'success' : 'default'}
          />
        </Stack>
      ),
    },
  ];

  return (
    <Box>
      <PageHeader title="ディスポジション承認" subtitle="二者サイン（品質担当 + 監督者）が必要" />
      {error && (
        <Alert severity="error" sx={{ mb: 2 }} onClose={() => setError(null)}>
          {error}
        </Alert>
      )}
      <Stack spacing={3}>
        <Paper sx={{ p: 2 }} elevation={1}>
          <Typography variant="h3" gutterBottom>
            未完了の処置 ({pending.data?.length ?? 0})
          </Typography>
          <DataGrid
            rows={pending.data ?? []}
            columns={columns}
            loading={pending.isLoading}
            getRowId={(r) => r.id}
            autoHeight
            onRowClick={(p) => setSelected(p.row)}
            aria-label="ディスポジションテーブル"
          />
        </Paper>
        {selected && (
          <Paper sx={{ p: 3 }} elevation={2}>
            <Typography variant="h3" gutterBottom>
              ディスポジション {selected.id} に署名
            </Typography>
            <Stack spacing={2}>
              <TextField
                select
                label="署名者ロール"
                value={signerRole}
                onChange={(e) => setSignerRole(e.target.value as SignerRole)}
                inputProps={{ 'aria-label': '署名者ロール' }}
              >
                <MenuItem value="quality_admin">品質担当</MenuItem>
                <MenuItem value="supervisor">監督者</MenuItem>
              </TextField>
              <Typography variant="caption" color="text.secondary">
                ※ NFR-SEC-048 に従い、品質担当と監督者が独立して署名する必要があります（同一ユーザーによる二重署名は不可）。
              </Typography>
              <SignaturePad onChange={setSignature} />
              <Stack direction="row" spacing={2}>
                <Button onClick={() => setSelected(null)} aria-label="キャンセル">
                  キャンセル
                </Button>
                <Button
                  variant="contained"
                  disabled={sign.isPending}
                  onClick={() => sign.mutate()}
                  aria-label="署名を確定"
                >
                  署名する
                </Button>
              </Stack>
              {DISPOSITION_TYPES.length > 0 && null}
            </Stack>
          </Paper>
        )}
      </Stack>
    </Box>
  );
}
