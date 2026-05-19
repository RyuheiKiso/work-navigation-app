import type React from 'react';
import { useState } from 'react';
import { useNavigate, useLocation } from 'react-router-dom';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { Box, Paper, TextField, Button, Typography, Alert, Stack } from '@mui/material';
import { api, ApiError, storeDevToken } from '@/api/client';
import { AUTH_QUERY_KEY, type AuthUser } from '@/auth/useAuth';
import type { UserRole } from '@wnav/shared/types';

// AuthLoginResponse の必要フィールドのみ定義する（shared/types の AuthLoginResponse は refreshToken 等を含むが未使用）
interface LoginResponse {
  accessToken: string;
  tokenType: string;
  expiresIn: number;
  userId: string;
  roles: UserRole[];
  factoryId: string;
}

// ログイン画面（認証不要）。本番では httpOnly Cookie でセッション管理するため /auth/me への再フェッチで認証状態を確定する。
// DEV/MSW 環境では Cookie が設定できないため setQueryData でキャッシュに直接書き込んで競合を回避する。
export function LoginPage(): React.ReactElement {
  const navigate = useNavigate();
  const location = useLocation();
  const queryClient = useQueryClient();
  const [loginId, setLoginId] = useState('');
  const [password, setPassword] = useState('');
  const [error, setError] = useState<string | null>(null);

  const loginMutation = useMutation({
    mutationFn: async (): Promise<LoginResponse> => {
      const result = await api.post<LoginResponse>('/auth/login', { loginId, password, deviceId: 'master-web', factoryId: 'default' });
      return result.data;
    },
    onSuccess: (data) => {
      storeDevToken(data.accessToken);
      // invalidateQueries ではなく setQueryData で直接書き込む。
      // ログイン画面では AuthGuard が非マウントのためリフェッチが発火せず、
      // キャッシュが null のまま遷移すると AuthGuard が /login へリダイレクトするループが起きる。
      const authUser: AuthUser = {
        id: data.userId,
        loginId,
        role: data.roles[0] ?? ('operator' as UserRole),
        roles: data.roles,
        locale: 'ja',
        factoryId: data.factoryId,
      };
      queryClient.setQueryData(AUTH_QUERY_KEY, authUser);
      const fromState = location.state as { from?: string } | null;
      const from = fromState?.from ?? '/';
      navigate(from, { replace: true });
    },
    onError: (e: unknown) => {
      if (e instanceof ApiError) {
        setError(e.problem.detail || e.problem.title);
      } else {
        setError(e instanceof Error ? e.message : '認証に失敗しました');
      }
    },
  });

  return (
    <Box display="flex" alignItems="center" justifyContent="center" minHeight="100vh" bgcolor="background.default">
      <Paper sx={{ p: 4, width: 400 }} elevation={2}>
        <Typography variant="h1" component="h1" gutterBottom>
          WNAV ログイン
        </Typography>
        <form
          onSubmit={(e) => {
            e.preventDefault();
            setError(null);
            loginMutation.mutate();
          }}
        >
          <Stack spacing={2}>
            <TextField
              label="ログイン ID"
              value={loginId}
              onChange={(e) => setLoginId(e.target.value)}
              required
              autoFocus
              inputProps={{ 'aria-label': 'ログイン ID' }}
            />
            <TextField
              label="パスワード"
              type="password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              required
              inputProps={{ 'aria-label': 'パスワード' }}
            />
            {error && <Alert severity="error" role="alert">{error}</Alert>}
            <Button
              type="submit"
              variant="contained"
              size="large"
              disabled={loginMutation.isPending}
              aria-label="ログインを実行"
            >
              ログイン
            </Button>
          </Stack>
        </form>
      </Paper>
    </Box>
  );
}
