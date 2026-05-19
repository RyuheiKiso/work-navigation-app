import type React from 'react';
import { useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import { Box, Grid, Paper, Stack, Typography, MenuItem, TextField } from '@mui/material';
import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  Legend,
  ResponsiveContainer,
} from 'recharts';
import { api } from '@/api/client';
import { PageHeader } from '@/components/PageHeader';
import { SliGauge } from '@/components/SliGauge';
import { StatusLight, type LightLevel } from '@/components/StatusLight';

interface DashboardMetrics {
  availability: number;
  latencyP95Ms: number;
  errorRate: number;
  errorBudgetRemaining: number;
  dlqCount: number;
  andonActiveCount: number;
  backupStatus: LightLevel;
  series: Array<{ ts: string; availability: number; errorRate: number }>;
}

// SCR-MC-001 運用ダッシュボード（system_admin/executive、OPS-036〜041）。
export function OperationDashboardPage(): React.ReactElement {
  const [refreshMs, setRefreshMs] = useState<number>(30_000);

  const metricsQuery = useQuery({
    queryKey: ['console', 'dashboard', refreshMs],
    queryFn: async (): Promise<DashboardMetrics> => {
      try {
        const r = await api.get<DashboardMetrics>('/system/metrics');
        return r.data;
      } catch {
        // 未実装時のダミー値（UI 動作確認用）
        return {
          availability: 99.92,
          latencyP95Ms: 230,
          errorRate: 0.05,
          errorBudgetRemaining: 78,
          dlqCount: 0,
          andonActiveCount: 0,
          backupStatus: 'green',
          series: [],
        };
      }
    },
    refetchInterval: refreshMs,
  });

  const m = metricsQuery.data;

  return (
    <Box>
      <PageHeader
        title="運用ダッシュボード"
        subtitle="SLI / エラーバジェット / 異常系の俯瞰"
        actions={
          <TextField
            select
            size="small"
            label="自動更新"
            value={refreshMs}
            onChange={(e) => setRefreshMs(Number(e.target.value))}
            inputProps={{ 'aria-label': '自動更新間隔' }}
          >
            <MenuItem value={10_000}>10 秒</MenuItem>
            <MenuItem value={30_000}>30 秒</MenuItem>
            <MenuItem value={60_000}>1 分</MenuItem>
            <MenuItem value={300_000}>5 分</MenuItem>
          </TextField>
        }
      />

      {m && (
        <Grid container spacing={3}>
          <Grid item xs={12} md={4}>
            <Paper sx={{ p: 2 }} elevation={1}>
              <SliGauge label="可用性 SLI" value={m.availability} target={99.9} />
            </Paper>
          </Grid>
          <Grid item xs={12} md={4}>
            <Paper sx={{ p: 2 }} elevation={1}>
              <SliGauge label="レイテンシ p95" value={m.latencyP95Ms} target={500} unit="ms" inverse />
            </Paper>
          </Grid>
          <Grid item xs={12} md={4}>
            <Paper sx={{ p: 2 }} elevation={1}>
              <SliGauge label="エラー率" value={m.errorRate} target={1} unit="%" inverse />
            </Paper>
          </Grid>

          <Grid item xs={12} md={4}>
            <Paper sx={{ p: 2 }} elevation={1}>
              <Typography variant="overline">エラーバジェット残量</Typography>
              <Typography variant="h2">{m.errorBudgetRemaining}%</Typography>
            </Paper>
          </Grid>
          <Grid item xs={12} md={4}>
            <Paper sx={{ p: 2 }} elevation={1}>
              <Typography variant="overline">DLQ 未処理</Typography>
              <Typography variant="h2">{m.dlqCount} 件</Typography>
            </Paper>
          </Grid>
          <Grid item xs={12} md={4}>
            <Paper sx={{ p: 2 }} elevation={1}>
              <Typography variant="overline">アンドン発報中</Typography>
              <Typography variant="h2">{m.andonActiveCount} 件</Typography>
            </Paper>
          </Grid>

          <Grid item xs={12}>
            <Paper sx={{ p: 2 }} elevation={1}>
              <Stack direction="row" justifyContent="space-between" alignItems="center" mb={2}>
                <Typography variant="h3">時系列 SLI</Typography>
                <StatusLight level={m.backupStatus} label="バックアップ状態" />
              </Stack>
              <Box sx={{ height: 320 }}>
                <ResponsiveContainer>
                  <LineChart data={m.series}>
                    <CartesianGrid strokeDasharray="3 3" />
                    <XAxis dataKey="ts" />
                    <YAxis yAxisId="left" />
                    <YAxis yAxisId="right" orientation="right" />
                    <Tooltip />
                    <Legend />
                    <Line yAxisId="left" type="monotone" dataKey="availability" stroke="#059669" name="可用性 %" />
                    <Line yAxisId="right" type="monotone" dataKey="errorRate" stroke="#DC2626" name="エラー率 %" />
                  </LineChart>
                </ResponsiveContainer>
              </Box>
            </Paper>
          </Grid>
        </Grid>
      )}
    </Box>
  );
}
