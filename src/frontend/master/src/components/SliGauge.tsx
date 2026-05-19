import type React from 'react';
import { Box, Typography, LinearProgress, Stack } from '@mui/material';

// SLI ゲージ（CMP-MC-001）。可用性・レイテンシ・エラー率の目標値と実測の差を可視化（OPS-036〜038）。
export function SliGauge({
  label,
  value,
  target,
  unit = '%',
  inverse = false,
}: {
  label: string;
  value: number;
  target: number;
  unit?: string;
  inverse?: boolean;
}): React.ReactElement {
  // inverse=true は「低いほど良い」指標（エラー率・レイテンシ）
  const ratio = Math.min(100, Math.max(0, inverse ? (target / Math.max(value, 0.0001)) * 100 : (value / target) * 100));
  const ok = inverse ? value <= target : value >= target;
  return (
    <Box role="region" aria-label={`${label} SLI`}>
      <Stack direction="row" justifyContent="space-between" mb={0.5}>
        <Typography variant="overline">{label}</Typography>
        <Typography variant="body2" color={ok ? 'success.main' : 'error.main'}>
          {value.toFixed(2)}
          {unit} / 目標 {target}
          {unit}
        </Typography>
      </Stack>
      <LinearProgress
        variant="determinate"
        value={ratio}
        color={ok ? 'success' : 'error'}
        aria-valuemin={0}
        aria-valuemax={100}
        aria-valuenow={Math.round(ratio)}
      />
    </Box>
  );
}
