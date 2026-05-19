import { vi } from 'vitest';
import type { AuthUser } from '@/auth/useAuth';

// useAuth をテスト用にモック化するヘルパー。指定ロールで認証済み状態を再現する。
export function mockAuthAs(role: AuthUser['role']): void {
  vi.mock('@/auth/useAuth', () => ({
    AUTH_QUERY_KEY: ['auth', 'me'],
    useAuth: () => ({
      user: {
        id: 'u-test',
        loginId: 'tester',
        role,
        roles: [role],
        locale: 'ja',
        factoryId: 'f-1',
      },
      isLoading: false,
      isAuthenticated: true,
    }),
    useInvalidateAuth: () => async () => undefined,
  }));
}

export function mockUnauthenticated(): void {
  vi.mock('@/auth/useAuth', () => ({
    AUTH_QUERY_KEY: ['auth', 'me'],
    useAuth: () => ({ user: null, isLoading: false, isAuthenticated: false }),
    useInvalidateAuth: () => async () => undefined,
  }));
}
