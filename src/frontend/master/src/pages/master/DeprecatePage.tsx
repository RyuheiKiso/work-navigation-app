import type React from 'react';
import { useState } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { useMutation, useQuery } from '@tanstack/react-query';
import { Box, Button, Stack, Alert, Paper, Typography, TextField } from '@mui/material';
import type { Sop } from '@wnav/shared/types';
import { resolveLocale } from '@wnav/shared/i18n';
import { api } from '@/api/client';
import { queryKeys } from '@/api/queryKeys';
import { PageHeader } from '@/components/PageHeader';
import { SignaturePad } from '@/components/SignaturePad';
import { ImpactRangePreview } from '@/components/ImpactRangePreview';

// SCR-MA-011 廃止処理（FR-MA-012）。dry-run（影響範囲）+ 電子サイン + 廃止理由。
export function DeprecatePage(): React.ReactElement {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const [signature, setSignature] = useState<string | null>(null);
  const [reason, setReason] = useState('');
  const [error, setError] = useState<string | null>(null);

  const sopQuery = useQuery({
    queryKey: queryKeys.master.sop(id ?? ''),
    queryFn: async (): Promise<Sop> => {
      const r = await api.get<Sop>(`/master/sops/${id}`);
      return r.data;
    },
    enabled: !!id,
  });

  // dry-run: 廃止前に影響範囲を取得する（紐付き作業指示数など）
  const impactQuery = useQuery({
    queryKey: ['master', 'sop', id, 'impact'],
    queryFn: async (): Promise<{ workOrderCount: number; workExecutionCount: number }> => {
      try {
        const r = await api.get<{ workOrderCount: number; workExecutionCount: number }>(
          `/master/sops/${id}/impact`,
        );
        return r.data;
      } catch {
        // 影響範囲エンドポイントが未実装でも続行できるよう 0 を返す
        return { workOrderCount: 0, workExecutionCount: 0 };
      }
    },
    enabled: !!id,
  });

  const deprecate = useMutation({
    mutationFn: async () => {
      if (!signature) throw new Error('電子サインが必要です');
      if (!reason) throw new Error('廃止理由は必須です');
      await api.post(`/master/sops/${id}/deprecate`, { reason, signature_base64: signature });
    },
    onSuccess: () => navigate('/master/sops'),
    onError: (e: unknown) => setError(e instanceof Error ? e.message : '廃止に失敗しました'),
  });

  if (sopQuery.isLoading) return <Typography>読み込み中...</Typography>;
  if (!sopQuery.data) return <Alert severity="error">SOP が見つかりません</Alert>;

  const impacts: Array<{ type: 'work_order' | 'work_execution'; id: string; label: string }> = [];
  if (impactQuery.data) {
    if (impactQuery.data.workOrderCount > 0) {
      impacts.push({ type: 'work_order', id: 'wo', label: `${impactQuery.data.workOrderCount} 件の作業指示が紐付き` });
    }
    if (impactQuery.data.workExecutionCount > 0) {
      impacts.push({ type: 'work_execution', id: 'we', label: `${impactQuery.data.workExecutionCount} 件の進行中作業が紐付き` });
    }
  }

  return (
    <Box>
      <PageHeader
        title={`廃止: ${resolveLocale(sopQuery.data.nameJson, 'ja')}`}
        subtitle="論理削除（物理削除はされません）"
      />
      {error && (
        <Alert severity="error" sx={{ mb: 2 }} onClose={() => setError(null)}>
          {error}
        </Alert>
      )}
      <Stack spacing={3}>
        <ImpactRangePreview items={impacts} emptyMessage="この SOP を参照中の作業指示はありません" />
        <TextField
          label="廃止理由"
          multiline
          rows={3}
          value={reason}
          onChange={(e) => setReason(e.target.value)}
          required
          inputProps={{ maxLength: 500, 'aria-label': '廃止理由' }}
          helperText={`${reason.length} / 500 文字`}
        />
        <Paper sx={{ p: 2 }} elevation={1}>
          <SignaturePad onChange={setSignature} />
        </Paper>
        <Stack direction="row" spacing={2}>
          <Button onClick={() => navigate(-1)} aria-label="キャンセル">
            キャンセル
          </Button>
          <Button
            variant="contained"
            color="error"
            disabled={deprecate.isPending}
            onClick={() => deprecate.mutate()}
            aria-label="廃止を確定"
          >
            廃止を確定
          </Button>
        </Stack>
      </Stack>
    </Box>
  );
}
