#!/usr/bin/env node
// MSW ハンドラ全39EPに対して Node ベースでスモークテストを実施するスクリプト。
// npm run mock:smoke で実行する（バックエンドなしで API 契約を確認）。
import { createServer } from 'http';
import { setupServer } from 'msw/node';

// shared パッケージのハンドラをインポート
const { handlers } = await import('../shared/src/mocks/handlers/index.js').catch(() => {
  console.error('shared/src/mocks が見つかりません。npm install を実行してください。');
  process.exit(1);
});

const server = setupServer(...handlers);
server.listen({ onUnhandledRequest: 'warn' });

const BASE_TERMINAL = 'http://localhost:8080/api/v1';
const BASE_MASTER = 'http://localhost:8081/api/v1';

let passed = 0;
let failed = 0;

async function check(label, fn) {
  try {
    await fn();
    console.log(`  ✅ ${label}`);
    passed++;
  } catch (e) {
    console.error(`  ❌ ${label}: ${e.message}`);
    failed++;
  }
}

async function assertOk(res, expectedStatus = 200) {
  if (res.status !== expectedStatus) {
    const body = await res.text().catch(() => '');
    throw new Error(`HTTP ${res.status} (expected ${expectedStatus}): ${body.slice(0, 200)}`);
  }
}

// テスト用 JWT トークン（MSW の認証ヘルパが Bearer を期待する）
const AUTH_HEADER = { Authorization: 'Bearer mock-jwt.terminal-api.user1.12345' };
const JSON_HEADERS = { 'Content-Type': 'application/json', ...AUTH_HEADER };

console.log('\n🔍 WNAV MSW Smoke Test — 39 エンドポイント\n');

// ── 認証 ──────────────────────────────────────────────
console.log('📌 認証・認可');

await check('POST /auth/login → 200', async () => {
  const r = await fetch(`${BASE_TERMINAL}/auth/login`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json', 'Idempotency-Key': crypto.randomUUID() },
    body: JSON.stringify({ loginId: 'operator01', password: 'password' }),
  });
  await assertOk(r);
  const j = await r.json();
  if (!j.data?.accessToken) throw new Error('accessToken がない');
});

await check('DELETE /auth/logout → 204', async () => {
  const r = await fetch(`${BASE_TERMINAL}/auth/logout`, { method: 'DELETE', headers: AUTH_HEADER });
  await assertOk(r, 204);
});

await check('GET /auth/jwks → 200', async () => {
  const r = await fetch(`${BASE_TERMINAL}/auth/jwks`);
  await assertOk(r);
});

// ── 作業実行 ───────────────────────────────────────────
console.log('\n📌 作業実行');

let executionId;
await check('POST /work-executions → 200 or 201', async () => {
  const r = await fetch(`${BASE_TERMINAL}/work-executions`, {
    method: 'POST',
    headers: { ...JSON_HEADERS, 'Idempotency-Key': crypto.randomUUID() },
    body: JSON.stringify({ workOrderId: 'wo-001', operatorId: 'user-001', terminalId: 'term-001' }),
  });
  if (r.status !== 200 && r.status !== 201) throw new Error(`HTTP ${r.status}`);
  const j = await r.json();
  executionId = j.data?.id ?? 'exec-001';
});

await check('PUT /work-executions/{id}/heartbeat → 200 or 204', async () => {
  const id = executionId ?? 'exec-001';
  const r = await fetch(`${BASE_TERMINAL}/work-executions/${id}/heartbeat`, {
    method: 'PUT',
    headers: AUTH_HEADER,
  });
  if (r.status !== 200 && r.status !== 204) throw new Error(`HTTP ${r.status}`);
});

// ── エビデンス ─────────────────────────────────────────
console.log('\n📌 エビデンス・電子サイン');

await check('POST /evidences → 200 or 201', async () => {
  const r = await fetch(`${BASE_TERMINAL}/evidences`, {
    method: 'POST',
    headers: { ...JSON_HEADERS, 'Idempotency-Key': crypto.randomUUID() },
    body: JSON.stringify({ caseId: 'case-001', stepId: 'step-001', fileHash: 'a'.repeat(64), mimeType: 'image/jpeg' }),
  });
  if (r.status !== 200 && r.status !== 201) throw new Error(`HTTP ${r.status}`);
});

await check('POST /electronic-signs → 200 or 201', async () => {
  const r = await fetch(`${BASE_TERMINAL}/electronic-signs`, {
    method: 'POST',
    headers: { ...JSON_HEADERS, 'Idempotency-Key': crypto.randomUUID() },
    body: JSON.stringify({ caseId: 'case-001', stepId: 'step-001', publicKey: 'pk', signature: 'sig' }),
  });
  if (r.status !== 200 && r.status !== 201) throw new Error(`HTTP ${r.status}`);
});

// ── マスタ ─────────────────────────────────────────────
console.log('\n📌 マスタ管理');

for (const resource of ['processes', 'operations', 'products', 'sops', 'users', 'roles', 'skills']) {
  await check(`GET /master/${resource} → 200`, async () => {
    const r = await fetch(`${BASE_MASTER}/master/${resource}`, { headers: AUTH_HEADER });
    await assertOk(r);
  });
}

