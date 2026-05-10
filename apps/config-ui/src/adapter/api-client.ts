// 対応 §: ロードマップ §10.2 §10.3.1 §10.5 §11.4.1 §20.1
// 設定 UI → バックエンド REST クライアント。失敗は ApiError へ正規化する。

import { ApiError } from './api-error';

const TOKEN_KEY = 'wna.session.token';
const USER_KEY = 'wna.session.user';
const BACKEND_URL_KEY = 'wna.backend.url';

export function getBackendUrl(): string {
  return localStorage.getItem(BACKEND_URL_KEY) ?? 'http://localhost:8080';
}
export function setBackendUrl(url: string): void {
  localStorage.setItem(BACKEND_URL_KEY, url);
}
export function getToken(): string | null {
  return localStorage.getItem(TOKEN_KEY);
}
export function getCurrentUser(): { user_id: string; display_name: string } | null {
  const raw = localStorage.getItem(USER_KEY);
  if (!raw) return null;
  try {
    return JSON.parse(raw) as { user_id: string; display_name: string };
  } catch {
    return null;
  }
}

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

export function logout(): void {
  localStorage.removeItem(TOKEN_KEY);
  localStorage.removeItem(USER_KEY);
}

async function authFetch(path: string, init: RequestInit = {}): Promise<Response> {
  const token = getToken();
  if (!token) throw new ApiError('auth', null, false, 'no session');
  const headers = new Headers(init.headers);
  headers.set('Authorization', `Bearer ${token}`);
  if (!headers.has('Content-Type') && init.body) headers.set('Content-Type', 'application/json');
  return safeFetch(`${getBackendUrl()}${path}`, { ...init, headers });
}

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

// ===== マスタ =====
export interface MasterRow { code: string; name: string; extra: string | null }

export async function listProducts(): Promise<MasterRow[]> {
  return jsonOrThrow<MasterRow[]>(await authFetch('/master/products'));
}
export async function upsertProduct(row: MasterRow): Promise<void> {
  return voidOrThrow(await authFetch('/master/products', { method: 'PUT', body: JSON.stringify(row) }));
}
export async function deleteProduct(code: string): Promise<void> {
  return voidOrThrow(await authFetch(`/master/products/${encodeURIComponent(code)}`, { method: 'DELETE' }));
}

export async function listEquipments(): Promise<MasterRow[]> {
  return jsonOrThrow<MasterRow[]>(await authFetch('/master/equipments'));
}
export async function upsertEquipment(row: MasterRow): Promise<void> {
  return voidOrThrow(await authFetch('/master/equipments', { method: 'PUT', body: JSON.stringify(row) }));
}
export async function deleteEquipment(code: string): Promise<void> {
  return voidOrThrow(await authFetch(`/master/equipments/${encodeURIComponent(code)}`, { method: 'DELETE' }));
}

export async function listParts(): Promise<MasterRow[]> {
  return jsonOrThrow<MasterRow[]>(await authFetch('/master/parts'));
}
export async function upsertPart(row: MasterRow): Promise<void> {
  return voidOrThrow(await authFetch('/master/parts', { method: 'PUT', body: JSON.stringify(row) }));
}
export async function deletePart(code: string): Promise<void> {
  return voidOrThrow(await authFetch(`/master/parts/${encodeURIComponent(code)}`, { method: 'DELETE' }));
}

// ===== 監査 =====
export interface AuditRow {
  id: string;
  actor_id: string;
  action: string;
  target_id: string | null;
  terminal_time: string | null;
  server_time: string;
  payload: string | null;
}
export async function listAudit(limit = 100): Promise<AuditRow[]> {
  return jsonOrThrow<AuditRow[]>(await authFetch(`/audit?limit=${limit}`));
}

// ===== 班長ダッシュボード =====
export interface DashboardTask {
  id: string;
  title: string | null;
  state: string;
  device_id: string;
  responsible_user: string | null;
  current_step_id: string | null;
  updated_at: string;
}
export async function listDashboardTasks(): Promise<DashboardTask[]> {
  return jsonOrThrow<DashboardTask[]>(await authFetch('/dashboard/tasks'));
}

// ===== フロー =====
export interface FlowSummary {
  id: string; version: number; name: string; status: string; industry: string | null;
}
export async function listFlows(): Promise<FlowSummary[]> {
  return jsonOrThrow<FlowSummary[]>(await authFetch('/flows'));
}

export async function publishTrial(
  flowId: string,
  body: { version: number; name: string; industry: string | null; body: unknown; pilot_device_ids: string[] }
): Promise<{ flow_id: string; version: number; status: string; pilot_device_ids: string[] }> {
  return jsonOrThrow<{ flow_id: string; version: number; status: string; pilot_device_ids: string[] }>(
    await authFetch(`/flows/${encodeURIComponent(flowId)}/trials`, {
      method: 'POST',
      body: JSON.stringify(body)
    })
  );
}
