import { v7 as uuidv7 } from 'uuid';
import {
  HttpResponse,
  envelope,
  problem,
  requireAuth,
  route,
  storeIdempotency,
  withIdempotency,
} from '../_helpers';
import { db } from '../db/seed';
import { computeContentHash, GENESIS_HASH } from '../../domain/hash-chain';
import type {
  CompleteWorkExecutionRequest,
  CreateWorkExecutionRequest,
  ResumeWorkExecutionRequest,
  SuspendWorkExecutionRequest,
  WorkExecution,
} from '../../types';

type CompleteRequest = { completed_by?: string; timestamp_client?: string; final_remarks?: string } & CompleteWorkExecutionRequest;
type SuspendRequest = { reason_code?: string; reason_detail?: string; timestamp_client?: string } & SuspendWorkExecutionRequest;
type ResumeRequest = { resumed_by?: string; timestamp_client?: string } & ResumeWorkExecutionRequest;
type CreateRequest = { work_order_id?: string; operator_id?: string; device_id?: string; start_timestamp_client?: string } & CreateWorkExecutionRequest;

export const workExecutionHandlers = [
  ...route('post', 'terminal', '/work-executions', async ({ request }) => {
    const authErr = requireAuth(request);
    if (authErr) return authErr;
    const body = (await request.json().catch(() => null)) as CreateRequest | null;
    if (!body?.work_order_id || !body.operator_id || !body.device_id) {
      return problem(422, 'ERR-VAL-001', 'Required field missing', '必須フィールドが不足しています');
    }
    const workOrder = db.workOrders.find((wo) => wo.id === body.work_order_id);
    if (!workOrder) {
      return problem(409, 'ERR-DB-002', 'Foreign key violation', 'work_order が存在しません');
    }
    if (db.workExecutions.some((we) => we.workOrderId === body.work_order_id && we.status === 'in_progress')) {
      return problem(409, 'ERR-BIZ-007', 'version_already_published', '同一作業指示で実行中の作業が存在します');
    }
    const idem = await withIdempotency<WorkExecution>(request, body);
    if (idem.conflict) return idem.conflict;
    if (idem.cached) return HttpResponse.json(envelope(idem.cached.response), { status: idem.cached.status });

    const steps = db.steps.filter((s) => s.sopVersionId === workOrder.sopVersionId);
    const exec: WorkExecution = {
      id: uuidv7(),
      workOrderId: workOrder.id,
      operatorId: body.operator_id,
      deviceId: body.device_id,
      status: 'in_progress',
      currentStepId: steps[0]?.id ?? null,
      completedStepCount: 0,
      totalStepCount: steps.length,
      sopVersionSnapshot: {
        sopId: workOrder.sopId,
        version: '1.0.0',
        snapshotHash: 'sha256:mock',
      },
      startedAt: new Date().toISOString(),
      lastEventAt: new Date().toISOString(),
      completedAt: null,
      createdAt: new Date().toISOString(),
    };
    db.workExecutions.push(exec);
    storeIdempotency(idem.key, idem.bodyHash, exec, 201);
    return HttpResponse.json(envelope(exec), { status: 201 });
  }),

  ...route('put', 'terminal', '/work-executions/:id/heartbeat', ({ request, params }) => {
    const authErr = requireAuth(request);
    if (authErr) return authErr;
    const exec = db.workExecutions.find((we) => we.id === params['id']);
    if (!exec) return problem(404, 'ERR-DB-002', 'NotFound', 'work_execution が存在しません');
    exec.lastEventAt = new Date().toISOString();
    return HttpResponse.json(envelope({ id: exec.id, status: exec.status, last_event_at: exec.lastEventAt }));
  }),

  ...route('post', 'terminal', '/work-executions/:id/suspend', async ({ request, params }) => {
    const authErr = requireAuth(request);
    if (authErr) return authErr;
    const exec = db.workExecutions.find((we) => we.id === params['id']);
    if (!exec) return problem(404, 'ERR-DB-002', 'NotFound', 'work_execution が存在しません');
    if (exec.status !== 'in_progress') {
      return problem(409, 'ERR-BIZ-001', 'lock_step_violation', '実行中以外の作業は中断できません');
    }
    const body = (await request.json().catch(() => null)) as SuspendRequest | null;
    if (!body?.reason_code) {
      return problem(422, 'ERR-VAL-001', 'Required field missing', 'reason_code は必須です');
    }
    const suspendedAt = new Date().toISOString();
    const suspensionId = uuidv7();
    exec.status = 'suspended';
    db.suspensions.push({
      id: suspensionId,
      workExecutionId: exec.id,
      reasonCode: body.reason_code as 'equipment_breakdown',
      reasonDetail: body.reason_detail ?? '',
      suspendedAt,
      resumedAt: null,
      resumedBy: null,
    });
    return HttpResponse.json(envelope({
      id: exec.id,
      status: 'suspended',
      suspension_id: suspensionId,
      suspended_at: suspendedAt,
    }));
  }),

  ...route('post', 'terminal', '/work-executions/:id/resume', async ({ request, params }) => {
    const authErr = requireAuth(request);
    if (authErr) return authErr;
    const exec = db.workExecutions.find((we) => we.id === params['id']);
    if (!exec) return problem(404, 'ERR-DB-002', 'NotFound', 'work_execution が存在しません');
    const body = (await request.json().catch(() => null)) as ResumeRequest | null;
    if (!body?.resumed_by) {
      return problem(422, 'ERR-VAL-001', 'Required field missing', 'resumed_by は必須です');
    }
    exec.status = 'in_progress';
    const suspension = db.suspensions.find((s) => s.workExecutionId === exec.id && s.resumedAt === null);
    if (suspension) {
      suspension.resumedAt = new Date().toISOString();
      suspension.resumedBy = body.resumed_by;
    }
    return HttpResponse.json(envelope({
      id: exec.id,
      status: 'in_progress',
      resumed_at: new Date().toISOString(),
      current_step_id: exec.currentStepId,
    }));
  }),

  ...route('post', 'terminal', '/work-executions/:id/complete', async ({ request, params }) => {
    const authErr = requireAuth(request);
    if (authErr) return authErr;
    const exec = db.workExecutions.find((we) => we.id === params['id']);
    if (!exec) return problem(404, 'ERR-DB-002', 'NotFound', 'work_execution が存在しません');
    const body = (await request.json().catch(() => null)) as CompleteRequest | null;
    if (!body?.completed_by) {
      return problem(422, 'ERR-VAL-001', 'Required field missing', 'completed_by は必須です');
    }
    const idem = await withIdempotency<unknown>(request, body);
    if (idem.conflict) return idem.conflict;

    const completedAt = new Date().toISOString();
    exec.status = 'completed';
    exec.completedAt = completedAt;
    const lastBlock = db.hashChainBlocks[db.hashChainBlocks.length - 1];
    const prevHash = lastBlock?.contentHash ?? GENESIS_HASH;
    const blockPayload = { workExecutionId: exec.id, completedAt };
    const contentHash = computeContentHash(prevHash, blockPayload);
    const block = {
      id: uuidv7(),
      blockNumber: db.hashChainBlocks.length + 1,
      prevHash,
      contentHash,
      payload: JSON.stringify(blockPayload),
      createdAt: completedAt,
    };
    db.hashChainBlocks.push(block);
    const response = {
      id: exec.id,
      status: 'completed' as const,
      completed_at: completedAt,
      hash_chain_block_id: block.id,
      hash_chain_value: `sha256:${block.contentHash}`,
    };
    storeIdempotency(idem.key, idem.bodyHash, response, 200);
    return HttpResponse.json(envelope(response));
  }),
];
