import { v7 as uuidv7 } from 'uuid';
import {
  HttpResponse,
  cursorEnvelope,
  envelope,
  problem,
  requireAuth,
  route,
} from '../_helpers';
import { db } from '../db/seed';
import { resolveLocale } from '../../i18n/resolveLocale';
import type { WorkAssignment } from '../../types';

interface CreateAssignmentBody {
  external_order_id?: string;
  external_system?: string;
  work_pattern_key?: string;
  target_terminal_key?: string;
  lot_id_ext?: string;
  suggested_worker_key?: string;
  suggested_equipment_key?: string;
  due_at?: string;
  priority?: number;
}

export const workAssignmentHandlers = [
  ...route('get', 'terminal', '/work-assignments', ({ request }) => {
    const authErr = requireAuth(request);
    if (authErr) return authErr;
    const u = new URL(request.url);
    const statusFilter = (u.searchParams.get('status') ?? 'pending,dispatched').split(',');
    const limit = Math.min(200, Math.max(1, Number(u.searchParams.get('limit') ?? '50')));
    const after = u.searchParams.get('after');

    let assignments = db.workAssignments.filter((a) => statusFilter.includes(a.status));
    if (after) {
      const anchor = db.workAssignments.find((a) => a.id === after);
      if (anchor) {
        assignments = assignments.filter((a) => a.receivedAt > anchor.receivedAt);
      }
    }
    assignments.sort((a, b) => a.receivedAt.localeCompare(b.receivedAt));
    const limited = assignments.slice(0, limit + 1);
    const hasMore = limited.length > limit;
    const page = hasMore ? limited.slice(0, limit) : limited;
    const nextCursor = hasMore ? page[page.length - 1]!.id : null;
    return HttpResponse.json(cursorEnvelope(page, limit, hasMore, nextCursor));
  }),

  ...route('post', 'master', '/work-assignments', async ({ request }) => {
    const body = (await request.json().catch(() => null)) as CreateAssignmentBody | null;
    if (!body?.external_order_id || !body.external_system || !body.work_pattern_key || !body.target_terminal_key) {
      return problem(422, 'ERR-VAL-001', 'Required field missing', '必須フィールドが不足しています');
    }
    const sop = db.sops.find((s) => s.sopCode === body.work_pattern_key);
    const terminal = db.terminals.find((t) => t.externalKey === body.target_terminal_key);
    if (!sop || !terminal) {
      return problem(422, 'ERR-BIZ-027', 'external_key_resolution_failed', '外部キーの解決に失敗しました', {
        violations: [
          !sop ? { field: 'work_pattern_key', message: 'sop が見つかりません' } : null,
          !terminal ? { field: 'target_terminal_key', message: 'terminal が見つかりません' } : null,
        ].filter((v): v is { field: string; message: string } => v !== null),
      });
    }
    const assignment: WorkAssignment = {
      id: uuidv7(),
      externalOrderId: body.external_order_id,
      externalSystem: body.external_system,
      sopId: sop.id,
      sopName: resolveLocale(sop.nameJson, 'ja'),
      targetTerminalId: terminal.id,
      lotId: null,
      lotNumber: null,
      suggestedWorkerId: null,
      suggestedEquipmentId: null,
      dueAt: body.due_at ?? null,
      priority: body.priority ?? 3,
      status: 'pending',
      receivedAt: new Date().toISOString(),
      acknowledgedAt: null,
      cancelledAt: null,
    };
    db.workAssignments.push(assignment);
    return HttpResponse.json(envelope({
      assignment_id: assignment.id,
      status: assignment.status,
      target_terminal_id: assignment.targetTerminalId,
      received_at: assignment.receivedAt,
    }), { status: 202 });
  }),

  ...route('get', 'terminal', '/work-assignments/stream', ({ request }) => {
    const authErr = requireAuth(request);
    if (authErr) return authErr;
    // SSE は ReadableStream で event-stream を模擬する（keepalive 1 件のみ流して終了）
    const encoder = new TextEncoder();
    const body = new ReadableStream({
      start(controller) {
        const event = `id: ${uuidv7()}\nevent: keepalive\ndata: ${JSON.stringify({ timestamp: new Date().toISOString() })}\n\n`;
        controller.enqueue(encoder.encode(event));
        controller.close();
      },
    });
    return new HttpResponse(body, {
      status: 200,
      headers: {
        'Content-Type': 'text/event-stream; charset=utf-8',
        'Cache-Control': 'no-cache',
        Connection: 'keep-alive',
        'X-Accel-Buffering': 'no',
      },
    });
  }),

  ...route('post', 'terminal', '/work-assignments/:id/ack', ({ request, params }) => {
    const authErr = requireAuth(request);
    if (authErr) return authErr;
    const assignment = db.workAssignments.find((a) => a.id === params['id']);
    if (!assignment) return problem(404, 'ERR-DB-002', 'NotFound', 'assignment が存在しません');
    assignment.status = 'acknowledged';
    assignment.acknowledgedAt = new Date().toISOString();
    return HttpResponse.json(envelope({
      assignment_id: assignment.id,
      status: assignment.status,
      acknowledged_at: assignment.acknowledgedAt,
    }));
  }),
];
