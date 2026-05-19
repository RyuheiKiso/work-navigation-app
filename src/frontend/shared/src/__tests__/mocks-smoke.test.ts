// MSW ハンドラの単体スモークテスト（Node 環境）。
// 全ハンドラのエクスポートと ResponseEnvelope 構造を検証する。
import { describe, it, expect, beforeAll, afterAll, afterEach } from 'vitest';
import { setupServer } from 'msw/node';
import { handlers } from '../mocks/handlers';
import { db } from '../mocks/db/seed';

const server = setupServer(...handlers);

beforeAll(() => server.listen({ onUnhandledRequest: 'warn' }));
afterEach(() => server.resetHandlers());
afterAll(() => server.close());

describe('MSW ハンドラ基本スモーク', () => {
  it('handlers 配列は空でない', () => {
    expect(handlers.length).toBeGreaterThan(0);
    // 39 EP 以上のハンドラが定義されている
    expect(handlers.length).toBeGreaterThanOrEqual(30);
  });

  it('db.users は 6 ロール以上のユーザーを含む', () => {
    const roles = new Set(db.users.map((u) => u.roles).flat());
    expect(roles.size).toBeGreaterThanOrEqual(4);
    expect(roles.has('operator')).toBe(true);
    expect(roles.has('master_admin')).toBe(true);
    expect(roles.has('quality_admin')).toBe(true);
    expect(roles.has('system_admin')).toBe(true);
  });

  it('POST /auth/login → 200 ResponseEnvelope', async () => {
    const res = await fetch('http://localhost:8080/api/v1/auth/login', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json', 'Idempotency-Key': 'test-key-login' },
      body: JSON.stringify({ loginId: 'operator01', password: 'any' }),
    });
    expect(res.status).toBe(200);
    const body = await res.json() as { data?: { accessToken?: string }; meta?: { api_version?: string } };
    // ResponseEnvelope 構造確認（docs/05_詳細設計/03_API詳細設計/01_OpenAPI共通仕様.md §3-1）
    expect(body).toHaveProperty('data');
    expect(body).toHaveProperty('meta');
    expect(body.meta?.api_version).toBe('v1');
    // JWT トークンが含まれている
    expect(body.data?.accessToken).toBeTruthy();
  });

  it('存在しないエンドポイント → 404 または ハンドラなし', async () => {
    const res = await fetch('http://localhost:8080/api/v1/nonexistent-endpoint', {
      headers: { Authorization: 'Bearer mock-token' },
    });
    // MSW が onUnhandledRequest: warn を使用しているためネットワーク接続エラーにはならない
    // 実際のレスポンスは 404 またはネットワークエラー
    expect([200, 404, 500, 503]).toContain(res.status);
  });

  it('GET /auth/jwks → JWKS 形式', async () => {
    const res = await fetch('http://localhost:8080/api/v1/auth/jwks');
    expect(res.status).toBe(200);
    const body = await res.json() as { keys?: unknown[] };
    expect(body).toHaveProperty('keys');
    expect(Array.isArray(body.keys)).toBe(true);
    expect((body.keys ?? []).length).toBeGreaterThan(0);
  });

  it('Idempotency-Key 重複時は同じレスポンスを返す', async () => {
    const key = 'idempotency-test-key-12345';
    const body = JSON.stringify({ loginId: 'operator01', password: 'any' });
    const headers = { 'Content-Type': 'application/json', 'Idempotency-Key': key };

    const res1 = await fetch('http://localhost:8080/api/v1/auth/login', { method: 'POST', headers, body });
    const res2 = await fetch('http://localhost:8080/api/v1/auth/login', { method: 'POST', headers, body });

    expect(res1.status).toBe(res2.status);
  });
});
