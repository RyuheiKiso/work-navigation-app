import type React from 'react';
import { Box, Paper, Typography, Stack, Chip } from '@mui/material';

interface DiffField {
  field: string;
  before: unknown;
  after: unknown;
  changed: boolean;
}

// 旧版 vs 新版の side-by-side 差分表示。変更フィールドをハイライト。
export function VersionDiffViewer({
  beforeLabel = '旧版',
  afterLabel = '新版',
  fields,
}: {
  beforeLabel?: string;
  afterLabel?: string;
  fields: DiffField[];
}): React.ReactElement {
  return (
    <Stack direction="row" spacing={2} aria-label="バージョン差分">
      <Paper sx={{ flex: 1, p: 2 }} elevation={1}>
        <Typography variant="h3" gutterBottom>
          {beforeLabel}
        </Typography>
        {fields.map((f) => (
          <Box key={`b-${f.field}`} sx={{ mb: 1 }} aria-label={`${beforeLabel} ${f.field}`}>
            <Typography variant="caption" color="text.secondary">{f.field}</Typography>
            <Box
              sx={{
                p: 1,
                borderRadius: 1,
                backgroundColor: f.changed ? 'warning.light' : 'transparent',
                color: f.changed ? 'warning.contrastText' : 'text.primary',
              }}
            >
              {formatValue(f.before)}
            </Box>
          </Box>
        ))}
      </Paper>
      <Paper sx={{ flex: 1, p: 2 }} elevation={1}>
        <Typography variant="h3" gutterBottom>
          {afterLabel}
        </Typography>
        {fields.map((f) => (
          <Box key={`a-${f.field}`} sx={{ mb: 1 }} aria-label={`${afterLabel} ${f.field}`}>
            <Typography variant="caption" color="text.secondary">
              {f.field}
              {f.changed && <Chip label="変更" size="small" color="warning" sx={{ ml: 1, height: 18 }} />}
            </Typography>
            <Box
              sx={{
                p: 1,
                borderRadius: 1,
                backgroundColor: f.changed ? 'success.light' : 'transparent',
                color: f.changed ? 'success.contrastText' : 'text.primary',
              }}
            >
              {formatValue(f.after)}
            </Box>
          </Box>
        ))}
      </Paper>
    </Stack>
  );
}

function formatValue(value: unknown): string {
  if (value == null) return '—';
  if (typeof value === 'string') return value;
  if (typeof value === 'number' || typeof value === 'boolean') return String(value);
  try {
    return JSON.stringify(value, null, 2);
  } catch {
    return '[unserializable]';
  }
}
