import type React from 'react';
import { useState } from 'react';
import { useMutation } from '@tanstack/react-query';
import {
  Box,
  Button,
  Stack,
  MenuItem,
  TextField,
  Alert,
  Paper,
  LinearProgress,
  Typography,
} from '@mui/material';
import { DateTimePicker } from '@mui/x-date-pickers/DateTimePicker';
import dayjs, { type Dayjs } from 'dayjs';
import { Download } from '@mui/icons-material';
import { API_BASE_URL } from '@/api/client';
import { PageHeader } from '@/components/PageHeader';

type ScopeMode = 'ALL' | 'COMPLETED';

// SCR-MC-005 XES エクスポート（quality_admin/system_admin、FR-AU-005）。
// 期間最大 1 年・XES XML を fetch でストリームダウンロード。
export function XesExportPage(): React.ReactElement {
  const [from, setFrom] = useState<Dayjs | null>(dayjs().subtract(30, 'day'));
  const [to, setTo] = useState<Dayjs | null>(dayjs());
  const [scope, setScope] = useState<ScopeMode>('COMPLETED');
  const [error, setError] = useState<string | null>(null);

  const exportMutation = useMutation({
    mutationFn: async () => {
      if (!from || !to) throw new Error('期間を指定してください');
      const diffDays = to.diff(from, 'day');
      if (diffDays < 0) throw new Error('開始日が終了日より後です');
      if (diffDays > 365) throw new Error('期間は 365 日以内に指定してください');

      const params = new URLSearchParams({ from: from.toISOString(), to: to.toISOString(), scope });
      const res = await fetch(`${API_BASE_URL}/reports/xes?${params.toString()}`, {
        credentials: 'include',
      });
      if (!res.ok) throw new Error('XES 生成に失敗しました');
      const blob = await res.blob();
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `events-${from.format('YYYYMMDD')}-${to.format('YYYYMMDD')}.xes`;
      a.click();
      URL.revokeObjectURL(url);
    },
    onError: (e: unknown) => setError(e instanceof Error ? e.message : 'エクスポートに失敗しました'),
  });

  return (
    <Box>
      <PageHeader title="XES エクスポート" subtitle="期間指定で XES 2.0 XML をダウンロード" />
      {error && (
        <Alert severity="error" sx={{ mb: 2 }} onClose={() => setError(null)}>
          {error}
        </Alert>
      )}
      <Paper sx={{ p: 4 }} elevation={1}>
        <Stack spacing={3}>
          <Stack direction="row" spacing={2}>
            <DateTimePicker label="開始日時" value={from} onChange={setFrom} ampm={false} />
            <DateTimePicker label="終了日時" value={to} onChange={setTo} ampm={false} />
          </Stack>
          <TextField
            select
            label="範囲"
            value={scope}
            onChange={(e) => setScope(e.target.value as ScopeMode)}
            inputProps={{ 'aria-label': 'エクスポート範囲' }}
          >
            <MenuItem value="ALL">すべて</MenuItem>
            <MenuItem value="COMPLETED">完了済みのみ</MenuItem>
          </TextField>
          {exportMutation.isPending && <LinearProgress aria-label="エクスポート中" />}
          <Typography variant="caption" color="text.secondary">
            期間: {from && to ? to.diff(from, 'day') : 0} 日 / 上限 365 日
          </Typography>
          <Box>
            <Button
              variant="contained"
              startIcon={<Download />}
              disabled={exportMutation.isPending}
              onClick={() => exportMutation.mutate()}
              aria-label="XES をダウンロード"
            >
              ダウンロード
            </Button>
          </Box>
        </Stack>
      </Paper>
    </Box>
  );
}
