import { v7 as uuidv7 } from 'uuid';
import {
  HttpResponse,
  envelope,
  extractBearer,
  problem,
  requireAuth,
  route,
  storeIdempotency,
  withIdempotency,
} from '../_helpers';
import { db } from '../db/seed';
import type {
  AuthLoginRequest,
  AuthLoginResponse,
  AuthRefreshRequest,
  AuthRefreshResponse,
  Jwks,
} from '../../types';

// JWT 署名検証はモックなので audience を含むフェイク文字列で代替する
function fakeJwt(userId: string, audience: 'terminal-api' | 'master-api'): string {
  return `mock-jwt.${audience}.${userId}.${Date.now()}`;
}

export const authHandlers = [
  ...route('post', 'any', '/auth/login', async ({ request }) => {
    const body = (await request.json().catch(() => null)) as AuthLoginRequest | null;
    if (!body) return problem(422, 'ERR-VAL-001', 'Required field missing', 'リクエストボディが必要です');

    const idem = await withIdempotency<AuthLoginResponse>(request, body);
    if (idem.conflict) return idem.conflict;
    if (idem.cached) return HttpResponse.json(envelope(idem.cached.response), { status: idem.cached.status });

    const user = db.users.find((u) => u.loginId === body.loginId);
    if (!user) {
      return problem(401, 'ERR-AUTH-001', 'Unauthorized', 'ユーザー ID またはパスワードが正しくありません');
    }

    const audience = request.url.includes('8080') ? 'terminal-api' : 'master-api';
    const accessToken = fakeJwt(user.id, audience);
    const refreshToken = uuidv7();
    db.refreshTokens.set(refreshToken, { userId: user.id, expiresAt: Date.now() + 7 * 86400000 });

    const response: AuthLoginResponse = {
      accessToken,
      refreshToken,
      tokenType: 'Bearer',
      expiresIn: 28800,
      refreshExpiresIn: 604800,
      roles: user.roles,
      userId: user.id,
      factoryId: user.factoryId,
    };
    storeIdempotency(idem.key, idem.bodyHash, response, 200);
    // モック環境では httpOnly Cookie が使えないため通常 Cookie で代替する
    return HttpResponse.json(envelope(response), {
      status: 200,
      headers: { 'Set-Cookie': `wnav_mock_token=${accessToken}; Path=/; SameSite=Lax` },
    });
  }),

  ...route('post', 'any', '/auth/refresh', async ({ request }) => {
    const body = (await request.json().catch(() => null)) as AuthRefreshRequest | null;
    if (!body?.refreshToken) {
      return problem(422, 'ERR-VAL-001', 'required_field_missing', 'refresh_token は必須です');
    }
    const cached = db.refreshTokens.get(body.refreshToken);
    if (!cached || cached.expiresAt < Date.now()) {
      return problem(401, 'ERR-AUTH-001', 'Unauthorized', 'refresh_token が失効しています');
    }
    const audience = request.url.includes('8080') ? 'terminal-api' : 'master-api';
    const response: AuthRefreshResponse = {
      accessToken: fakeJwt(cached.userId, audience),
      tokenType: 'Bearer',
      expiresIn: 28800,
    };
    return HttpResponse.json(envelope(response));
  }),

  ...route('post', 'any', '/auth/logout', ({ request }) => {
    const authErr = requireAuth(request);
    if (authErr) return authErr;
    const token = request.headers.get('authorization')?.replace(/^Bearer\s+/i, '');
    if (token) db.jtiBlacklist.add(token);
    return new HttpResponse(null, { status: 204 });
  }),

  // Cookie ベースのセッション確認エンドポイント（master-api 専用）
  ...route('get', 'master', '/auth/me', ({ request }) => {
    const cookieHeader = request.headers.get('cookie') ?? '';
    const cookieMatch = /wnav_mock_token=([^\s;]+)/.exec(cookieHeader);
    // Authorization ヘッダも許容（将来の Bearer 移行に備えるフォールバック）
    const bearerToken = extractBearer(request);
    const token = bearerToken ?? (cookieMatch ? cookieMatch[1] : null);

    if (!token) return problem(401, 'ERR-AUTH-001', 'Unauthorized', '認証が必要です');
    if (db.jtiBlacklist.has(token)) return problem(401, 'ERR-AUTH-001', 'Unauthorized', 'トークンは無効化済みです');

    // fakeJwt 形式: mock-jwt.{audience}.{userId}.{ts}
    const parts = token.split('.');
    const userId = parts[2];
    if (!userId) return problem(401, 'ERR-AUTH-001', 'Unauthorized', 'トークン形式が不正です');

    const user = db.users.find((u) => u.id === userId);
    if (!user) return problem(401, 'ERR-AUTH-001', 'Unauthorized', 'ユーザーが見つかりません');

    return HttpResponse.json(envelope({
      id: user.id,
      loginId: user.loginId,
      role: user.role,
      roles: user.roles,
      locale: user.locale,
      factoryId: user.factoryId,
    }));
  }),

  ...route('get', 'any', '/auth/jwks', () => {
    const jwks: Jwks = {
      keys: [
        { kty: 'RSA', use: 'sig', kid: '2026-Q2', alg: 'RS256', n: 'mock-n-value', e: 'AQAB' },
      ],
    };
    return HttpResponse.json(jwks);
  }),
];
