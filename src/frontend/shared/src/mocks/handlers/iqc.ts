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
import { judgeAql, resolveSamplingPlan } from '../../domain/aql';
import type { ConcessionApproval, IncomingInspection, IncomingInspectionMeasurement } from '../../types';

interface CreateInspectionBody {
  lot_id?: string;
  supplier_id?: string;
  material_id?: string;
  lot_quantity?: number;
}

interface MeasurementBody {
  sample_no?: number;
  measured_value?: number | null;
  defect_flag?: boolean;
  evidence_file_id?: string | null;
}

interface ConcessionBody {
  incoming_inspection_id?: string;
  reason?: string;
  validity_scope?: unknown;
  valid_until?: string;
  requested_by?: string;
  electronic_sign_id?: string;
}

export const iqcHandlers = [
  // SCR-MC-011 受入検査ダッシュボード集計値
  ...route('get', 'any', '/iqc/dashboard', ({ request }) => {
    const authErr = requireAuth(request);
    if (authErr) return authErr;
    const inspections = db.incomingInspections;
    const total = inspections.length;
    const passed = inspections.filter((i) => i.qcStatus === 'PASSED' || i.qcStatus === 'CONDITIONAL_PASS').length;
    const failed = inspections.filter((i) => i.qcStatus === 'FAILED' || i.qcStatus === 'REJECTED').length;
    const passRate = total > 0 ? (passed / total) * 100 : 100;
    const failRate = total > 0 ? (failed / total) * 100 : 0;
    // 仕入先別の合格・不合格数を集計する
    const supplierMap = new Map<string, { passed: number; failed: number }>();
    for (const ins of inspections) {
      const supplier = db.suppliers.find((s) => s.id === ins.supplierId);
      const name = supplier ? (supplier.nameJson.ja || ins.supplierId) : ins.supplierId;
      const entry = supplierMap.get(name) ?? { passed: 0, failed: 0 };
      if (ins.qcStatus === 'PASSED' || ins.qcStatus === 'CONDITIONAL_PASS') entry.passed++;
      else if (ins.qcStatus === 'FAILED' || ins.qcStatus === 'REJECTED') entry.failed++;
      supplierMap.set(name, entry);
    }
    const bySupplier = [...supplierMap.entries()].map(([supplierName, v]) => ({ supplierName, ...v }));
    // 直近10点のダミー不合格率推移
    const now = Date.now();
    const failRateTrend = Array.from({ length: 10 }, (_, i) => ({
      ts: new Date(now - (9 - i) * 86400_000).toISOString().slice(0, 10),
      rate: Math.max(0, failRate + (Math.random() - 0.5) * 2),
    }));
    return HttpResponse.json(envelope({ passRate, failRate, totalLots: total, bySupplier, failRateTrend }));
  }),

  ...route('get', 'any', '/iqc/incoming-inspections', ({ request }) => {
    const authErr = requireAuth(request);
    if (authErr) return authErr;
    const { page, perPage } = parsePagination(request);
    const { slice, total } = paginate(db.incomingInspections, page, perPage);
    return HttpResponse.json(paginatedEnvelope(slice, total, page, perPage));
  }),

  ...route('post', 'terminal', '/iqc/incoming-inspections', async ({ request }) => {
    const authErr = requireAuth(request);
    if (authErr) return authErr;
    const body = (await request.json().catch(() => null)) as CreateInspectionBody | null;
    if (!body?.lot_id || !body.supplier_id || !body.material_id || !body.lot_quantity) {
      return problem(422, 'ERR-VAL-001', 'Required field missing', '必須フィールドが不足しています');
    }
    if (body.lot_quantity < 2) {
      return problem(422, 'ERR-VAL-002', 'Value out of range', 'lot_quantity は 2 以上である必要があります');
    }
    const samplingPlan = db.samplingPlans[0];
    if (!samplingPlan) {
      return problem(422, 'ERR-VAL-028', 'sampling_plan_not_found', 'サンプリング計画が登録されていません');
    }
    const resolved = resolveSamplingPlan(body.lot_quantity, samplingPlan.aqlValue, samplingPlan.inspectionLevel);
    const inspection: IncomingInspection = {
      id: uuidv7(),
      lotId: body.lot_id,
      supplierId: body.supplier_id,
      materialId: body.material_id,
      receivedQty: body.lot_quantity,
      samplingPlanId: samplingPlan.id,
      sampleSizeN: resolved.sampleSizeN,
      acceptNumberAc: resolved.acceptNumberAc,
      rejectNumberRe: resolved.rejectNumberRe,
      severityState: 'NORMAL',
      qcStatus: 'PENDING',
      defectCount: 0,
      inspectedAt: null,
      judgedAt: null,
      judgedBy: null,
      createdAt: new Date().toISOString(),
    };
    db.incomingInspections.push(inspection);
    return HttpResponse.json(envelope({
      inspection_id: inspection.id,
      sampling_plan_id: inspection.samplingPlanId,
      sample_size_n: inspection.sampleSizeN,
      accept_number_ac: inspection.acceptNumberAc,
      reject_number_re: inspection.rejectNumberRe,
      severity_state: inspection.severityState,
      qc_status: inspection.qcStatus,
    }), { status: 201 });
  }),

  ...route('get', 'any', '/iqc/incoming-inspections/:id', ({ request, params }) => {
    const authErr = requireAuth(request);
    if (authErr) return authErr;
    const inspection = db.incomingInspections.find((i) => i.id === params['id']);
    if (!inspection) return problem(404, 'ERR-VAL-027', 'lot_not_found', '受入検査が存在しません');
    return HttpResponse.json(envelope(inspection));
  }),

  ...route('post', 'terminal', '/iqc/incoming-inspections/:id/measurements', async ({ request, params }) => {
    const authErr = requireAuth(request);
    if (authErr) return authErr;
    const inspection = db.incomingInspections.find((i) => i.id === params['id']);
    if (!inspection) return problem(404, 'ERR-VAL-027', 'lot_not_found', '受入検査が存在しません');
    const body = (await request.json().catch(() => null)) as MeasurementBody | null;
    if (!body || body.sample_no === undefined || body.defect_flag === undefined) {
      return problem(422, 'ERR-VAL-001', 'Required field missing', 'sample_no と defect_flag は必須です');
    }
    const measurement: IncomingInspectionMeasurement = {
      id: uuidv7(),
      inspectionId: inspection.id,
      sampleNo: body.sample_no,
      measuredValue: body.measured_value ?? null,
      defectFlag: body.defect_flag,
      evidenceFileId: body.evidence_file_id ?? null,
      recordedAt: new Date().toISOString(),
      recordedBy: '00000000-0000-7000-0000-000000000000',
    };
    db.incomingInspectionMeasurements.push(measurement);
    if (body.defect_flag) inspection.defectCount += 1;
    return HttpResponse.json(envelope({
      id: measurement.id,
      inspection_id: measurement.inspectionId,
      sample_no: measurement.sampleNo,
      measured_value: measurement.measuredValue,
      defect_flag: measurement.defectFlag,
    }), { status: 201 });
  }),

  ...route('post', 'master', '/iqc/incoming-inspections/:id/judge', ({ request, params }) => {
    const authErr = requireAuth(request);
    if (authErr) return authErr;
    const inspection = db.incomingInspections.find((i) => i.id === params['id']);
    if (!inspection) return problem(404, 'ERR-VAL-027', 'lot_not_found', '受入検査が存在しません');
    if (inspection.qcStatus !== 'PENDING' && inspection.qcStatus !== 'INSPECTING') {
      return problem(409, 'ERR-BIZ-017', 'iqc_already_judged', '判定済みの検査です');
    }
    const measurements = db.incomingInspectionMeasurements.filter((m) => m.inspectionId === inspection.id);
    if (measurements.length < inspection.sampleSizeN) {
      return problem(422, 'ERR-VAL-030', 'measurement_count_below_n', `必要サンプル数 ${inspection.sampleSizeN} に対して ${measurements.length} 件しかありません`);
    }
    const verdict = judgeAql(inspection.defectCount, inspection.acceptNumberAc, inspection.rejectNumberRe);
    inspection.qcStatus = verdict === 'PASSED' ? 'PASSED' : verdict === 'REJECTED' ? 'REJECTED' : 'INSPECTING';
    inspection.judgedAt = new Date().toISOString();
    return HttpResponse.json(envelope(inspection));
  }),

  ...route('get', 'master', '/concession-approvals', ({ request }) => {
    const authErr = requireAuth(request);
    if (authErr) return authErr;
    const { page, perPage } = parsePagination(request);
    const { slice, total } = paginate(db.concessionApprovals, page, perPage);
    return HttpResponse.json(paginatedEnvelope(slice, total, page, perPage));
  }),

  ...route('post', 'master', '/concession-approvals', async ({ request }) => {
    const authErr = requireAuth(request);
    if (authErr) return authErr;
    const body = (await request.json().catch(() => null)) as ConcessionBody | null;
    if (!body?.incoming_inspection_id || !body.reason || !body.requested_by) {
      return problem(422, 'ERR-VAL-001', 'Required field missing', '必須フィールドが不足しています');
    }
    const concession: ConcessionApproval = {
      id: uuidv7(),
      incomingInspectionId: body.incoming_inspection_id,
      requestedBy: body.requested_by,
      approvedBy: null,
      approvalSign: null,
      electronicSignId: body.electronic_sign_id ?? null,
      conditionNote: body.reason,
      validityScope: JSON.stringify(body.validity_scope ?? {}),
      validUntil: body.valid_until ?? null,
      status: 'PENDING',
      createdAt: new Date().toISOString(),
    };
    db.concessionApprovals.push(concession);
    return HttpResponse.json(envelope(concession), { status: 201 });
  }),

  ...route('post', 'master', '/concession-approvals/:id/sign', async ({ request, params }) => {
    const authErr = requireAuth(request);
    if (authErr) return authErr;
    const concession = db.concessionApprovals.find((c) => c.id === params['id']);
    if (!concession) return problem(404, 'ERR-DB-002', 'NotFound', 'concession が存在しません');
    const body = (await request.json().catch(() => null)) as { electronic_sign_id?: string } | null;
    if (!body?.electronic_sign_id) {
      return problem(422, 'ERR-VAL-001', 'Required field missing', 'electronic_sign_id は必須です');
    }
    concession.electronicSignId = body.electronic_sign_id;
    concession.status = 'APPROVED';
    concession.approvedBy = '00000000-0000-7000-0000-000000000000';
    return HttpResponse.json(envelope(concession));
  }),
];
