// 対応 §: ロードマップ §10.5 §10.6 §10.3.1 §20.1
// 端末→バックエンドの REST クライアント。
// 失敗は全て ApiError へ正規化し、UI 側で `t(error.i18nKey())` でローカライズ可能にする。

import { ApiError } from './api-error';

const TOKEN_KEY = 'wna.session.token';
const USER_KEY = 'wna.session.user';
const BACKEND_URL_KEY = 'wna.backend.url';

/** バックエンド URL を取得（QR ペアリング後） */
export function getBackendUrl(): string {
  return localStorage.getItem(BACKEND_URL_KEY) ?? 'http://localhost:8080';
}

/** バックエンド URL を設定 */
export function setBackendUrl(url: string): void {
  localStorage.setItem(BACKEND_URL_KEY, url);
}

/** トークン取得 */
export function getToken(): string | null {
  return localStorage.getItem(TOKEN_KEY);
}

/** ユーザ情報取得 */
export function getCurrentUser(): { user_id: string; display_name: string } | null {
  const raw = localStorage.getItem(USER_KEY);
  if (!raw) return null;
  try {
    return JSON.parse(raw) as { user_id: string; display_name: string };
  } catch {
    return null;
  }
}

/** ログイン */
export async function login(
  userId: string,
  password: string
): Promise<{ user_id: string; display_name: string; session_token: string }> {
  const res = await safeFetch(`${getBackendUrl()}/auth/login`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ user_id: userId, password })
  });
  if (!res.ok) throw ApiError.fromResponse(res);
  const json = (await res.json()) as { user_id: string; display_name: string; session_token: string };
  localStorage.setItem(TOKEN_KEY, json.session_token);
  localStorage.setItem(USER_KEY, JSON.stringify({ user_id: json.user_id, display_name: json.display_name }));
  return json;
}

/** ログアウト */
export function logout(): void {
  localStorage.removeItem(TOKEN_KEY);
  localStorage.removeItem(USER_KEY);
}

/** 認証付き fetch */
async function authFetch(path: string, init: RequestInit = {}): Promise<Response> {
  const token = getToken();
  if (!token) throw new ApiError('auth', null, false, 'no session');
  const headers = new Headers(init.headers);
  headers.set('Authorization', `Bearer ${token}`);
  if (!headers.has('Content-Type') && init.body) {
    headers.set('Content-Type', 'application/json');
  }
  return safeFetch(`${getBackendUrl()}${path}`, { ...init, headers });
}

/** fetch を ApiError 正規化付きでラップする */
async function safeFetch(input: RequestInfo, init?: RequestInit): Promise<Response> {
  try {
    return await fetch(input, init);
  } catch (e) {
    throw ApiError.fromNetwork(e);
  }
}

async function jsonOrThrow<T>(res: Response): Promise<T> {
  if (!res.ok) throw ApiError.fromResponse(res);
  return (await res.json()) as T;
}

async function voidOrThrow(res: Response): Promise<void> {
  if (!res.ok) throw ApiError.fromResponse(res);
}

/** タスク一覧 */
export interface TaskListItem {
  id: string;
  title: string | null;
  state: string;
  device_id: string;
  responsible_user: string | null;
  current_step_id: string | null;
  updated_at: string;
}
/** 増分同期結果。`since` 経由の呼び出しで 304 が返ると items は null */
export interface TaskListResult {
  items: TaskListItem[] | null;
  /** 次回 listTasks 呼び出しに渡すカーソル */
  cursor: string | null;
}
/** 既存呼び出し互換: 全件取得して配列を返す */
export async function listTasks(): Promise<TaskListItem[]> {
  return jsonOrThrow<TaskListItem[]>(await authFetch('/tasks'));
}
/** 増分取得。since 以降の更新行と Last-Modified カーソルを返す */
export async function listTasksSince(since: string | null): Promise<TaskListResult> {
  const path = since ? `/tasks?since=${encodeURIComponent(since)}` : '/tasks';
  const res = await authFetch(path);
  if (res.status === 304) {
    return { items: null, cursor: since };
  }
  if (!res.ok) throw ApiError.fromResponse(res);
  const items = (await res.json()) as TaskListItem[];
  const lastMod = res.headers.get('Last-Modified');
  return { items, cursor: lastMod ?? since };
}

/** タスク取得 */
export interface TaskDto {
  id: string;
  state: string;
  device_id: string;
  lamport: number;
  schema_version: number;
}
export async function getTask(id: string): Promise<TaskDto> {
  return jsonOrThrow<TaskDto>(await authFetch(`/tasks/${encodeURIComponent(id)}`));
}

/** タスク開始 */
export async function startTask(id: string): Promise<TaskDto> {
  return jsonOrThrow<TaskDto>(
    await authFetch(`/tasks/${encodeURIComponent(id)}/start`, { method: 'POST' })
  );
}

/** タスク完了 */
export async function completeTask(
  id: string,
  evidence: { manually_marked?: boolean; photo_attached?: boolean }
): Promise<TaskDto> {
  return jsonOrThrow<TaskDto>(
    await authFetch(`/tasks/${encodeURIComponent(id)}/complete`, {
      method: 'POST',
      body: JSON.stringify(evidence)
    })
  );
}

/** タスク中断 */
export async function suspendTask(id: string): Promise<TaskDto> {
  return jsonOrThrow<TaskDto>(
    await authFetch(`/tasks/${encodeURIComponent(id)}/suspend`, { method: 'POST' })
  );
}

/** タスク再開 */
export async function resumeTask(id: string): Promise<TaskDto> {
  return jsonOrThrow<TaskDto>(
    await authFetch(`/tasks/${encodeURIComponent(id)}/resume`, { method: 'POST' })
  );
}

/** ステップ一覧 */
export interface StepDto {
  id: string;
  sequence: number;
  label: string;
  completion_criteria: string;
  standard_time_seconds: number;
  done: boolean;
}
export async function listSteps(taskId: string): Promise<StepDto[]> {
  return jsonOrThrow<StepDto[]>(
    await authFetch(`/tasks/${encodeURIComponent(taskId)}/steps`)
  );
}

/** ステップを完了マーク */
export async function markStepDone(taskId: string, stepId: string): Promise<void> {
  return voidOrThrow(
    await authFetch(
      `/tasks/${encodeURIComponent(taskId)}/steps/${encodeURIComponent(stepId)}/done`,
      { method: 'POST' }
    )
  );
}

/** 実績追記 */
export async function appendRecord(
  taskId: string,
  deviceId: string,
  lamport: number,
  payload: Record<string, unknown>
): Promise<void> {
  return voidOrThrow(
    await authFetch(`/tasks/${encodeURIComponent(taskId)}/records`, {
      method: 'POST',
      body: JSON.stringify({ device_id: deviceId, lamport, payload })
    })
  );
}
