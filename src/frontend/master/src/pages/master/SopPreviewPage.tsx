import type React from 'react';
import { useState } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { useQuery } from '@tanstack/react-query';
import {
  Box,
  Button,
  Stack,
  Paper,
  Typography,
  Chip,
  Alert,
  LinearProgress,
} from '@mui/material';
import { NavigateBefore, NavigateNext, Close } from '@mui/icons-material';
import type { Sop, Step } from '@wnav/shared/types';
import { resolveLocale } from '@wnav/shared/i18n';
import { api } from '@/api/client';
import { queryKeys } from '@/api/queryKeys';
import { PageHeader } from '@/components/PageHeader';

// SCR-MA-006 SOP プレビュー。Step 実行シミュレーション（FR-MA-007）。読み取り専用。
export function SopPreviewPage(): React.ReactElement {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const [stepIndex, setStepIndex] = useState(0);

  const sopQuery = useQuery({
    queryKey: queryKeys.master.sop(id ?? ''),
    queryFn: async (): Promise<Sop> => {
      const r = await api.get<Sop>(`/master/sops/${id}`);
      return r.data;
    },
    enabled: !!id,
  });

  const stepsQuery = useQuery<Step[]>({
    queryKey: ['master', 'sop', id, 'steps'],
    queryFn: async () => {
      const r = await api.getList<Step>(`/master/sops/${id}/steps`);
      return r.data;
    },
    enabled: !!id,
  });

  if (sopQuery.isLoading || stepsQuery.isLoading) return <LinearProgress aria-label="SOP 読み込み中" />;
  if (!sopQuery.data) return <Alert severity="error">SOP が見つかりません</Alert>;
  const steps = stepsQuery.data ?? [];
  if (steps.length === 0) return <Alert severity="info">Step が登録されていません</Alert>;

  const current = steps[stepIndex];
  if (!current) return <Alert severity="info">Step が登録されていません</Alert>;

  return (
    <Box>
      <PageHeader
        title={`プレビュー: ${resolveLocale(sopQuery.data.nameJson, 'ja')}`}
        subtitle={`Step ${stepIndex + 1} / ${steps.length}`}
        actions={
          <Button
            startIcon={<Close />}
            onClick={() => navigate(`/master/sops/${id}/edit`)}
            aria-label="プレビューを閉じる"
          >
            閉じる
          </Button>
        }
      />
      <Paper sx={{ p: 4 }} elevation={2}>
        <Stack spacing={2}>
          <Stack direction="row" spacing={1} alignItems="center">
            <Typography variant="h2">
              Step {current.stepNumber}: {resolveLocale(current.titleJson, 'ja')}
            </Typography>
            <Chip label={current.stepType} size="small" color="primary" />
            {current.isMandatory && <Chip label="必須" size="small" color="error" />}
            {current.requiresSign && <Chip label="署名要" size="small" color="warning" />}
            {current.requiresEvidence && <Chip label="証跡要" size="small" color="info" />}
          </Stack>
          <Typography variant="body1" sx={{ whiteSpace: 'pre-wrap' }}>
            {resolveLocale(current.instructionJson, 'ja')}
          </Typography>
          <Typography variant="caption" color="text.secondary">
            所要時間目安: {current.estimatedSeconds} 秒 / スキルレベル: {current.skillLevelRequired}
          </Typography>
          <Stack direction="row" spacing={2} mt={3}>
            <Button
              startIcon={<NavigateBefore />}
              disabled={stepIndex === 0}
              onClick={() => setStepIndex((i) => Math.max(0, i - 1))}
              aria-label="前の Step へ"
            >
              前へ
            </Button>
            <Button
              endIcon={<NavigateNext />}
              variant="contained"
              disabled={stepIndex >= steps.length - 1}
              onClick={() => setStepIndex((i) => Math.min(steps.length - 1, i + 1))}
              aria-label="次の Step へ"
            >
              次へ
            </Button>
          </Stack>
        </Stack>
      </Paper>
    </Box>
  );
}
