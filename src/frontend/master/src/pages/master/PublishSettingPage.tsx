import type React from 'react';
import { useState } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { useMutation, useQuery } from '@tanstack/react-query';
import { Box, Button, Stack, Alert, Paper, Typography, Chip } from '@mui/material';
import { DateTimePicker } from '@mui/x-date-pickers/DateTimePicker';
import dayjs, { type Dayjs } from 'dayjs';
import type { Sop } from '@wnav/shared/types';
import { resolveLocale } from '@wnav/shared/i18n';
import { api } from '@/api/client';
import { queryKeys } from '@/api/queryKeys';
import { PageHeader } from '@/components/PageHeader';

// SCR-MA-009 公開設定（quality_admin、FR-MA-010）。有効化日時は未来日のみ。
export function PublishSettingPage(): React.ReactElement {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const [effectiveAt, setEffectiveAt] = useState<Dayjs | null>(dayjs().add(1, 'day'));
  const [error, setError] = useState<string | null>(null);

  const sopQuery = useQuery({
    queryKey: queryKeys.master.sop(id ?? ''),
    queryFn: async (): Promise<Sop> => {
      const r = await api.get<Sop>(`/master/sops/${id}`);
      return r.data;
    },
    enabled: !!id,
  });

  const publish = useMutation({
    mutationFn: async () => {
      if (!effectiveAt) throw new Error('有効化日時は必須です');
      if (effectiveAt.isBefore(dayjs())) throw new Error('有効化日時は未来日である必要があります');
      await api.post(`/master/sops/${id}/publish`, { effective_at: effectiveAt.toISOString() });
    },
    onSuccess: () => navigate('/master/sops'),
    onError: (e: unknown) => setError(e instanceof Error ? e.message : '公開に失敗しました'),
  });

  if (sopQuery.isLoading) return <Typography>読み込み中...</Typography>;
  if (!sopQuery.data) return <Alert severity="error">SOP が見つかりません</Alert>;

  return (
    <Box>
      <PageHeader
        title={`公開設定: ${resolveLocale(sopQuery.data.nameJson, 'ja')}`}
        subtitle="有効化日時を指定して公開（過去日不可）"
      />
      {error && (
        <Alert severity="error" sx={{ mb: 2 }} onClose={() => setError(null)}>
          {error}
        </Alert>
      )}
      <Paper sx={{ p: 4 }} elevation={1}>
        <Stack spacing={3}>
          <Stack direction="row" spacing={2} alignItems="center">
            <Typography variant="body1">現在のステータス:</Typography>
            <Chip label={sopQuery.data.currentVersionId ? '公開済' : '未公開'} color="info" />
          </Stack>
          <DateTimePicker
            label="有効化日時"
            value={effectiveAt}
            onChange={setEffectiveAt}
            disablePast
            ampm={false}
            slotProps={{
              textField: { required: true, 'aria-label': '有効化日時' } as never,
            }}
          />
          <Stack direction="row" spacing={2}>
            <Button onClick={() => navigate(-1)} aria-label="キャンセル">
              キャンセル
            </Button>
            <Button
              variant="contained"
              disabled={publish.isPending}
              onClick={() => publish.mutate()}
              aria-label="公開を実行"
            >
              公開する
            </Button>
          </Stack>
        </Stack>
      </Paper>
    </Box>
  );
}
