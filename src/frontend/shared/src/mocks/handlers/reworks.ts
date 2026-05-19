import { v7 as uuidv7 } from 'uuid';
import {
  HttpResponse,
  envelope,
  paginate,
  paginatedEnvelope,
  parsePagination,
  problem,
  requireAuth,
  route,
} from '../_helpers';
import { db } from '../db/seed';
import type { Disposition, Rework, ReworkVerification } from '../../types';

interface CreateReworkBody {
  parent_case_id?: string;
  nonconformity_id?: string;
  sop_id?: string;
  rework_sop_version_id?: string;
  assigned_to?: string;
  deadline?: string;
}

interface CreateDispositionBody {
  nonconformity_id?: string;
  decision?: Disposition['decision'];
  decision_reason?: string;
  quality_admin_sign_id?: string;
  supervisor_sign_id?: string;
}

interface CreateReworkVerificationBody {
  verifier_id?: string;
  passed?: boolean;
  note?: string;
  evidence_ids?: string[];
}

export const reworkHandlers = [
  ...route('get', 'any', '/reworks', ({ request }) => {
    const authErr = requireAuth(request);
    if (authErr) return authErr;
    const { page, perPage } = parsePagination(request);
    const u = new URL(request.url);
    const relatedTo = u.searchParams.get('related_to');
    const status = u.searchParams.get('status');
    const assignedTo = u.searchParams.get('assigned_to');
    let filtered = db.reworks;
    if (relatedTo) filtered = filtered.filter((r) => r.parentCaseId === relatedTo || r.reworkCaseId === relatedTo);
    if (status) filtered = filtered.filter((r) => r.status === status);
    if (assignedTo) filtered = filtered.filter((r) => r.assignedTo === assignedTo);
    const { slice, total } = paginate(filtered, page, perPage);
    return HttpResponse.json(paginatedEnvelope(slice, total, page, perPage));
  }),

  ...route('post', 'terminal', '/reworks', async ({ request }) => {
    const authErr = requireAuth(request);
    if (authErr) return authErr;
    const body = (await request.json().catch(() => null)) as CreateReworkBody | null;
    if (!body?.parent_case_id || !body.nonconformity_id || !body.sop_id) {
      return problem(422, 'ERR-VAL-001', 'Required field missing', '必須フィールドが不足しています');
    }
    if (body.parent_case_id === body.nonconformity_id) {
      return problem(409, 'ERR-BIZ-020', 'rework_case_same_as_parent', 'parent_case_id と nonconformity_id が同一です');
    }
    const existingCount = db.reworks.filter((r) => r.parentCaseId === body.parent_case_id).length;
    if (existingCount >= 3) {
      return problem(409, 'ERR-BIZ-022', 'rework_max_count_exceeded', 'リワーク上限を超えました');
    }
    const rework: Rework = {
      id: uuidv7(),
      parentCaseId: body.parent_case_id,
      reworkCaseId: uuidv7(),
      nonconformityId: body.nonconformity_id,
      sopId: body.sop_id,
      reworkSopVersionId: body.rework_sop_version_id ?? '',
      assignedTo: body.assigned_to ?? null,
      status: 'OPEN',
      reworkCount: existingCount + 1,
      deadline: body.deadline ?? null,
      createdAt: new Date().toISOString(),
    };
    db.reworks.push(rework);
    return HttpResponse.json(envelope(rework), { status: 201 });
  }),

  ...route('get', 'any', '/reworks/:id', ({ request, params }) => {
    const authErr = requireAuth(request);
    if (authErr) return authErr;
    const item = db.reworks.find((r) => r.id === params['id']);
    if (!item) return problem(404, 'ERR-DB-002', 'NotFound', 'rework が存在しません');
    return HttpResponse.json(envelope(item));
  }),

  ...route('patch', 'terminal', '/reworks/:id', async ({ request, params }) => {
    const authErr = requireAuth(request);
    if (authErr) return authErr;
    const item = db.reworks.find((r) => r.id === params['id']);
    if (!item) return problem(404, 'ERR-DB-002', 'NotFound', 'rework が存在しません');
    const body = (await request.json().catch(() => null)) as Partial<Rework> | null;
    if (!body) return problem(422, 'ERR-VAL-001', 'Required field missing', 'リクエストボディが必要です');
    Object.assign(item, body);
    return HttpResponse.json(envelope(item));
  }),

  ...route('post', 'terminal', '/reworks/:id/verifications', async ({ request, params }) => {
    const authErr = requireAuth(request);
    if (authErr) return authErr;
    const rework = db.reworks.find((r) => r.id === params['id']);
    if (!rework) return problem(404, 'ERR-DB-002', 'NotFound', 'rework が存在しません');
    const body = (await request.json().catch(() => null)) as CreateReworkVerificationBody | null;
    if (!body?.verifier_id || body.passed === undefined) {
      return problem(422, 'ERR-VAL-001', 'Required field missing', 'verifier_id と passed は必須です');
    }
    if (rework.assignedTo === body.verifier_id) {
      return problem(422, 'ERR-BIZ-023', 'rework_verifier_same_as_worker', '再検査者は作業者と異なる必要があります');
    }
    const verification: ReworkVerification = {
      id: uuidv7(),
      reworkId: rework.id,
      verifierId: body.verifier_id,
      verifiedAt: new Date().toISOString(),
      passed: body.passed,
      note: body.note ?? '',
      evidenceIds: body.evidence_ids ?? [],
    };
    db.reworkVerifications.push(verification);
    rework.status = body.passed ? 'CLOSED' : 'IN_PROGRESS';
    return HttpResponse.json(envelope(verification), { status: 201 });
  }),

  ...route('get', 'master', '/dispositions', ({ request }) => {
    const authErr = requireAuth(request);
    if (authErr) return authErr;
    const { page, perPage } = parsePagination(request);
    const { slice, total } = paginate(db.dispositions, page, perPage);
    return HttpResponse.json(paginatedEnvelope(slice, total, page, perPage));
  }),

  ...route('post', 'master', '/dispositions', async ({ request }) => {
    const authErr = requireAuth(request);
    if (authErr) return authErr;
    const body = (await request.json().catch(() => null)) as CreateDispositionBody | null;
    if (!body?.nonconformity_id || !body.decision || !body.decision_reason || !body.quality_admin_sign_id || !body.supervisor_sign_id) {
      return problem(422, 'ERR-VAL-001', 'Required field missing', '必須フィールドが不足しています');
    }
    // Two-Person Integrity: 同一署名者を DB トリガで物理禁止する制約をモックでも検証する
    const qaSign = db.electronicSigns.find((s) => s.id === body.quality_admin_sign_id);
    const supSign = db.electronicSigns.find((s) => s.id === body.supervisor_sign_id);
    if (qaSign && supSign && qaSign.signerId === supSign.signerId) {
      return problem(422, 'ERR-BIZ-021', 'disposition_same_signer', 'Two-Person Integrity 違反: 同一署名者が検出されました');
    }
    if (db.dispositions.some((d) => d.nonconformityId === body.nonconformity_id)) {
      return problem(409, 'ERR-BIZ-019', 'disposition_already_decided', 'すでにディスポジションが決定済みです');
    }
    const disposition: Disposition = {
      id: uuidv7(),
      nonconformityId: body.nonconformity_id,
      dispositionType: body.decision === 'RETURN' ? 'RETURN_TO_VENDOR' : body.decision === 'USE_AS_IS' ? 'USE_AS_IS' : body.decision,
      decision: body.decision,
      decisionReason: body.decision_reason,
      qualityAdminSignId: body.quality_admin_sign_id,
      supervisorSignId: body.supervisor_sign_id,
      signedAt: new Date().toISOString(),
      createdAt: new Date().toISOString(),
    };
    db.dispositions.push(disposition);
    return HttpResponse.json(envelope(disposition), { status: 201 });
  }),

  ...route('post', 'master', '/dispositions/:id/sign', async ({ request, params }) => {
    const authErr = requireAuth(request);
    if (authErr) return authErr;
    const item = db.dispositions.find((d) => d.id === params['id']);
    if (!item) return problem(404, 'ERR-DB-002', 'NotFound', 'disposition が存在しません');
    item.signedAt = new Date().toISOString();
    return HttpResponse.json(envelope(item));
  }),
];
