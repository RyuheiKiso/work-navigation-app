import { http, HttpResponse, type DefaultBodyType, type HttpHandler, type HttpResponseResolver, type PathParams } from 'msw';
import { v7 as uuidv7 } from 'uuid';
import { sha256 } from '@noble/hashes/sha2';
import { bytesToHex } from '@noble/hashes/utils';
import type { ErrorCode, ResponseMeta } from '../types';
import { db } from './db/seed';

export const TERMINAL_API_BASE = 'http://localhost:8080/api/v1';
export const MASTER_API_BASE = 'http://localhost:8081/api/v1';

export type ApiTarget = 'terminal' | 'master' | 'any';

function basesFor(target: ApiTarget): string[] {
  if (target === 'terminal') return [TERMINAL_API_BASE];
  if (target === 'master') return [MASTER_API_BASE];
  return [TERMINAL_API_BASE, MASTER_API_BASE];
}

// MSW ハンドラを HTTP method × target で簡潔に登録するためのヘルパ
type HttpMethod = 'get' | 'post' | 'put' | 'patch' | 'delete';

// eslint-disable-next-line @typescript-eslint/no-explicit-any
type ResolverArg<P extends PathParams = PathParams> = HttpResponseResolver<P, any, any>;

export function route<P extends PathParams = PathParams>(
  method: HttpMethod,
  target: ApiTarget,
  path: string,
  resolver: ResolverArg<P>,
): HttpHandler[] {
  return basesFor(target).map((base) => http[method]<P>(`${base}${path}`, resolver));
}

// 全エンドポイントで共通する meta オブジェクトを生成する（request_id は UUID v7）
export function buildMeta(): ResponseMeta {
  return {
    request_id: uuidv7(),
    server_time: new Date().toISOString(),
    api_version: 'v1',
  };
}

export function envelope<T>(data: T): { data: T; meta: ResponseMeta } {
  return { data, meta: buildMeta() };
}

export function paginatedEnvelope<T>(
  data: T[],
  total: number,
  page: number,
  perPage: number,
): { data: T[]; meta: ResponseMeta & { pagination: { total: number; page: number; per_page: number; total_pages: number } } } {
  return {
    data,
    meta: {
      ...buildMeta(),
      pagination: {
        total,
        page,
        per_page: perPage,
        total_pages: Math.max(1, Math.ceil(total / perPage)),
      },
    },
  };
}

export function cursorEnvelope<T>(
  data: T[],
  limit: number,
  hasMore: boolean,
  nextCursor: string | null,
): { data: T[]; meta: ResponseMeta & { limit: number; has_more: boolean; next_cursor: string | null } } {
  return {
    data,
    meta: {
      ...buildMeta(),
      limit,
      has_more: hasMore,
      next_cursor: nextCursor,
    },
  };
}

// RFC 9457 Problem Details 形式で application/problem+json を返却する
export function problem(
  status: number,
  errorId: ErrorCode,
  title: string,
  detail: string,
  options?: { instance?: string; violations?: Array<{ field: string; message: string; value?: unknown }> },
): HttpResponse<DefaultBodyType> {
  return HttpResponse.json(
    {
      type: `https://errors.wnav.example.com/${errorId}`,
      title,
      status,
      detail,
      instance: options?.instance,
      error_id: errorId,
      violations: options?.violations,
    },
    {
      status,
      headers: { 'Content-Type': 'application/problem+json' },
    },
  );
}

// Authorization ヘッダの Bearer JWT を抜き出して空 / blacklist チェックを行う
export function extractBearer(request: Request): string | null {
  const auth = request.headers.get('authorization') ?? request.headers.get('Authorization');
  if (!auth) return null;
  const match = /^Bearer\s+(.+)$/i.exec(auth);
  if (!match) return null;
  const token = match[1]!;
  if (db.jtiBlacklist.has(token)) return null;
  return token;
}

export function requireAuth(request: Request): HttpResponse<DefaultBodyType> | null {
  const token = extractBearer(request);
  if (!token) {
    return problem(401, 'ERR-AUTH-001', 'Unauthorized', '認証が必要です');
  }
  return null;
}

// ボディは Idempotency 比較に用いるため固定アルゴリズムでハッシュ化する
export async function hashBody(body: unknown): Promise<string> {
  const text = typeof body === 'string' ? body : JSON.stringify(body ?? null);
  const bytes = new TextEncoder().encode(text);
  return bytesToHex(sha256(bytes));
}

// Idempotency-Key の重複チェック（24h TTL）。同一キー + 同一ボディは前回レスポンスを返却する
export async function checkIdempotency(
  request: Request,
  body: unknown,
): Promise<{ cached: { response: unknown; status: number } | null; key: string | null; bodyHash: string | null; conflict: boolean }> {
  const key = request.headers.get('idempotency-key') ?? request.headers.get('Idempotency-Key');
  if (!key) return { cached: null, key: null, bodyHash: null, conflict: false };
  const bodyHash = await hashBody(body);
  const cached = db.idempotencyKeys.get(key);
  if (cached && cached.expiresAt > Date.now()) {
    if (cached.bodyHash !== bodyHash) {
      return { cached: null, key, bodyHash, conflict: true };
    }
    return { cached: { response: cached.response, status: cached.status }, key, bodyHash, conflict: false };
  }
  return { cached: null, key, bodyHash, conflict: false };
}

export function storeIdempotency(key: string | null, bodyHash: string | null, response: unknown, status: number): void {
  if (!key || !bodyHash) return;
  db.idempotencyKeys.set(key, {
    response,
    status,
    bodyHash,
    expiresAt: Date.now() + 24 * 60 * 60 * 1000,
  });
}

export interface IdempotencyResult<T> {
  cached?: { response: T; status: number };
  conflict?: HttpResponse<DefaultBodyType>;
  key: string | null;
  bodyHash: string | null;
}

export async function withIdempotency<T>(request: Request, body: unknown): Promise<IdempotencyResult<T>> {
  const result = await checkIdempotency(request, body);
  if (result.conflict) {
    return {
      conflict: problem(409, 'ERR-DB-001', 'idempotency_replay_conflict', '同じ Idempotency-Key で異なる Body の再送を検出しました'),
      key: result.key,
      bodyHash: result.bodyHash,
    };
  }
  if (result.cached) {
    return {
      cached: { response: result.cached.response as T, status: result.cached.status },
      key: result.key,
      bodyHash: result.bodyHash,
    };
  }
  return { key: result.key, bodyHash: result.bodyHash };
}

export function parsePagination(request: Request): { page: number; perPage: number } {
  const u = new URL(request.url);
  const page = Math.max(1, Number(u.searchParams.get('page') ?? '1'));
  const perPageRaw = Number(u.searchParams.get('per_page') ?? '50');
  const perPage = Math.min(200, Math.max(1, perPageRaw));
  return { page, perPage };
}

export function paginate<T>(items: T[], page: number, perPage: number): { slice: T[]; total: number } {
  const start = (page - 1) * perPage;
  return { slice: items.slice(start, start + perPage), total: items.length };
}

export { http, HttpResponse };
