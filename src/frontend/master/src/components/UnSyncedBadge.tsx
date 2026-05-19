import type React from 'react';
import { Chip, Tooltip } from '@mui/material';
import SyncProblemIcon from '@mui/icons-material/SyncProblem';
import dayjs from 'dayjs';

// 「記録済（未同期）」フラグ表示。事後記録と誤解されないよう未同期期間を併記する（src/frontend/master/CLAUDE.md §未同期記録の識別表示）。
export function UnSyncedBadge({
  serverReceivedAt,
  clientRecordedAt,
}: {
  serverReceivedAt: string | null;
  clientRecordedAt: string;
}): React.ReactElement | null {
  if (serverReceivedAt && new Date(serverReceivedAt).getTime() - new Date(clientRecordedAt).getTime() < 60_000) {
    return null;
  }
  const delaySec = serverReceivedAt
    ? Math.max(0, Math.floor((new Date(serverReceivedAt).getTime() - new Date(clientRecordedAt).getTime()) / 1000))
    : null;
  const label = serverReceivedAt
    ? `未同期 ${formatDelay(delaySec ?? 0)}`
    : '未同期（サーバ未受信）';
  const tooltip = serverReceivedAt
    ? `端末記録: ${dayjs(clientRecordedAt).format('YYYY-MM-DD HH:mm:ss')} / サーバ受信: ${dayjs(serverReceivedAt).format('YYYY-MM-DD HH:mm:ss')}`
    : `端末記録: ${dayjs(clientRecordedAt).format('YYYY-MM-DD HH:mm:ss')} / サーバ未受信`;
  return (
    <Tooltip title={tooltip} arrow>
      <Chip
        icon={<SyncProblemIcon />}
        label={label}
        color="warning"
        size="small"
        variant="outlined"
        aria-label={`未同期記録 ${tooltip}`}
      />
    </Tooltip>
  );
}

function formatDelay(seconds: number): string {
  if (seconds < 60) return `${seconds}s`;
  if (seconds < 3600) return `${Math.floor(seconds / 60)}m`;
  if (seconds < 86_400) return `${Math.floor(seconds / 3600)}h`;
  return `${Math.floor(seconds / 86_400)}d`;
}
