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
import type { AndonAlert, Nonconformity, Capa, KaizenProposal } from '../../types';

export const alertHandlers = [
  ...route('get', 'any', '/alerts', ({ request }) => {
    const authErr = requireAuth(request);
    if (authErr) return authErr;
    const { page, perPage } = parsePagination(request);
    const u = new URL(request.url);
    const status = u.searchParams.get('status');
    let items = db.andonAlerts.slice();
    if (status) items = items.filter((a) => a.status === status);
    const { slice, total } = paginate(items, page, perPage);
    return HttpResponse.json(paginatedEnvelope(slice, total, page, perPage));
  }),

  ...route('post', 'terminal', '/alerts', async ({ request }) => {
    const authErr = requireAuth(request);
    if (authErr) return authErr;
    const body = (await request.json().catch(() => null)) as {
      alert_type?: AndonAlert['alertType'];
      severity?: AndonAlert['severity'];
      raised_by?: string;
      title?: string;
      description?: string;
      timestamp_client?: string;
      work_execution_id?: string;
      step_id?: string;
    } | null;
    if (!body?.alert_type || !body.severity || !body.raised_by || !body.title || !body.description) {
      return problem(422, 'ERR-VAL-001', 'Required field missing', '必須フィールドが不足しています');
    }
    const alert: AndonAlert = {
      id: uuidv7(),
      alertType: body.alert_type,
      severity: body.severity,
      status: 'open',
      workExecutionId: body.work_execution_id ?? null,
      stepId: body.step_id ?? null,
      raisedBy: body.raised_by,
      title: body.title,
      description: body.description,
      raisedAt: new Date().toISOString(),
      acknowledgedBy: null,
      acknowledgedAt: null,
      resolvedBy: null,
      resolvedAt: null,
      resolutionNote: null,
    };
    db.andonAlerts.push(alert);
    return HttpResponse.json(envelope({
      alert_id: alert.id,
      alert_type: alert.alertType,
      severity: alert.severity,
      status: alert.status,
      work_execution_id: alert.workExecutionId,
      raised_by: alert.raisedBy,
      title: alert.title,
      raised_at: alert.raisedAt,
      notification_sent: alert.severity === 'critical',
    }), { status: 201 });
  }),

  ...route('patch', 'master', '/alerts/:id/acknowledge', async ({ request, params }) => {
    const authErr = requireAuth(request);
    if (authErr) return authErr;
    const alert = db.andonAlerts.find((a) => a.id === params['id']);
    if (!alert) return problem(404, 'ERR-DB-002', 'NotFound', 'alert が存在しません');
    const body = (await request.json().catch(() => null)) as { acknowledged_by?: string } | null;
    if (!body?.acknowledged_by) {
      return problem(422, 'ERR-VAL-001', 'Required field missing', 'acknowledged_by は必須です');
    }
    alert.status = 'acknowledged';
    alert.acknowledgedBy = body.acknowledged_by;
    alert.acknowledgedAt = new Date().toISOString();
    return HttpResponse.json(envelope({
      alert_id: alert.id,
      status: alert.status,
      acknowledged_by: alert.acknowledgedBy,
      acknowledged_at: alert.acknowledgedAt,
    }));
  }),

  ...route('post', 'master', '/alerts/:id/resolve', async ({ request, params }) => {
    const authErr = requireAuth(request);
    if (authErr) return authErr;
    const alert = db.andonAlerts.find((a) => a.id === params['id']);
    if (!alert) return problem(404, 'ERR-DB-002', 'NotFound', 'alert が存在しません');
    const body = (await request.json().catch(() => null)) as { resolved_by?: string; resolution_note?: string } | null;
    if (!body?.resolved_by) {
      return problem(422, 'ERR-VAL-001', 'Required field missing', 'resolved_by は必須です');
    }
    alert.status = 'resolved';
    alert.resolvedBy = body.resolved_by;
    alert.resolvedAt = new Date().toISOString();
    alert.resolutionNote = body.resolution_note ?? null;
    return HttpResponse.json(envelope(alert));
  }),

  ...route('post', 'master', '/nonconformities', async ({ request }) => {
    const authErr = requireAuth(request);
    if (authErr) return authErr;
    const body = (await request.json().catch(() => null)) as Partial<{
      alert_id: string;
      work_execution_id: string;
      lot_id: string;
      nc_type: Nonconformity['ncType'];
      description: string;
      discovered_by: string;
      discovery_step_id: string;
      evidence_ids: string[];
    }> | null;
    if (!body?.nc_type || !body.description || !body.discovered_by) {
      return problem(422, 'ERR-VAL-001', 'Required field missing', '必須フィールドが不足しています');
    }
    const nc: Nonconformity = {
      id: uuidv7(),
      alertId: body.alert_id ?? null,
      workExecutionId: body.work_execution_id ?? null,
      lotId: body.lot_id ?? null,
      ncType: body.nc_type,
      description: body.description,
      discoveredBy: body.discovered_by,
      discoveryStepId: body.discovery_step_id ?? null,
      evidenceIds: body.evidence_ids ?? [],
      createdAt: new Date().toISOString(),
    };
    db.nonconformities.push(nc);
    return HttpResponse.json(envelope(nc), { status: 201 });
  }),

  ...route('post', 'master', '/capas', async ({ request }) => {
    const authErr = requireAuth(request);
    if (authErr) return authErr;
    const body = (await request.json().catch(() => null)) as Partial<{
      nonconformity_id: string;
      title: string;
      root_cause_analysis: string;
      corrective_action: string;
      preventive_action: string;
      assigned_to: string;
      due_date: string;
      created_by: string;
    }> | null;
    if (!body?.title || !body.root_cause_analysis || !body.corrective_action || !body.assigned_to || !body.due_date || !body.created_by) {
      return problem(422, 'ERR-VAL-001', 'Required field missing', '必須フィールドが不足しています');
    }
    const capa: Capa = {
      id: uuidv7(),
      nonconformityId: body.nonconformity_id ?? null,
      title: body.title,
      status: 'open',
      rootCauseAnalysis: body.root_cause_analysis,
      correctiveAction: body.corrective_action,
      preventiveAction: body.preventive_action ?? null,
      assignedTo: body.assigned_to,
      dueDate: body.due_date,
      createdBy: body.created_by,
      createdAt: new Date().toISOString(),
      progressNote: null,
      closedAt: null,
      closedBy: null,
    };
    db.capas.push(capa);
    return HttpResponse.json(envelope({
      capa_id: capa.id,
      status: capa.status,
      title: capa.title,
      assigned_to: capa.assignedTo,
      due_date: capa.dueDate,
      created_at: capa.createdAt,
    }), { status: 201 });
  }),

  ...route('patch', 'master', '/capas/:id', async ({ request, params }) => {
    const authErr = requireAuth(request);
    if (authErr) return authErr;
    const capa = db.capas.find((c) => c.id === params['id']);
    if (!capa) return problem(404, 'ERR-DB-002', 'NotFound', 'capa が存在しません');
    if (capa.status === 'closed') {
      return problem(409, 'ERR-BIZ-008', 'CAPA already closed', 'CAPA は既にクローズされています');
    }
    const body = (await request.json().catch(() => null)) as Partial<{
      status: Capa['status'];
      progress_note: string;
      corrective_action: string;
      preventive_action: string;
      due_date: string;
      updated_by: string;
    }> | null;
    if (!body?.updated_by) {
      return problem(422, 'ERR-VAL-001', 'Required field missing', 'updated_by は必須です');
    }
    if (body.status) capa.status = body.status;
    if (body.progress_note !== undefined) capa.progressNote = body.progress_note;
    if (body.corrective_action) capa.correctiveAction = body.corrective_action;
    if (body.preventive_action) capa.preventiveAction = body.preventive_action;
    if (body.due_date) capa.dueDate = body.due_date;
    if (capa.status === 'closed') {
      capa.closedAt = new Date().toISOString();
      capa.closedBy = body.updated_by;
    }
    return HttpResponse.json(envelope(capa));
  }),

  ...route('post', 'terminal', '/kaizen-proposals', async ({ request }) => {
    const authErr = requireAuth(request);
    if (authErr) return authErr;
    const body = (await request.json().catch(() => null)) as Partial<{
      proposer_id: string;
      process_id: string;
      category: KaizenProposal['category'];
      title: string;
      current_situation: string;
      proposal_detail: string;
      expected_benefit: string;
      related_sop_id: string;
      evidence_ids: string[];
    }> | null;
    if (!body?.proposer_id || !body.category || !body.title || !body.current_situation || !body.proposal_detail) {
      return problem(422, 'ERR-VAL-001', 'Required field missing', '必須フィールドが不足しています');
    }
    if (body.evidence_ids && body.evidence_ids.length > 10) {
      return problem(422, 'ERR-VAL-002', 'Value out of range', 'evidence_ids は最大 10 件まで');
    }
    const proposal: KaizenProposal = {
      id: uuidv7(),
      proposerId: body.proposer_id,
      processId: body.process_id ?? null,
      category: body.category,
      title: body.title,
      currentSituation: body.current_situation,
      proposalDetail: body.proposal_detail,
      expectedBenefit: body.expected_benefit ?? null,
      relatedSopId: body.related_sop_id ?? null,
      evidenceIds: body.evidence_ids ?? [],
      status: 'submitted',
      createdAt: new Date().toISOString(),
    };
    db.kaizenProposals.push(proposal);
    return HttpResponse.json(envelope({
      proposal_id: proposal.id,
      status: proposal.status,
      title: proposal.title,
      proposer_id: proposal.proposerId,
      created_at: proposal.createdAt,
    }), { status: 201 });
  }),
];
