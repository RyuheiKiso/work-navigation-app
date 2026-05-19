import type React from 'react';
import { useState } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { useMutation, useQuery } from '@tanstack/react-query';
import { Box, Button, Stack, TextField, Alert, Paper, Typography } from '@mui/material';
import type { Sop } from '@wnav/shared/types';
import { resolveLocale } from '@wnav/shared/i18n';
import { api } from '@/api/client';
import { queryKeys } from '@/api/queryKeys';
import { PageHeader } from '@/components/PageHeader';
import { VersionDiffViewer } from '@/components/VersionDiffViewer';

// SCR-MA-007 レビュー依頼（docs/05/05_WebAPP詳細設計/04_ApprovalWorkflow詳細設計.md）。
// 差分表示 + コメント + 送信。送信後は in_review 状態になる。
export function ReviewRequestPage(): React.ReactElement {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const [comment, setComment] = useState('');
  const [error, setError] = useState<string | null>(null);

  const sopQuery = useQuery({
    queryKey: queryKeys.master.sop(id ?? ''),
    queryFn: async (): Promise<Sop> => {
      const r = await api.get<Sop>(`/master/sops/${id}`);
      return r.data;
    },
    enabled: !!id,
  });

  const review = useMutation({
    mutationFn: async () => {
      await api.post(`/master/sops/${id}/review`, { comment });
    },
    onSuccess: () => navigate(`/master/sops/${id}/edit`, { state: { reviewRequested: true } }),
    onError: (e: unknown) => setError(e instanceof Error ? e.message : 'レビュー依頼に失敗しました'),
  });

  if (sopQuery.isLoading) return <Typography>読み込み中...</Typography>;
  if (!sopQuery.data) return <Alert severity="error">SOP が見つかりません</Alert>;

  return (
    <Box>
      <PageHeader
        title={`レビュー依頼: ${resolveLocale(sopQuery.data.nameJson, 'ja')}`}
        subtitle="差分を確認し、レビュアー宛にコメント付きで依頼します"
      />
      {error && (
        <Alert severity="error" sx={{ mb: 2 }} onClose={() => setError(null)}>
          {error}
        </Alert>
      )}
      <Stack spacing={3}>
        <Paper sx={{ p: 2 }} elevation={1}>
          <Typography variant="h3" gutterBottom>
            差分プレビュー
          </Typography>
          <VersionDiffViewer
            fields={[
              { field: 'sopCode', before: sopQuery.data.sopCode, after: sopQuery.data.sopCode, changed: false },
              {
                field: '名称（ja）',
                before: '(旧版未保存)',
                after: resolveLocale(sopQuery.data.nameJson, 'ja'),
                changed: true,
              },
            ]}
          />
        </Paper>
        <TextField
          label="レビュアー宛コメント"
          multiline
          rows={4}
          value={comment}
          onChange={(e) => setComment(e.target.value)}
          inputProps={{ maxLength: 1000, 'aria-label': 'レビューコメント' }}
          helperText={`${comment.length} / 1000 文字`}
        />
        <Stack direction="row" spacing={2}>
          <Button onClick={() => navigate(`/master/sops/${id}/edit`)} aria-label="編集に戻る">
            キャンセル
          </Button>
          <Button
            variant="contained"
            disabled={review.isPending}
            onClick={() => review.mutate()}
            aria-label="レビュー依頼を送信"
          >
            レビュー依頼を送信
          </Button>
        </Stack>
      </Stack>
    </Box>
  );
}
