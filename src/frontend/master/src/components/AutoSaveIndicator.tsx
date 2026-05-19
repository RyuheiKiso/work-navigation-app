import type React from 'react';
import { Box, Chip } from '@mui/material';
import CheckCircleIcon from '@mui/icons-material/CheckCircle';
import SyncIcon from '@mui/icons-material/Sync';
import EditIcon from '@mui/icons-material/Edit';
import dayjs from 'dayjs';

// SOP 編集の保存状態を視覚化。Auto-Save debounce 中（編集中）・保存中・保存完了の 3 状態。
export function AutoSaveIndicator({
  state,
  lastSavedAt,
}: {
  state: 'idle' | 'editing' | 'saving' | 'saved' | 'error';
  lastSavedAt: string | null;
}): React.ReactElement {
  switch (state) {
    case 'saving':
      return <Chip icon={<SyncIcon />} label="保存中..." color="info" size="small" />;
    case 'editing':
      return <Chip icon={<EditIcon />} label="未保存の変更があります" color="warning" size="small" />;
    case 'error':
      return <Chip label="保存に失敗しました" color="error" size="small" />;
    case 'saved':
      return (
        <Chip
          icon={<CheckCircleIcon />}
          label={lastSavedAt ? `保存済 ${dayjs(lastSavedAt).format('HH:mm:ss')}` : '保存済'}
          color="success"
          size="small"
        />
      );
    case 'idle':
    default:
      return <Box />;
  }
}