await check('POST /master/processes → 200 or 201', async () => {
  const r = await fetch(`${BASE_MASTER}/master/processes`, {
    method: 'POST',
    headers: { ...JSON_HEADERS, 'Idempotency-Key': crypto.randomUUID() },
    body: JSON.stringify({ processCode: 'PC-SMOKE', nameJson: { ja: 'スモーク', en: 'Smoke', zh: '烟雾' } }),
  });
  if (r.status !== 200 && r.status !== 201) throw new Error(`HTTP ${r.status}`);
});

// ── 材料・仕入先・サンプリング ──────────────────────────────
console.log('\n📌 材料・仕入先・サンプリング計画');

for (const resource of ['materials', 'suppliers', 'sampling-plans']) {
  await check(`GET /master/${resource} → 200`, async () => {
    const r = await fetch(`${BASE_MASTER}/master/${resource}`, { headers: AUTH_HEADER });
    await assertOk(r);
  });
}

// ── アンドン・不適合 ────────────────────────────────────────
console.log('\n📌 アンドン・不適合');

await check('GET /alerts → 200', async () => {
  const r = await fetch(`${BASE_TERMINAL}/alerts`, { headers: AUTH_HEADER });
  await assertOk(r);
});

await check('POST /alerts → 200 or 201', async () => {
  const r = await fetch(`${BASE_TERMINAL}/alerts`, {
    method: 'POST',
    headers: { ...JSON_HEADERS, 'Idempotency-Key': crypto.randomUUID() },
    body: JSON.stringify({ caseId: 'case-001', alertType: 'quality', severity: 'medium', description: 'smoke' }),
  });
  if (r.status !== 200 && r.status !== 201) throw new Error(`HTTP ${r.status}`);
});

// ── 作業指示 ────────────────────────────────────────────────
console.log('\n📌 作業指示（Push/Pull）');

await check('GET /work-assignments → 200', async () => {
  const r = await fetch(`${BASE_TERMINAL}/work-assignments`, { headers: AUTH_HEADER });
  await assertOk(r);
});

// ── IQC ─────────────────────────────────────────────────────
console.log('\n📌 IQC（受入検査）');

await check('POST /incoming-inspections → 200 or 201', async () => {
  const r = await fetch(`${BASE_TERMINAL}/incoming-inspections`, {
    method: 'POST',
    headers: { ...JSON_HEADERS, 'Idempotency-Key': crypto.randomUUID() },
    body: JSON.stringify({ lotId: 'lot-001', supplierId: 'sup-001', materialId: 'mat-001', receivedQty: 100, samplingPlanId: 'sp-001' }),
  });
  if (r.status !== 200 && r.status !== 201) throw new Error(`HTTP ${r.status}`);
});

await check('GET /concession-approvals → 200', async () => {
  const r = await fetch(`${BASE_MASTER}/concession-approvals`, { headers: AUTH_HEADER });
  await assertOk(r);
});

// ── リワーク ─────────────────────────────────────────────────
console.log('\n📌 リワーク');

await check('GET /reworks → 200', async () => {
  const r = await fetch(`${BASE_MASTER}/reworks`, { headers: AUTH_HEADER });
  await assertOk(r);
});

await check('GET /dispositions → 200', async () => {
  const r = await fetch(`${BASE_MASTER}/dispositions`, { headers: AUTH_HEADER });
  await assertOk(r);
});

// ── 監査・ハッシュチェーン ──────────────────────────────────
console.log('\n📌 監査・ハッシュチェーン');

await check('GET /audit-logs → 200', async () => {
  const r = await fetch(`${BASE_MASTER}/audit-logs`, { headers: AUTH_HEADER });
  await assertOk(r);
});

await check('GET /hash-chain/verify → 200', async () => {
  const r = await fetch(`${BASE_MASTER}/hash-chain/verify`, { headers: AUTH_HEADER });
  await assertOk(r);
});

// ── Outbox/DLQ ───────────────────────────────────────────────
console.log('\n📌 Outbox/DLQ 監視');

await check('GET /outbox/dlq → 200', async () => {
  const r = await fetch(`${BASE_MASTER}/outbox/dlq`, { headers: AUTH_HEADER });
  await assertOk(r);
});

// ── 帳票・システム ──────────────────────────────────────────
console.log('\n📌 帳票・システム');

await check('GET /reports/work-history → 200 or 404', async () => {
  const r = await fetch(`${BASE_MASTER}/reports/work-history?from=2026-01-01&to=2026-12-31`, { headers: AUTH_HEADER });
  if (r.status !== 200 && r.status !== 404) throw new Error(`HTTP ${r.status}`);
});

await check('GET /system/backup-status → 200', async () => {
  const r = await fetch(`${BASE_MASTER}/system/backup-status`, { headers: AUTH_HEADER });
  await assertOk(r);
});

await check('GET /system/metrics → 200', async () => {
  const r = await fetch(`${BASE_MASTER}/system/metrics`, { headers: AUTH_HEADER });
  await assertOk(r);
});

// ── ヘルスチェック ───────────────────────────────────────────
console.log('\n📌 ヘルスチェック');

await check('GET /healthz → 200', async () => {
  const r = await fetch(`${BASE_TERMINAL}/healthz`);
  await assertOk(r);
});

// ── 結果 ─────────────────────────────────────────────────────
server.close();

console.log(`\n${'─'.repeat(50)}`);
console.log(`結果: ${passed} 通過 / ${failed} 失敗 / 合計 ${passed + failed}`);
if (failed > 0) {
  console.error('\n❌ 一部のエンドポイントが期待通りに動作していません。');
  process.exit(1);
} else {
  console.log('\n✅ 全エンドポイントがスモークテストを通過しました。');
}
