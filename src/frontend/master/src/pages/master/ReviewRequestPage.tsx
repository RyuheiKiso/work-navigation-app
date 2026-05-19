import type React from 'react';
import { useState } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { useMutation, useQuery } from '@tanstack/react-query';
import { Box, Button, Stack, TextField, Alert, Paper, Typography, CircularProgress } from '@mui/material';
import type { MasterVersion, Sop } from '@wnav/shared/types';
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

  // 過去の公開済みバージョン一覧を取得して最新の published を特定する
  const versionsQuery = useQuery({
    queryKey: ['master', 'sops', id, 'versions'],
    queryFn: async (): Promise<MasterVersion[]> => {
      const r = await api.getList<MasterVersion>(`/master/sops/${id}/versions`);
      return r.data;
    },
    enabled: !!id,
  });

  const lastPublished = versionsQuery.data
    ?.filter((v) => v.status === 'published' && v.publishedAt != null)
    .sort((a, b) => {
      const aAt = a.publishedAt ?? '';
      const bAt = b.publishedAt ?? '';
      return bAt.localeCompare(aAt);
    })[0] ?? null;

  // 直前の公開版 SOP を時点参照で取得する（初版の場合は skip）
  const prevSopQuery = useQuery({
    queryKey: ['master', 'sops', id, 'asOf', lastPublished?.publishedAt],
    queryFn: async (): Promise<Sop> => {
      // enabled ガードで lastPublished と publishedAt の存在を保証済みだが、
      // queryFn 内でも型安全のためランタイムガードを設ける
      if (!lastPublished?.publishedAt) throw new Error('publishedAt が取得できません');
      const r = await api.get<Sop>(`/master/sops/${id}?as_of=${encodeURIComponent(lastPublished.publishedAt)}`);
      return r.data;
    },
    enabled: !!id && lastPublished != null && lastPublished.publishedAt != null,
  });

  const review = useMutation({
    mutationFn: async () => {
      // OpenAPI operationId: submitSopForReview → /submit エンドポイントを使用する
      await api.post(`/master/sops/${id}/submit`, { comment });
    },
    onSuccess: () => navigate(`/master/sops/${id}/edit`, { state: { reviewRequested: true } }),
    onError: (e: unknown) => setError(e instanceof Error ? e.message : 'レビュー依頼に失敗しました'),
  });

  if (sopQuery.isLoading || versionsQuery.isLoading) {
    return <CircularProgress aria-label="読み込み中" />;
  }
  if (!sopQuery.data) return <Alert severity="error">SOP が見つかりません</Alert>;

  const current = sopQuery.data;
  // 直前公開版がなければ初版（before 列は「—」表示になる）
  const prev = prevSopQuery.data ?? null;

  const isFirstVersion = lastPublished == null;

  const diffFields = [
    {
      field: 'SOPコード',
      before: prev?.sopCode ?? (isFirstVersion ? '(初版)' : null),
      after: current.sopCode,
      changed: prev != null && prev.sopCode !== current.sopCode,
    },
    {
      field: '名称（ja）',
      before: prev != null ? resolveLocale(prev.nameJson, 'ja') : (isFirstVersion ? '(初版)' : null),
      after: resolveLocale(current.nameJson, 'ja'),
      changed: prev != null && resolveLocale(prev.nameJson, 'ja') !== resolveLocale(current.nameJson, 'ja'),
    },
    {
      field: '説明（ja）',
      before: prev != null ? resolveLocale(prev.descriptionJson, 'ja') : (isFirstVersion ? '(初版)' : null),
      after: resolveLocale(current.descriptionJson, 'ja'),
      changed: prev != null && resolveLocale(prev.descriptionJson, 'ja') !== resolveLocale(current.descriptionJson, 'ja'),
    },
    {
      field: 'SOP種別',
      before: prev?.sopType ?? (isFirstVersion ? '(初版)' : null),
      after: current.sopType,
      changed: prev != null && prev.sopType !== current.sopType,
    },
  ];

  return (
    <Box>
      <PageHeader
        title={`レビュー依頼: ${resolveLocale(current.nameJson, 'ja')}`}
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
            {isFirstVersion && (
              <Typography component="span" variant="caption" color="text.secondary" sx={{ ml: 1 }}>
                （初回公開 — 比較対象なし）
              </Typography>
            )}
          </Typography>
          <VersionDiffViewer
            beforeLabel={lastPublished != null ? `旧版 v${lastPublished.version}` : '旧版'}
            afterLabel="新版（ドラフト）"
            fields={diffFields}
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
