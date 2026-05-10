// 対応 §: ロードマップ §10.5 §10.6 §20.1 §11.4
// バックエンドからの失敗を分類した例外型。
// `HTTP 403` のような技術的文字列を UI へ漏らさず、ユーザー向け i18n キー
// (`error.api.<kind>`) に対応付ける。再試行可能性 (retriable) も付与する。

export type ApiErrorKind =
  | 'network'
  | 'timeout'
  | 'auth'
  | 'forbidden'
  | 'not_found'
  | 'conflict'
  | 'rate_limited'
  | 'server'
  | 'unknown';

export class ApiError extends Error {
  constructor(
    public readonly kind: ApiErrorKind,
    public readonly httpStatus: number | null,
    public readonly retriable: boolean,
    message: string
  ) {
    super(message);
    this.name = 'ApiError';
  }

  /** Response の status から ApiError を構築する */
  static fromResponse(res: Response): ApiError {
    const s = res.status;
    if (s === 401) return new ApiError('auth', s, false, `HTTP ${s}`);
    if (s === 403) return new ApiError('forbidden', s, false, `HTTP ${s}`);
    if (s === 404) return new ApiError('not_found', s, false, `HTTP ${s}`);
    if (s === 409) return new ApiError('conflict', s, false, `HTTP ${s}`);
    if (s === 429) return new ApiError('rate_limited', s, true, `HTTP ${s}`);
    if (s >= 500) return new ApiError('server', s, true, `HTTP ${s}`);
    return new ApiError('unknown', s, false, `HTTP ${s}`);
  }

  /** fetch そのものが失敗したケース (ネットワーク到達不可・AbortError 等) */
  static fromNetwork(e: unknown): ApiError {
    if (e instanceof DOMException && e.name === 'AbortError') {
      return new ApiError('timeout', null, true, 'aborted');
    }
    const msg = e instanceof Error ? e.message : String(e);
    return new ApiError('network', null, true, msg);
  }

  /** UI 側で `t()` に渡すための i18n キー */
  i18nKey(): string {
    return `error.api.${this.kind}`;
  }
}

/** 既存エラーを ApiError に正規化する。ApiError ならそのまま、それ以外は unknown 包装 */
export function toApiError(e: unknown): ApiError {
  if (e instanceof ApiError) return e;
  if (e instanceof TypeError) return ApiError.fromNetwork(e);
  if (e instanceof Error) return new ApiError('unknown', null, false, e.message);
  return new ApiError('unknown', null, false, String(e));
}
