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
} from '@mui/material';
import { DataGrid, type GridColDef } from '@mui/x-data-grid';
import { DateTimePicker } from '@mui/x-date-pickers/DateTimePicker';
import dayjs, { type Dayjs } from 'dayjs';
import type { ConcessionApproval } from '@wnav/shared/types';
import { api } from '@/api/client';
import { queryKeys } from '@/api/queryKeys';
import { PageHeader } from '@/components/PageHeader';
import { SignaturePad } from '@/components/SignaturePad';
import { useAuth } from '@/auth/useAuth';

// SCR-MC-010 特採承認（quality_admin、FR-IQC-009）。
export function ConcessionApprovalPage(): React.ReactElement {
  const { user } = useAuth();
  const queryClient = useQueryClient();
  const [selected, setSelected] = useState<ConcessionApproval | null>(null);
  const [signature, setSignature] = useState<string | null>(null);
  const [validUntil, setValidUntil] = useState<Dayjs | null>(dayjs().add(30, 'day'));
  const [conditionNote, setConditionNote] = useState('');
  const [error, setError] = useState<string | null>(null);

  const pendingQuery = useQuery({
    queryKey: queryKeys.console.concessions('PENDING'),
    queryFn: async (): Promise<ConcessionApproval[]> => {
      const r = await api.getList<ConcessionApproval>('/concession-approvals?status=PENDING');
      return r.data;
    },
  });

  const approve = useMutation({
    mutationFn: async () => {
      if (!user || !selected) throw new Error('未選択');
      if (!signature) throw new Error('署名が必要です');
      if (!validUntil) throw new Error('有効期限が必要です');
      const signResp = await api.post<{ id: string }>('/electronic-signs', {
        context_type: 'approval_sign',
        context_id: selected.id,
        signature_base64: signature,
      });
      await api.post(`/concession-approvals/${selected.id}/sign`, {
        approved_by: user.id,
        electronic_sign_id: signResp.data.id,
        valid_until: validUntil.toISOString(),
        condition_note: conditionNote,
      });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.console.concessions('PENDING') });
      setSelected(null);
      setSignature(null);
      setConditionNote('');
    },
    onError: (e: unknown) => setError(e instanceof Error ? e.message : '承認に失敗しました'),
  });

  const columns: GridColDef<ConcessionApproval>[] = [
    { field: 'id', headerName: '申請 ID', width: 280 },
    { field: 'incomingInspectionId', headerName: '検査 ID', width: 280 },
    { field: 'createdAt', headerName: '申請日時', width: 200 },
    {
      field: 'status',
      headerName: '状態',
      width: 140,
      renderCell: ({ value }) => <Chip label={String(value)} size="small" color={value === 'PENDING' ? 'warning' : 'success'} />,
    },
  ];

  return (
    <Box>
      <PageHeader title="特採承認" subtitle="CONDITIONAL_PASS 申請の電子サイン承認" />
      {error && (
        <Alert severity="error" sx={{ mb: 2 }} onClose={() => setError(null)}>
          {error}
        </Alert>
      )}
      <Stack spacing={3}>
        <Paper sx={{ p: 2 }} elevation={1}>
          <Typography variant="h3" gutterBottom>
            未承認の特採申請 ({pendingQuery.data?.length ?? 0})
          </Typography>
          <DataGrid
            rows={pendingQuery.data ?? []}
            columns={columns}
            loading={pendingQuery.isLoading}
            getRowId={(r) => r.id}
            autoHeight
            onRowClick={(p) => setSelected(p.row)}
            aria-label="特採承認テーブル"
          />
        </Paper>
        {selected && (
          <Paper sx={{ p: 3 }} elevation={2}>
            <Typography variant="h3" gutterBottom>
              申請 {selected.id} を承認
            </Typography>
            <Stack spacing={2}>
              <DateTimePicker
                label="有効期限"
                value={validUntil}
                onChange={setValidUntil}
                disablePast
                ampm={false}
              />
              <TextField
                label="条件・備考"
                multiline
                rows={3}
                value={conditionNote}
                onChange={(e) => setConditionNote(e.target.value)}
                inputProps={{ maxLength: 1000, 'aria-label': '条件・備考' }}
              />
              <SignaturePad onChange={setSignature} />
              <Stack direction="row" spacing={2}>
                <Button onClick={() => setSelected(null)} aria-label="承認をキャンセル">
                  キャンセル
                </Button>
                <Button
                  variant="contained"
                  color="success"
                  disabled={approve.isPending}
                  onClick={() => approve.mutate()}
                  aria-label="特採を承認"
                >
                  承認
                </Button>
              </Stack>
            </Stack>
          </Paper>
        )}
      </Stack>
    </Box>
  );
}
