// 対応 §: ロードマップ §10.2.3
// 試行版発行 (POST /flows/<id>/trials) のフェッチ呼出を分離する。
// JSX onClick からこの関数を呼ぶ形にしてビュー層を薄く保つ。

import type { Flow } from '../../domain/flow';

export interface TrialPublishResult {
  flow_id: string;
  version: number;
  status: string;
  pilot_device_ids: string[];
}

export async function publishTrial(
  flow: Flow,
  pilotDeviceIds: string[] = ['terminal-001']
): Promise<TrialPublishResult> {
  const body = {
    version: flow.version,
    name: flow.name,
    industry: flow.industry ?? null,
    body: { nodes: flow.nodes, edges: flow.edges },
    pilot_device_ids: pilotDeviceIds
  };
  const token = localStorage.getItem('wna.session.token');
  const backendUrl = localStorage.getItem('wna.backend.url') ?? 'http://localhost:8080';
  const res = await fetch(`${backendUrl}/flows/${encodeURIComponent(flow.id)}/trials`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      Authorization: `Bearer ${token ?? ''}`
    },
    body: JSON.stringify(body)
  });
  if (!res.ok) throw new Error(`HTTP ${res.status}`);
  return (await res.json()) as TrialPublishResult;
}
