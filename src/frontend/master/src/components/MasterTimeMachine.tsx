import type React from 'react';
import { Box, FormControlLabel, Switch, Stack, Alert } from '@mui/material';
import { DateTimePicker } from '@mui/x-date-pickers/DateTimePicker';
import { LocalizationProvider } from '@mui/x-date-pickers/LocalizationProvider';
import { AdapterDayjs } from '@mui/x-date-pickers/AdapterDayjs';
import dayjs, { type Dayjs } from 'dayjs';
import { useState } from 'react';

// 「現在版」と「指定時点版」を切替えるコントロール。as_of_utc クエリパラメータをコールバックで返す。
// 過去の作業記録が参照したマスタ版を固定表示する用途（src/frontend/master/CLAUDE.md §時点参照の UI 表現）。
export function MasterTimeMachine({
  onAsOfChange,
  initialAsOf,
}: {
  onAsOfChange: (asOfUtc: string | null) => void;
  initialAsOf?: string;
}): React.ReactElement {
  const [enabled, setEnabled] = useState(Boolean(initialAsOf));
  const [value, setValue] = useState<Dayjs | null>(initialAsOf ? dayjs(initialAsOf) : null);

  const handleToggle = (next: boolean): void => {
    setEnabled(next);
    onAsOfChange(next && value ? value.toISOString() : null);
  };

  const handleDateChange = (next: Dayjs | null): void => {
    setValue(next);
    if (enabled) onAsOfChange(next ? next.toISOString() : null);
  };

  return (
    <LocalizationProvider dateAdapter={AdapterDayjs}>
      <Box mb={2} role="region" aria-label="時点参照コントロール">
        <Stack direction="row" spacing={2} alignItems="center">
          <FormControlLabel
            control={<Switch checked={enabled} onChange={(e) => handleToggle(e.target.checked)} />}
            label="指定時点版で表示"
          />
          {enabled && (
            <DateTimePicker
              label="参照日時（UTC）"
              value={value}
              onChange={handleDateChange}
              ampm={false}
              slotProps={{ textField: { size: 'small', 'aria-label': '参照日時' } }}
            />
          )}
        </Stack>
        {enabled && (
          <Alert severity="info" sx={{ mt: 1 }}>
            指定時点での有効版を表示中。現在版とは内容が異なる場合があります。
          </Alert>
        )}
      </Box>
    </LocalizationProvider>
  );
}
