import { v7 as uuidv7 } from 'uuid';
import {
  HttpResponse,
  envelope,
  paginatedEnvelope,
  parsePagination,
  paginate,
  problem,
  requireAuth,
  route,
} from '../_helpers';
import { db } from '../db/seed';
import { verifyChain } from '../../domain/hash-chain';

export const auditHandlers = [
  ...route('get', 'master', '/audit-logs', ({ request }) => {
    const authErr = requireAuth(request);
    if (authErr) return authErr;
    const { page, perPage } = parsePagination(request);
    // 監査ログはイベント・サイン・ディスポジションを統合した簡易ビューを返す
    const logs = [
      ...db.workEvents.map((e) => ({ id: e.eventId, type: 'work_event', activity: e.activity, timestamp: e.timestampClient })),
      ...db.electronicSigns.map((s) => ({ id: s.id, type: 'electronic_sign', context: s.contextType, timestamp: s.signedAt })),
      ...db.andonAlerts.map((a) => ({ id: a.id, type: 'andon_alert', severity: a.severity, timestamp: a.raisedAt })),
    ].sort((a, b) => a.timestamp.localeCompare(b.timestamp));
    const { slice, total } = paginate(logs, page, perPage);
    return HttpResponse.json(paginatedEnvelope(slice, total, page, perPage));
  }),

  ...route('get', 'master', '/hash-chain/verify', ({ request }) => {
    const authErr = requireAuth(request);
    if (authErr) return authErr;
    const events = db.hashChainBlocks.map((b) => ({
      contentHash: b.contentHash,
      prevHash: b.prevHash,
      payload: JSON.parse(b.payload),
    }));
    const verified = verifyChain(events);
    return HttpResponse.json(envelope({
      verified,
      total_blocks: db.hashChainBlocks.length,
      first_broken_block_id: verified ? null : (db.hashChainBlocks[0]?.id ?? null),
      checked_at: new Date().toISOString(),
    }));
  }),

  ...route('get', 'master', '/outbox/dlq', ({ request }) => {
    const authErr = requireAuth(request);
    if (authErr) return authErr;
    const dlqItems = db.outboxEvents.filter((e) => e.status === 'dlq');
    return HttpResponse.json(envelope({
      dlq_count: dlqItems.length,
      items: dlqItems.map((e) => ({
        dlq_item_id: e.id,
        outbox_event_id: e.id,
        event_type: e.eventType,
        retry_count: e.retryCount,
        last_error: e.lastError,
        first_failed_at: e.firstFailedAt,
        last_failed_at: e.lastFailedAt,
      })),
    }));
  }),

  // DLQ イベントを完全削除（廃棄）する（論理削除ではなく物理削除が仕様）
  ...route('delete', 'master', '/outbox/dlq/:id', ({ request, params }) => {
    const authErr = requireAuth(request);
    if (authErr) return authErr;
    const idx = db.outboxEvents.findIndex((e) => e.id === params['id'] && e.status === 'dlq');
    if (idx === -1) return problem(404, 'ERR-DB-002', 'NotFound', 'DLQ イベントが存在しません');
    db.outboxEvents.splice(idx, 1);
    return new HttpResponse(null, { status: 204 });
  }),

  ...route('post', 'master', '/outbox/dlq/:id/retry', async ({ request, params }) => {
    const authErr = requireAuth(request);
    if (authErr) return authErr;
    const item = db.outboxEvents.find((e) => e.id === params['id']);
    if (!item) return problem(404, 'ERR-DB-002', 'NotFound', 'outbox イベントが存在しません');
    const body = (await request.json().catch(() => null)) as { requeued_by?: string; reason?: string } | null;
    if (!body?.requeued_by || !body.reason) {
      return problem(422, 'ERR-VAL-001', 'Required field missing', 'requeued_by と reason は必須です');
    }
    item.status = 'pending';
    item.retryCount = 0;
    item.lastError = null;
    return HttpResponse.json(envelope({
      outbox_event_id: item.id,
      status: item.status,
      retry_count_reset: true,
      requeued_at: new Date().toISOString(),
    }));
  }),

  ...route('get', 'master', '/reports/:type', ({ request, params }) => {
    const authErr = requireAuth(request);
    if (authErr) return authErr;
    const reportType = params['type'];
    return HttpResponse.json(envelope({
      report_id: uuidv7(),
      type: reportType,
      generated_at: new Date().toISOString(),
      summary: {
        total_executions: db.workExecutions.length,
        completed: db.workExecutions.filter((e) => e.status === 'completed').length,
      },
      document_hash: 'sha256:mock',
    }));
  }),

  ...route('get', 'master', '/system/backup-status', () => HttpResponse.json(envelope({
    last_backup_at: new Date().toISOString(),
    status: 'ok',
    next_scheduled_at: new Date(Date.now() + 24 * 60 * 60 * 1000).toISOString(),
  }))),

  ...route('get', 'master', '/system/metrics', () => {
    const dlqCount = db.outboxEvents.filter((e) => e.status === 'dlq').length;
    const andonActiveCount = db.andonAlerts.filter((a) => a.status === 'open').length;
    const now = Date.now();
    // 直近10分の時系列ダミーデータを生成する
    const series = Array.from({ length: 10 }, (_, i) => ({
      ts: new Date(now - (9 - i) * 60_000).toISOString().slice(11, 16),
      availability: 99.9 + Math.random() * 0.1,
      errorRate: Math.random() * 0.1,
    }));
    return HttpResponse.json(envelope({
      availability: 99.92,
      latencyP95Ms: 230,
      errorRate: 0.05,
      errorBudgetRemaining: 78,
      dlqCount,
      andonActiveCount,
      backupStatus: dlqCount > 0 ? 'yellow' : 'green',
      series,
    }));
  }),

  ...route('get', 'any', '/healthz', () => HttpResponse.json({ status: 'ok', timestamp: new Date().toISOString() })),

  ...route('get', 'any', '/readyz', () => HttpResponse.json({
    status: 'ready',
    checks: { database: 'ok', outbox_consumer: 'ok', ldap: 'ok' },
    timestamp: new Date().toISOString(),
  })),
];
