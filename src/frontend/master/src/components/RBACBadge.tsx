import type React from 'react';
import { Chip, Stack } from '@mui/material';
import type { UserRole } from '@wnav/shared/types';

const ROLE_LABEL: Record<UserRole, string> = {
  operator: '作業者',
  supervisor: '監督者',
  quality_admin: '品質管理',
  master_admin: 'マスタ管理',
  system_admin: 'システム管理',
  executive: '経営層',
};

const ROLE_COLOR: Record<UserRole, 'primary' | 'success' | 'warning' | 'error' | 'info' | 'default'> = {
  operator: 'default',
  supervisor: 'info',
  quality_admin: 'success',
  master_admin: 'primary',
  system_admin: 'warning',
  executive: 'error',
};

// 現在ユーザーのロールをバッジで視覚化（誤操作防止のための常時表示）
export function RBACBadge({ roles }: { roles: readonly UserRole[] }): React.ReactElement {
  return (
    <Stack direction="row" spacing={1} aria-label="現在のロール">
      {roles.map((role) => (
        <Chip
          key={role}
          label={ROLE_LABEL[role]}
          color={ROLE_COLOR[role]}
          size="small"
          variant="filled"
        />
      ))}
    </Stack>
  );
}
