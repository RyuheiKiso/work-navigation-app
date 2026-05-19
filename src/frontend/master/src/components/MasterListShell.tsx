import type React from 'react';
import { ReactNode } from 'react';
import { Box, Stack, TextField, InputAdornment } from '@mui/material';
import SearchIcon from '@mui/icons-material/Search';
import { PageHeader } from './PageHeader';
import { MasterTimeMachine } from './MasterTimeMachine';

// マスタ一覧画面の共通レイアウト。検索バー・時点参照コントロール・新規作成ボタン・テーブルを揃える。
export function MasterListShell({
  title,
  subtitle,
  search,
  onSearchChange,
  onAsOfChange,
  actions,
  children,
}: {
  title: string;
  subtitle?: string;
  search?: string;
  onSearchChange?: (next: string) => void;
  onAsOfChange?: (asOfUtc: string | null) => void;
  actions?: ReactNode;
  children: ReactNode;
}): React.ReactElement {
  return (
    <Box>
      <PageHeader
        title={title}
        {...(subtitle !== undefined ? { subtitle } : {})}
        {...(actions !== undefined ? { actions } : {})}
      />
      {onAsOfChange && <MasterTimeMachine onAsOfChange={onAsOfChange} />}
      <Stack direction="row" spacing={2} mb={2}>
        {onSearchChange && (
          <TextField
            size="small"
            placeholder="検索"
            value={search ?? ''}
            onChange={(e) => onSearchChange(e.target.value)}
            inputProps={{ maxLength: 128, 'aria-label': '一覧を検索' }}
            InputProps={{
              startAdornment: (
                <InputAdornment position="start">
                  <SearchIcon fontSize="small" />
                </InputAdornment>
              ),
            }}
            sx={{ width: 320 }}
          />
        )}
      </Stack>
      {children}
    </Box>
  );
}
