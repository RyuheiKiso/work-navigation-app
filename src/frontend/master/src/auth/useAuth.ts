import { useQuery, useQueryClient } from '@tanstack/react-query';
import type { UserRole, Locale } from '@wnav/shared/types';
import { api, ApiError } from '@/api/client';

// JWT は httpOnly Cookie 保管のため JS から直接読めない。サーバーから現在のユーザー情報を取得する。
export interface AuthUser {
  id: string;
  loginId: string;
  role: UserRole;
  roles: UserRole[];
  locale: Locale;
  factoryId: string;
}

export const AUTH_QUERY_KEY = ['auth', 'me'] as const;

export function useAuth(): {
  user: AuthUser | null;
  isLoading: boolean;
  isAuthenticated: boolean;
} {
  const { data, isLoading, isError } = useQuery({
    queryKey: AUTH_QUERY_KEY,
    queryFn: async (): Promise<AuthUser | null> => {
      try {
        const result = await api.get<AuthUser>('/auth/me');
        return result.data;
      } catch (e) {
        // 401 は未認証として正常扱い（ログイン画面遷移へ）
        if (e instanceof ApiError && e.status === 401) return null;
        throw e;
      }
    },
    retry: false,
    staleTime: 60_000,
  });

  return {
    user: data ?? null,
    isLoading,
    isAuthenticated: !isError && !!data,
  };
}

// 認証状態を即時無効化したい場合（ログアウト後など）に呼ぶ
export function useInvalidateAuth(): () => Promise<void> {
  const queryClient = useQueryClient();
  return async () => {
    await queryClient.invalidateQueries({ queryKey: AUTH_QUERY_KEY });
  };
}
