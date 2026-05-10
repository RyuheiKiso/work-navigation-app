// 対応 §: ロードマップ §10.2 §10.3.1 §10.5 §11.4.1
// 設定 UI → バックエンド REST クライアント。

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
  const res = await fetch(`${getBackendUrl()}/auth/login`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ user_id: userId, password })
  });
  if (!res.ok) throw new Error(`ログイン失敗（HTTP ${res.status}）`);
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
  if (!token) throw new Error('未ログイン');
  const headers = new Headers(init.headers);
  headers.set('Authorization', `Bearer ${token}`);
  if (!headers.has('Content-Type') && init.body) headers.set('Content-Type', 'application/json');
  return fetch(`${getBackendUrl()}${path}`, { ...init, headers });
}

// ===== マスタ =====
export interface MasterRow { code: string; name: string; extra: string | null }

export async function listProducts(): Promise<MasterRow[]> {
  const r = await authFetch('/master/products');
  if (!r.ok) throw new Error(`HTTP ${r.status}`);
  return (await r.json()) as MasterRow[];
}
export async function upsertProduct(row: MasterRow): Promise<void> {
  const r = await authFetch('/master/products', { method: 'PUT', body: JSON.stringify(row) });
  if (!r.ok) throw new Error(`HTTP ${r.status}`);
}
export async function deleteProduct(code: string): Promise<void> {
  const r = await authFetch(`/master/products/${encodeURIComponent(code)}`, { method: 'DELETE' });
  if (!r.ok) throw new Error(`HTTP ${r.status}`);
}

export async function listEquipments(): Promise<MasterRow[]> {
  const r = await authFetch('/master/equipments');
  if (!r.ok) throw new Error(`HTTP ${r.status}`);
  return (await r.json()) as MasterRow[];
}
export async function upsertEquipment(row: MasterRow): Promise<void> {
  const r = await authFetch('/master/equipments', { method: 'PUT', body: JSON.stringify(row) });
  if (!r.ok) throw new Error(`HTTP ${r.status}`);
}
export async function deleteEquipment(code: string): Promise<void> {
  const r = await authFetch(`/master/equipments/${encodeURIComponent(code)}`, { method: 'DELETE' });
  if (!r.ok) throw new Error(`HTTP ${r.status}`);
}

export async function listParts(): Promise<MasterRow[]> {
  const r = await authFetch('/master/parts');
  if (!r.ok) throw new Error(`HTTP ${r.status}`);
  return (await r.json()) as MasterRow[];
}
export async function upsertPart(row: MasterRow): Promise<void> {
  const r = await authFetch('/master/parts', { method: 'PUT', body: JSON.stringify(row) });
  if (!r.ok) throw new Error(`HTTP ${r.status}`);
}
export async function deletePart(code: string): Promise<void> {
  const r = await authFetch(`/master/parts/${encodeURIComponent(code)}`, { method: 'DELETE' });
  if (!r.ok) throw new Error(`HTTP ${r.status}`);
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
  const r = await authFetch(`/audit?limit=${limit}`);
  if (!r.ok) throw new Error(`HTTP ${r.status}`);
  return (await r.json()) as AuditRow[];
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
  const r = await authFetch('/dashboard/tasks');
  if (!r.ok) throw new Error(`HTTP ${r.status}`);
  return (await r.json()) as DashboardTask[];
}

// ===== フロー =====
export interface FlowSummary {
  id: string; version: number; name: string; status: string; industry: string | null;
}
export async function listFlows(): Promise<FlowSummary[]> {
  const r = await authFetch('/flows');
  if (!r.ok) throw new Error(`HTTP ${r.status}`);
  return (await r.json()) as FlowSummary[];
}

export async function publishTrial(
  flowId: string,
  body: { version: number; name: string; industry: string | null; body: unknown; pilot_device_ids: string[] }
): Promise<{ flow_id: string; version: number; status: string; pilot_device_ids: string[] }> {
  const r = await authFetch(`/flows/${encodeURIComponent(flowId)}/trials`, {
    method: 'POST',
    body: JSON.stringify(body)
  });
  if (!r.ok) throw new Error(`HTTP ${r.status}`);
  return (await r.json()) as { flow_id: string; version: number; status: string; pilot_device_ids: string[] };
}
