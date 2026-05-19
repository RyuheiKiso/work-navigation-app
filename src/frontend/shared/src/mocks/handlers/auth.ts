import { v7 as uuidv7 } from 'uuid';
import {
  HttpResponse,
  envelope,
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
    return HttpResponse.json(envelope(response), { status: 200 });
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

  ...route('get', 'any', '/auth/jwks', () => {
    const jwks: Jwks = {
      keys: [
        { kty: 'RSA', use: 'sig', kid: '2026-Q2', alg: 'RS256', n: 'mock-n-value', e: 'AQAB' },
      ],
    };
    return HttpResponse.json(jwks);
  }),
];
