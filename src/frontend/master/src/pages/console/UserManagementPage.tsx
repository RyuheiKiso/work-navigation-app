import type React from 'react';
import { TextField, MenuItem } from '@mui/material';
import type { GridColDef } from '@mui/x-data-grid';
import type { User, UserRole } from '@wnav/shared/types';
import { resolveLocale } from '@wnav/shared/i18n';
import { GenericMasterListPage } from '@/components/GenericMasterListPage';
import { useAuth } from '@/auth/useAuth';

interface UserForm {
  loginId: string;
  username: string;
  email: string;
  role: UserRole;
  initialPassword: string;
}

const ROLES: { value: UserRole; label: string }[] = [
  { value: 'operator', label: '作業者' },
  { value: 'supervisor', label: '監督者' },
  { value: 'quality_admin', label: '品質管理' },
  { value: 'master_admin', label: 'マスタ管理' },
  { value: 'system_admin', label: 'システム管理' },
  { value: 'executive', label: '経営層' },
];

// SCR-MC-002 ユーザー管理（system_admin、FR-MA-014）。
export function UserManagementPage(): React.ReactElement {
  const { user } = useAuth();
  const factoryId = user?.factoryId ?? '';
  const columns: GridColDef<User>[] = [
    { field: 'loginId', headerName: 'ログイン ID', width: 180 },
    { field: 'username', headerName: 'ユーザー名', width: 180 },
    {
      field: 'displayNameJson',
      headerName: '表示名',
      flex: 1,
      valueGetter: (_, row) => resolveLocale(row.displayNameJson, 'ja'),
    },
    { field: 'role', headerName: 'ロール', width: 140 },
    { field: 'email', headerName: 'メール', width: 220 },
  ];

  return (
    <GenericMasterListPage<User, UserForm>
      title="ユーザー管理"
      subtitle="ユーザー CRUD + ロール割当"
      endpoint="/master/users"
      queryKeyBuilder={() => ['console', 'users']}
      columns={columns}
      initialCreateForm={{ loginId: '', username: '', email: '', role: 'operator', initialPassword: '' }}
      labelOf={(item) => item.loginId}
      validateAndBuildPayload={(form) => {
        if (!form.loginId || form.loginId.length > 128) return { ok: false, message: 'ログイン ID は 1〜128 文字' };
        if (!form.username) return { ok: false, message: 'ユーザー名は必須です' };
        if (form.initialPassword.length < 8) return { ok: false, message: '初期パスワードは 8 文字以上' };
        return {
          ok: true,
          payload: {
            loginId: form.loginId,
            username: form.username,
            email: form.email || null,
            role: form.role,
            roles: [form.role],
            displayNameJson: { ja: form.username, en: '', zh: '' },
            isActive: true,
            locale: 'ja',
            factoryId,
          },
        };
      }}
      renderCreateForm={({ value, onChange }) => (
        <>
          <TextField
            label="ログイン ID"
            value={value.loginId}
            onChange={(e) => onChange({ ...value, loginId: e.target.value })}
            required
            inputProps={{ maxLength: 128, 'aria-label': 'ログイン ID' }}
          />
          <TextField
            label="ユーザー名"
            value={value.username}
            onChange={(e) => onChange({ ...value, username: e.target.value })}
            required
            inputProps={{ maxLength: 200, 'aria-label': 'ユーザー名' }}
          />
          <TextField
            label="メール"
            type="email"
            value={value.email}
            onChange={(e) => onChange({ ...value, email: e.target.value })}
            inputProps={{ 'aria-label': 'メール' }}
          />
          <TextField
            select
            label="ロール"
            value={value.role}
            onChange={(e) => onChange({ ...value, role: e.target.value as UserRole })}
            required
            inputProps={{ 'aria-label': 'ロール' }}
          >
            {ROLES.map((r) => (
              <MenuItem key={r.value} value={r.value}>
                {r.label}
              </MenuItem>
            ))}
          </TextField>
          <TextField
            label="初期パスワード"
            type="password"
            value={value.initialPassword}
            onChange={(e) => onChange({ ...value, initialPassword: e.target.value })}
            required
            inputProps={{ minLength: 8, maxLength: 64, 'aria-label': '初期パスワード' }}
            helperText="8 文字以上 64 文字以内"
          />
        </>
      )}
    />
  );
}
