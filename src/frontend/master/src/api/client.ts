import createClient from 'openapi-fetch';
import type { Middleware } from 'openapi-fetch';
import type {
  ApiResponse,
  PaginatedResponse,
  ProblemDetails,
} from '@wnav/shared/types';

// OpenAPI 生成型はビルド時に shared/openapi/generated/api.d.ts が存在しなくても fallback で動作させる
// 完全な型注入は CI の openapi-typescript 生成タスク完了後に有効になる
type Paths = Record<string, unknown>;

const BASE_URL: string = (import.meta.env['VITE_API_BASE_URL'] as string | undefined) ?? '/api/v1';

// MSW は httpOnly Cookie を設定できないため DEV モードのみ sessionStorage でトークンを補完する
const DEV_TOKEN_KEY = 'wnav_dev_token';

export function storeDevToken(token: string): void {
  if (import.meta.env.DEV) sessionStorage.setItem(DEV_TOKEN_KEY, token);
}

export function clearDevToken(): void {
  if (import.meta.env.DEV) sessionStorage.removeItem(DEV_TOKEN_KEY);
}

// 全リクエストに Accept-Language と Idempotency-Key 候補のヘッダ補完を行う
const headerMiddleware: Middleware = {
  async onRequest({ request }) {
    if (!request.headers.has('Accept-Language')) {
      request.headers.set('Accept-Language', navigator.language || 'ja');
    }
    // 本番では httpOnly Cookie を使用するが、MSW モックでは Cookie が設定できないため
    // DEV モードのみ sessionStorage のトークンを Authorization ヘッダに補完する
    if (import.meta.env.DEV && !request.headers.has('Authorization')) {
      const token = sessionStorage.getItem(DEV_TOKEN_KEY);
      if (token) request.headers.set('Authorization', `Bearer ${token}`);
    }
    return request;
  },
};

export const apiClient = createClient<Paths>({
  baseUrl: BASE_URL,
  credentials: 'include',
});

apiClient.use(headerMiddleware);

export const API_BASE_URL = BASE_URL;

// クライアント側の自前 fetch ラッパ。openapi 型が未生成のため、ジェネリック型で穴を埋める。
export class ApiError extends Error {
  constructor(public problem: ProblemDetails, public status: number) {
    super(problem.detail || problem.title);
    this.name = 'ApiError';
  }
}

async function request<T>(method: string, path: string, body?: unknown, init?: RequestInit): Promise<T> {
  const headers = new Headers(init?.headers);
  if (!headers.has('Accept-Language')) headers.set('Accept-Language', navigator.language || 'ja');
  if (body !== undefined) headers.set('Content-Type', 'application/json');
  if (method !== 'GET' && method !== 'HEAD' && !headers.has('Idempotency-Key')) {
    headers.set('Idempotency-Key', crypto.randomUUID());
  }
  // DEV モードのみ: MSW が Cookie を設定できないため sessionStorage のトークンを Bearer ヘッダに補完する
  if (import.meta.env.DEV && !headers.has('Authorization')) {
    const token = sessionStorage.getItem(DEV_TOKEN_KEY);
    if (token) headers.set('Authorization', `Bearer ${token}`);
  }
  const res = await fetch(`${BASE_URL}${path}`, {
    method,
    headers,
    credentials: 'include',
    body: body !== undefined ? JSON.stringify(body) : undefined,
    ...init,
  });
  if (!res.ok) {
    const problem = (await res.json().catch(() => ({
      type: 'about:blank',
      title: res.statusText,
      status: res.status,
      detail: res.statusText,
    }))) as ProblemDetails;
    throw new ApiError(problem, res.status);
  }
  if (res.status === 204) return undefined as T;
  return (await res.json()) as T;
}

// 型安全な薄ラッパ。@wnav/shared/types に定義済の Envelope/PageMeta を流用する。
export const api = {
  get: <T>(path: string): Promise<ApiResponse<T>> => request<ApiResponse<T>>('GET', path),
  getList: <T>(path: string): Promise<PaginatedResponse<T>> => request<PaginatedResponse<T>>('GET', path),
  post: <T>(path: string, body?: unknown): Promise<ApiResponse<T>> => request<ApiResponse<T>>('POST', path, body),
  patch: <T>(path: string, body?: unknown): Promise<ApiResponse<T>> => request<ApiResponse<T>>('PATCH', path, body),
  put: <T>(path: string, body?: unknown): Promise<ApiResponse<T>> => request<ApiResponse<T>>('PUT', path, body),
  delete: (path: string): Promise<void> => request<void>('DELETE', path),
};
