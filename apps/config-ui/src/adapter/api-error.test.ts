// 対応 §: ロードマップ §13.1 §10.5
// ApiError 分類の単体テスト。

import { describe, it, expect } from 'vitest';
import { ApiError, toApiError } from './api-error';

function makeRes(status: number): Response {
  // 最小限の Response 風 オブジェクト（status だけ参照される）
  return new Response(null, { status });
}

describe('ApiError.fromResponse', () => {
  it('classifies 401 as auth', () => {
    const e = ApiError.fromResponse(makeRes(401));
    expect(e.kind).toBe('auth');
    expect(e.retriable).toBe(false);
  });

  it('classifies 403 as forbidden', () => {
    expect(ApiError.fromResponse(makeRes(403)).kind).toBe('forbidden');
  });

  it('classifies 404 as not_found', () => {
    expect(ApiError.fromResponse(makeRes(404)).kind).toBe('not_found');
  });

  it('classifies 409 as conflict', () => {
    expect(ApiError.fromResponse(makeRes(409)).kind).toBe('conflict');
  });

  it('classifies 429 as rate_limited and retriable', () => {
    const e = ApiError.fromResponse(makeRes(429));
    expect(e.kind).toBe('rate_limited');
    expect(e.retriable).toBe(true);
  });

  it('classifies 5xx as server and retriable', () => {
    const e = ApiError.fromResponse(makeRes(503));
    expect(e.kind).toBe('server');
    expect(e.retriable).toBe(true);
  });

  it('classifies 418 as unknown', () => {
    expect(ApiError.fromResponse(makeRes(418)).kind).toBe('unknown');
  });
});

describe('ApiError.fromNetwork', () => {
  it('classifies AbortError as timeout', () => {
    const e = ApiError.fromNetwork(new DOMException('aborted', 'AbortError'));
    expect(e.kind).toBe('timeout');
    expect(e.retriable).toBe(true);
  });

  it('classifies generic errors as network', () => {
    const e = ApiError.fromNetwork(new Error('boom'));
    expect(e.kind).toBe('network');
    expect(e.retriable).toBe(true);
  });
});

describe('toApiError', () => {
  it('passes through ApiError unchanged', () => {
    const original = new ApiError('forbidden', 403, false, 'HTTP 403');
    expect(toApiError(original)).toBe(original);
  });

  it('wraps TypeError as network', () => {
    expect(toApiError(new TypeError('Failed to fetch')).kind).toBe('network');
  });

  it('wraps generic Error as unknown', () => {
    expect(toApiError(new Error('oops')).kind).toBe('unknown');
  });
});

describe('ApiError.i18nKey', () => {
  it('returns the canonical key path', () => {
    expect(new ApiError('forbidden', 403, false, '').i18nKey()).toBe('error.api.forbidden');
    expect(new ApiError('network', null, true, '').i18nKey()).toBe('error.api.network');
  });
});
