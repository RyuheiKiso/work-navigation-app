import type React from 'react';
import { useState } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { useMutation, useQuery } from '@tanstack/react-query';
import {
  Box,
  Button,
  Stack,
  Alert,
  Paper,
  Typography,
  TextField,
} from '@mui/material';
import type { Sop } from '@wnav/shared/types';
import { resolveLocale } from '@wnav/shared/i18n';
import { api } from '@/api/client';
import { queryKeys } from '@/api/queryKeys';
import { PageHeader } from '@/components/PageHeader';
import { SignaturePad } from '@/components/SignaturePad';
import { PinInput } from '@/components/PinInput';
import { useAuth } from '@/auth/useAuth';

// SCR-MA-008 承認サイン（quality_admin 限定、FR-MA-009）。
// 電子サイン（手書き署名）+ PIN（4〜8 桁）+ 承認/却下選択。即時公開禁止規約の起点。
type Mode = 'approve' | 'reject';

export function ApprovalSignPage(): React.ReactElement {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const { user } = useAuth();
  const [mode, setMode] = useState<Mode>('approve');
  const [signature, setSignature] = useState<string | null>(null);
  const [pin, setPin] = useState('');
  const [rejectReason, setRejectReason] = useState('');
  const [error, setError] = useState<string | null>(null);

  const sopQuery = useQuery({
    queryKey: queryKeys.master.sop(id ?? ''),
    queryFn: async (): Promise<Sop> => {
      const r = await api.get<Sop>(`/master/sops/${id}`);
      return r.data;
    },
    enabled: !!id,
  });

  const submitMutation = useMutation({
    mutationFn: async () => {
      if (!user) throw new Error('未認証');
      if (mode === 'approve') {
        if (!signature) throw new Error('署名が必要です');
        if (pin.length < 4) throw new Error('PIN は 4 桁以上');
        // 電子サインを先に発行（手書き署名画像 + PIN）。サーバが署名 ID を返す前提。
        const signResp = await api.post<{ id: string }>('/electronic-signs', {
          context_type: 'approval_sign',
          context_id: id,
          signature_base64: signature,
          pin,
        });
        await api.post(`/master/sops/${id}/approve`, {
          approved_by: user.id,
          electronic_sign_id: signResp.data.id,
        });
      } else {
        if (!rejectReason) throw new Error('却下理由は必須です');
        await api.post(`/master/sops/${id}/reject`, { reason: rejectReason });
      }
    },
    onSuccess: () =>
      navigate(mode === 'approve' ? `/master/sops/${id}/publish` : `/master/sops/${id}/edit`, {
        state: { approvalResult: mode },
      }),
    onError: (e: unknown) => setError(e instanceof Error ? e.message : '承認処理に失敗しました'),
  });

  if (sopQuery.isLoading) return <Typography>読み込み中...</Typography>;
  if (!sopQuery.data) return <Alert severity="error">SOP が見つかりません</Alert>;

  return (
    <Box>
      <PageHeader
        title={`承認サイン: ${resolveLocale(sopQuery.data.nameJson, 'ja')}`}
        subtitle="電子サイン + PIN による承認（quality_admin 限定）"
      />
      {error && (
        <Alert severity="error" sx={{ mb: 2 }} onClose={() => setError(null)}>
          {error}
        </Alert>
      )}
      <Stack spacing={3}>
        <Stack direction="row" spacing={2}>
          <Button
            variant={mode === 'approve' ? 'contained' : 'outlined'}
            color="success"
            onClick={() => setMode('approve')}
            aria-label="承認を選択"
          >
            承認
          </Button>
          <Button
            variant={mode === 'reject' ? 'contained' : 'outlined'}
            color="error"
            onClick={() => setMode('reject')}
            aria-label="却下を選択"
          >
            却下
          </Button>
        </Stack>

        {mode === 'approve' ? (
          <>
            <Paper sx={{ p: 2 }} elevation={1}>
              <SignaturePad onChange={setSignature} />
            </Paper>
            <PinInput value={pin} onChange={setPin} label="PIN（4〜8 桁）" />
          </>
        ) : (
          <TextField
            label="却下理由"
            multiline
            rows={4}
            value={rejectReason}
            onChange={(e) => setRejectReason(e.target.value)}
            required
            inputProps={{ maxLength: 500, 'aria-label': '却下理由' }}
            helperText={`${rejectReason.length} / 500 文字`}
          />
        )}

        <Stack direction="row" spacing={2}>
          <Button onClick={() => navigate(-1)} aria-label="キャンセル">
            キャンセル
          </Button>
          <Button
            variant="contained"
            color={mode === 'approve' ? 'success' : 'error'}
            disabled={submitMutation.isPending}
            onClick={() => submitMutation.mutate()}
            aria-label={mode === 'approve' ? '承認を確定' : '却下を確定'}
          >
            {mode === 'approve' ? '承認を確定' : '却下を確定'}
          </Button>
        </Stack>
      </Stack>
    </Box>
  );
}
