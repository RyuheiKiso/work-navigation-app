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
} from '@mui/material';
import { DateTimePicker } from '@mui/x-date-pickers/DateTimePicker';
import dayjs, { type Dayjs } from 'dayjs';
import { Download } from '@mui/icons-material';
import { API_BASE_URL } from '@/api/client';
import { PageHeader } from '@/components/PageHeader';

type ReportType = 'RP-001' | 'RP-002' | 'RP-003' | 'RP-004' | 'RP-005' | 'RP-006' | 'RP-007' | 'RP-008' | 'RP-009' | 'RP-010';

const REPORT_LABEL: Record<ReportType, string> = {
  'RP-001': '日次作業実績',
  'RP-002': '週次工程稼働',
  'RP-003': '月次品質サマリ',
  'RP-004': '電子サイン記録',
  'RP-005': 'ロット トレース',
  'RP-006': 'CAPA 状況',
  'RP-007': '受入検査サマリ',
  'RP-008': '特採承認履歴',
  'RP-009': 'リワーク実績',
  'RP-010': '廃却・返却記録',
};

// SCR-MC-009 帳票生成（quality_admin/system_admin、FR-AU-007）。
export function ReportGeneratorPage(): React.ReactElement {
  const [type, setType] = useState<ReportType>('RP-001');
  const [from, setFrom] = useState<Dayjs | null>(dayjs().subtract(7, 'day'));
  const [to, setTo] = useState<Dayjs | null>(dayjs());
  const [format, setFormat] = useState<'PDF' | 'XLSX' | 'CSV'>('PDF');
  const [error, setError] = useState<string | null>(null);

  const exportMutation = useMutation({
    mutationFn: async () => {
      if (!from || !to) throw new Error('期間を指定してください');
      const params = new URLSearchParams({
        from: from.toISOString(),
        to: to.toISOString(),
        format,
      });
      const res = await fetch(`${API_BASE_URL}/reports/${type}?${params.toString()}`, {
        credentials: 'include',
      });
      if (!res.ok) throw new Error('帳票生成に失敗しました');
      const blob = await res.blob();
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `${type}-${dayjs().format('YYYYMMDDHHmmss')}.${format.toLowerCase()}`;
      a.click();
      URL.revokeObjectURL(url);
    },
    onError: (e: unknown) => setError(e instanceof Error ? e.message : '帳票生成に失敗しました'),
  });

  return (
    <Box>
      <PageHeader title="帳票生成" subtitle="RP-001〜010 の帳票を出力" />
      {error && (
        <Alert severity="error" sx={{ mb: 2 }} onClose={() => setError(null)}>
          {error}
        </Alert>
      )}
      <Paper sx={{ p: 4 }} elevation={1}>
        <Stack spacing={3}>
          <TextField
            select
            label="帳票種別"
            value={type}
            onChange={(e) => setType(e.target.value as ReportType)}
            inputProps={{ 'aria-label': '帳票種別' }}
          >
            {(Object.keys(REPORT_LABEL) as ReportType[]).map((t) => (
              <MenuItem key={t} value={t}>
                {t}: {REPORT_LABEL[t]}
              </MenuItem>
            ))}
          </TextField>
          <Stack direction="row" spacing={2}>
            <DateTimePicker label="開始日時" value={from} onChange={setFrom} ampm={false} />
            <DateTimePicker label="終了日時" value={to} onChange={setTo} ampm={false} />
          </Stack>
          <TextField
            select
            label="フォーマット"
            value={format}
            onChange={(e) => setFormat(e.target.value as 'PDF' | 'XLSX' | 'CSV')}
            inputProps={{ 'aria-label': 'フォーマット' }}
          >
            <MenuItem value="PDF">PDF</MenuItem>
            <MenuItem value="XLSX">Excel</MenuItem>
            <MenuItem value="CSV">CSV</MenuItem>
          </TextField>
          {exportMutation.isPending && <LinearProgress aria-label="生成中" />}
          <Box>
            <Button
              variant="contained"
              startIcon={<Download />}
              disabled={exportMutation.isPending}
              onClick={() => exportMutation.mutate()}
              aria-label="帳票をダウンロード"
            >
              ダウンロード
            </Button>
          </Box>
        </Stack>
      </Paper>
    </Box>
  );
}
