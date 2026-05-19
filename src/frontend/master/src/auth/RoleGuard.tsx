import type React from 'react';
import { Navigate } from 'react-router-dom';
import type { UserRole } from '@wnav/shared/types';
import { useAuth } from './useAuth';

// 許可ロール以外はアクセス禁止（画面層での認可）。API 層でも独立に検証する（src/frontend/master/CLAUDE.md §認証・認可）。
export function RoleGuard({
  roles,
  children,
}: {
  roles: readonly UserRole[];
  children: React.ReactNode;
}): React.ReactElement {
  const { user } = useAuth();
  if (!user) return <Navigate to="/login" replace />;
  const allowed = user.roles.some((r) => roles.includes(r));
  if (!allowed) return <Navigate to="/403" replace />;
  return <>{children}</>;
}
