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
import type { LocalizedText, MasterVersion, Sop } from '../../types';

const emptyLocalized: LocalizedText = { ja: '', en: '', zh: '' };

export const sopWorkflowHandlers = [
  ...route('get', 'master', '/master/sops', ({ request }) => {
    const { page, perPage } = parsePagination(request);
    const u = new URL(request.url);
    const processId = u.searchParams.get('process_id');
    const hasPublished = u.searchParams.get('has_published_version');
    let sops = db.sops.filter((s) => s.deletedAt === null);
    if (processId) sops = sops.filter((s) => s.processId === processId);
    if (hasPublished === 'true') sops = sops.filter((s) => s.currentVersionId !== null);
    const { slice, total } = paginate(sops, page, perPage);
    return HttpResponse.json(paginatedEnvelope(slice, total, page, perPage));
  }),

  ...route('post', 'master', '/master/sops', async ({ request }) => {
    const authErr = requireAuth(request);
    if (authErr) return authErr;
    const body = (await request.json().catch(() => null)) as Partial<Sop> | null;
    if (!body) return problem(422, 'ERR-VAL-001', 'Required field missing', 'リクエストボディが必要です');
    const sop: Sop = {
      id: uuidv7(),
      sopCode: body.sopCode ?? `SOP-${Date.now()}`,
      nameJson: body.nameJson ?? emptyLocalized,
      descriptionJson: body.descriptionJson ?? emptyLocalized,
      sopType: body.sopType ?? 'STANDARD',
      processId: body.processId ?? '',
      operationId: body.operationId ?? '',
      currentVersionId: null,
      deletedAt: null,
    };
    db.sops.push(sop);
    return HttpResponse.json(envelope(sop), { status: 201 });
  }),

  ...route('get', 'any', '/master/sops/:id', ({ params }) => {
    const sop = db.sops.find((s) => s.id === params['id']);
    if (!sop) return problem(404, 'ERR-DB-002', 'NotFound', 'sop が存在しません');
    return HttpResponse.json(envelope(sop));
  }),

  ...route('patch', 'master', '/master/sops/:id', async ({ request, params }) => {
    const authErr = requireAuth(request);
    if (authErr) return authErr;
    const sop = db.sops.find((s) => s.id === params['id']);
    if (!sop) return problem(404, 'ERR-DB-002', 'NotFound', 'sop が存在しません');
    const body = (await request.json().catch(() => null)) as Partial<Sop> | null;
    if (!body) return problem(422, 'ERR-VAL-001', 'Required field missing', 'リクエストボディが必要です');
    Object.assign(sop, body);
    return HttpResponse.json(envelope(sop));
  }),

  // /submit: OpenAPI operationId submitSopForReview（API-master-004）
  ...route('post', 'master', '/master/sops/:id/submit', async ({ request, params }) => {
    const authErr = requireAuth(request);
    if (authErr) return authErr;
    const version = db.sopVersions.find((v) => v.sopId === params['id'] && v.status === 'draft');
    if (!version) return problem(409, 'ERR-BIZ-003', 'SOP version not in draft', 'draft バージョンが見つかりません');
    version.status = 'in_review';
    version.submittedAt = new Date().toISOString();
    return HttpResponse.json(envelope(version));
  }),

  // /review: 後方互換性のため残す（/submit へのエイリアス）
  ...route('post', 'master', '/master/sops/:id/review', async ({ request, params }) => {
    const authErr = requireAuth(request);
    if (authErr) return authErr;
    const version = db.sopVersions.find((v) => v.sopId === params['id'] && v.status === 'draft');
    if (!version) return problem(409, 'ERR-BIZ-003', 'SOP version not in draft', 'draft バージョンが見つかりません');
    version.status = 'in_review';
    version.submittedAt = new Date().toISOString();
    return HttpResponse.json(envelope(version));
  }),

  ...route('post', 'master', '/master/sops/:id/approve', async ({ request, params }) => {
    const authErr = requireAuth(request);
    if (authErr) return authErr;
    const body = (await request.json().catch(() => null)) as { approved_by?: string; electronic_sign_id?: string } | null;
    if (!body?.approved_by || !body.electronic_sign_id) {
      return problem(422, 'ERR-VAL-001', 'Required field missing', 'approved_by と electronic_sign_id は必須です');
    }
    const version = db.sopVersions.find((v) => v.sopId === params['id'] && v.status === 'in_review');
    if (!version) return problem(409, 'ERR-BIZ-003', 'SOP not in review', 'in_review バージョンが見つかりません');
    version.status = 'published';
    version.publishedAt = new Date().toISOString();
    version.approvedBy = body.approved_by;
    version.approvedAt = new Date().toISOString();
    return HttpResponse.json(envelope(version));
  }),

  ...route('post', 'master', '/master/sops/:id/publish', ({ request, params }) => {
    const authErr = requireAuth(request);
    if (authErr) return authErr;
    const version = db.sopVersions.find((v) => v.sopId === params['id']);
    if (!version) return problem(404, 'ERR-DB-002', 'NotFound', 'sop_version が存在しません');
    version.status = 'published';
    version.publishedAt = new Date().toISOString();
    return HttpResponse.json(envelope(version));
  }),

  ...route('post', 'master', '/master/sops/:id/deprecate', ({ request, params }) => {
    const authErr = requireAuth(request);
    if (authErr) return authErr;
    const version = db.sopVersions.find((v) => v.sopId === params['id'] && v.status === 'published');
    if (!version) return problem(409, 'ERR-BIZ-005', 'SOP frozen', '廃止対象が見つかりません');
    version.status = 'deprecated';
    version.deprecatedAt = new Date().toISOString();
    return HttpResponse.json(envelope(version));
  }),

  ...route('get', 'master', '/master/sops/:id/versions', ({ request, params }) => {
    const { page, perPage } = parsePagination(request);
    const versions: MasterVersion[] = db.sopVersions.filter((v) => v.sopId === params['id']);
    const { slice, total } = paginate(versions, page, perPage);
    return HttpResponse.json(paginatedEnvelope(slice, total, page, perPage));
  }),
];
