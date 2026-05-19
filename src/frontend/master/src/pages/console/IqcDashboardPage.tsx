import type React from 'react';
import { useQuery } from '@tanstack/react-query';
import { Box, Paper, Grid, Typography, Stack } from '@mui/material';
import {
  BarChart,
  Bar,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  Legend,
  ResponsiveContainer,
  LineChart,
  Line,
} from 'recharts';
import { api } from '@/api/client';
import { PageHeader } from '@/components/PageHeader';

interface IqcMetrics {
  passRate: number;
  failRate: number;
  totalLots: number;
  bySupplier: Array<{ supplierName: string; passed: number; failed: number }>;
  failRateTrend: Array<{ ts: string; rate: number }>;
}

// SCR-MC-011 受入検査ダッシュボード（quality_admin/executive、FR-IQC-011）。
export function IqcDashboardPage(): React.ReactElement {
  const query = useQuery({
    queryKey: ['console', 'iqc-dashboard'],
    queryFn: async (): Promise<IqcMetrics> => {
      try {
        const r = await api.get<IqcMetrics>('/iqc/dashboard');
        return r.data;
      } catch {
        return { passRate: 0, failRate: 0, totalLots: 0, bySupplier: [], failRateTrend: [] };
      }
    },
  });

  const m = query.data;

  return (
    <Box>
      <PageHeader title="受入検査ダッシュボード" subtitle="仕入先別品質実績・不合格率推移" />
      {m && (
        <Grid container spacing={3}>
          <Grid item xs={12} md={4}>
            <Paper sx={{ p: 2 }} elevation={1}>
              <Typography variant="overline">合格率</Typography>
              <Typography variant="h2" color="success.main">
                {m.passRate.toFixed(2)}%
              </Typography>
            </Paper>
          </Grid>
          <Grid item xs={12} md={4}>
            <Paper sx={{ p: 2 }} elevation={1}>
              <Typography variant="overline">不合格率</Typography>
              <Typography variant="h2" color="error.main">
                {m.failRate.toFixed(2)}%
              </Typography>
            </Paper>
          </Grid>
          <Grid item xs={12} md={4}>
            <Paper sx={{ p: 2 }} elevation={1}>
              <Typography variant="overline">対象ロット</Typography>
              <Typography variant="h2">{m.totalLots}</Typography>
            </Paper>
          </Grid>
          <Grid item xs={12} md={6}>
            <Paper sx={{ p: 2, height: 360 }} elevation={1}>
              <Typography variant="h3" gutterBottom>
                仕入先別 合格 / 不合格
              </Typography>
              <ResponsiveContainer>
                <BarChart data={m.bySupplier}>
                  <CartesianGrid strokeDasharray="3 3" />
                  <XAxis dataKey="supplierName" />
                  <YAxis />
                  <Tooltip />
                  <Legend />
                  <Bar dataKey="passed" stackId="a" fill="#059669" name="合格" />
                  <Bar dataKey="failed" stackId="a" fill="#DC2626" name="不合格" />
                </BarChart>
              </ResponsiveContainer>
            </Paper>
          </Grid>
          <Grid item xs={12} md={6}>
            <Paper sx={{ p: 2, height: 360 }} elevation={1}>
              <Typography variant="h3" gutterBottom>
                不合格率 推移
              </Typography>
              <ResponsiveContainer>
                <LineChart data={m.failRateTrend}>
                  <CartesianGrid strokeDasharray="3 3" />
                  <XAxis dataKey="ts" />
                  <YAxis />
                  <Tooltip />
                  <Line type="monotone" dataKey="rate" stroke="#DC2626" name="不合格率 %" />
                </LineChart>
              </ResponsiveContainer>
            </Paper>
          </Grid>
        </Grid>
      )}
      {!m && (
        <Stack>
          <Typography color="text.secondary">読み込み中...</Typography>
        </Stack>
      )}
    </Box>
  );
}
