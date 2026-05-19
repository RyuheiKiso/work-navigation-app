import type React from 'react';
import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import { MemoryRouter, Routes, Route } from 'react-router-dom';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { RoleGuard } from '@/auth/RoleGuard';
import type { AuthUser } from '@/auth/useAuth';

// useAuth をモックして異なるロールでのアクセス制御を検証する
vi.mock('@/auth/useAuth', () => ({
  useAuth: vi.fn(),
}));

import { useAuth } from '@/auth/useAuth';

function renderWithRouter(initialPath: string, ui: React.ReactElement): void {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  render(
    <QueryClientProvider client={client}>
      <MemoryRouter initialEntries={[initialPath]}>
        <Routes>
          <Route path="/forbidden-test" element={ui} />
          <Route path="/403" element={<div>403 page</div>} />
          <Route path="/login" element={<div>login page</div>} />
        </Routes>
      </MemoryRouter>
    </QueryClientProvider>,
  );
}

const mockUser = (role: AuthUser['role']): AuthUser => ({
  id: 'u-1',
  loginId: 'tester',
  role,
  roles: [role],
  locale: 'ja',
  factoryId: 'f-1',
});

describe('RoleGuard', () => {
  it('quality_admin が ApprovalSignPage に類するルートへアクセスできる', () => {
    vi.mocked(useAuth).mockReturnValue({ user: mockUser('quality_admin'), isLoading: false, isAuthenticated: true });
    renderWithRouter(
      '/forbidden-test',
      <RoleGuard roles={['quality_admin']}>
        <div>承認画面コンテンツ</div>
      </RoleGuard>,
    );
    expect(screen.getByText('承認画面コンテンツ')).toBeInTheDocument();
  });

  it('operator が quality_admin 限定の画面へアクセスすると 403 リダイレクト', () => {
    vi.mocked(useAuth).mockReturnValue({ user: mockUser('operator'), isLoading: false, isAuthenticated: true });
    renderWithRouter(
      '/forbidden-test',
      <RoleGuard roles={['quality_admin']}>
        <div>承認画面コンテンツ</div>
      </RoleGuard>,
    );
    expect(screen.getByText('403 page')).toBeInTheDocument();
  });

  it('未ログインなら /login にリダイレクトする', () => {
    vi.mocked(useAuth).mockReturnValue({ user: null, isLoading: false, isAuthenticated: false });
    renderWithRouter(
      '/forbidden-test',
      <RoleGuard roles={['quality_admin']}>
        <div>承認画面コンテンツ</div>
      </RoleGuard>,
    );
    expect(screen.getByText('login page')).toBeInTheDocument();
  });
});
