import type React from 'react';
import { Box, Typography } from '@mui/material';
import CircleIcon from '@mui/icons-material/Circle';

export type LightLevel = 'green' | 'yellow' | 'red' | 'gray';

const COLOR: Record<LightLevel, string> = {
  green: '#059669',
  yellow: '#F59E0B',
  red: '#DC2626',
  gray: '#94A3B8',
};

// 状態灯（緑/黄/赤）。バックアップ状態・SLI 健全性などで使用（NFR-AVL）。
export function StatusLight({
  level,
  label,
  ariaLabel,
}: {
  level: LightLevel;
  label: string;
  ariaLabel?: string;
}): React.ReactElement {
  return (
    <Box display="inline-flex" alignItems="center" gap={1} role="status" aria-label={ariaLabel ?? `${label} 状態 ${level}`}>
      <CircleIcon sx={{ fontSize: 14, color: COLOR[level] }} />
      <Typography variant="body2">{label}</Typography>
    </Box>
  );
}
