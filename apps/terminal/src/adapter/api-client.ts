// 対応 §: ロードマップ §10.5 §10.6 §10.3.1
// 端末→バックエンドの REST クライアント。
// セッショントークンを localStorage に永続化して継続ログイン（§10.5.1）。

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
  const res = await fetch(`${getBackendUrl()}/auth/login`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ user_id: userId, password })
  });
  if (!res.ok) {
    throw new Error(`ログイン失敗（HTTP ${res.status}）`);
  }
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
  if (!token) throw new Error('未ログイン');
  const headers = new Headers(init.headers);
  headers.set('Authorization', `Bearer ${token}`);
  if (!headers.has('Content-Type') && init.body) {
    headers.set('Content-Type', 'application/json');
  }
  return fetch(`${getBackendUrl()}${path}`, { ...init, headers });
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
export async function listTasks(): Promise<TaskListItem[]> {
  const res = await authFetch('/tasks');
  if (!res.ok) throw new Error(`タスク一覧取得失敗（HTTP ${res.status}）`);
  return (await res.json()) as TaskListItem[];
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
  const res = await authFetch(`/tasks/${encodeURIComponent(id)}`);
  if (!res.ok) throw new Error(`タスク取得失敗（HTTP ${res.status}）`);
  return (await res.json()) as TaskDto;
}

/** タスク開始 */
export async function startTask(id: string): Promise<TaskDto> {
  const res = await authFetch(`/tasks/${encodeURIComponent(id)}/start`, { method: 'POST' });
  if (!res.ok) throw new Error(`開始失敗（HTTP ${res.status}）`);
  return (await res.json()) as TaskDto;
}

/** タスク完了 */
export async function completeTask(
  id: string,
  evidence: { manually_marked?: boolean; photo_attached?: boolean }
): Promise<TaskDto> {
  const res = await authFetch(`/tasks/${encodeURIComponent(id)}/complete`, {
    method: 'POST',
    body: JSON.stringify(evidence)
  });
  if (!res.ok) throw new Error(`完了失敗（HTTP ${res.status}）`);
  return (await res.json()) as TaskDto;
}

/** タスク中断 */
export async function suspendTask(id: string): Promise<TaskDto> {
  const res = await authFetch(`/tasks/${encodeURIComponent(id)}/suspend`, { method: 'POST' });
  if (!res.ok) throw new Error(`中断失敗（HTTP ${res.status}）`);
  return (await res.json()) as TaskDto;
}

/** タスク再開 */
export async function resumeTask(id: string): Promise<TaskDto> {
  const res = await authFetch(`/tasks/${encodeURIComponent(id)}/resume`, { method: 'POST' });
  if (!res.ok) throw new Error(`再開失敗（HTTP ${res.status}）`);
  return (await res.json()) as TaskDto;
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
  const res = await authFetch(`/tasks/${encodeURIComponent(taskId)}/steps`);
  if (!res.ok) throw new Error(`ステップ取得失敗（HTTP ${res.status}）`);
  return (await res.json()) as StepDto[];
}

/** ステップを完了マーク */
export async function markStepDone(taskId: string, stepId: string): Promise<void> {
  const res = await authFetch(
    `/tasks/${encodeURIComponent(taskId)}/steps/${encodeURIComponent(stepId)}/done`,
    { method: 'POST' }
  );
  if (!res.ok) throw new Error(`ステップ完了失敗（HTTP ${res.status}）`);
}

/** 実績追記 */
export async function appendRecord(
  taskId: string,
  deviceId: string,
  lamport: number,
  payload: Record<string, unknown>
): Promise<void> {
  const res = await authFetch(`/tasks/${encodeURIComponent(taskId)}/records`, {
    method: 'POST',
    body: JSON.stringify({ device_id: deviceId, lamport, payload })
  });
  if (!res.ok) throw new Error(`実績追記失敗（HTTP ${res.status}）`);
}
