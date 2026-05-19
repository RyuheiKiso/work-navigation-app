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
import type { LocalizedText, MasterVersion, Sop, Step } from '../../types';

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

  ...route('get', 'any', '/master/sops/:id', ({ request, params }) => {
    const sop = db.sops.find((s) => s.id === params['id']);
    if (!sop) return problem(404, 'ERR-DB-002', 'NotFound', 'sop が存在しません');
    // as_of: 時点参照。MSW では publishedAt が as_of 以前の最新公開バージョンに対応する SOP を返す。
    // インメモリ DB には版ごとの SOP スナップショットがないため、実装上は現在の SOP を返す（開発用途）。
    const u = new URL(request.url);
    const asOf = u.searchParams.get('as_of');
    if (asOf) {
      // 指定時点以前に公開されたバージョンに関連する SOP を探す（簡易実装）
      const version = db.sopVersions
        .filter((v) => v.sopId === sop.id && v.status === 'published' && v.publishedAt != null && v.publishedAt <= asOf)
        .sort((a, b) => (b.publishedAt ?? '').localeCompare(a.publishedAt ?? ''))[0];
      if (version) {
        // 実際のスナップショットは別テーブルだが、MSW ではメタ情報をそのまま返す
        return HttpResponse.json(envelope({ ...sop, currentVersionId: version.id }));
      }
      // as_of が全公開版より前ならデータなし（初版扱い）
      return problem(404, 'ERR-DB-002', 'NotFound', 'as_of 時点に公開済み SOP が存在しません');
    }
    return HttpResponse.json(envelope(sop));
  }),

  ...route('get', 'master', '/master/sops/:id/steps', ({ params }) => {
    const sopId = params['id'];
    const sop = db.sops.find((s) => s.id === sopId);
    if (!sop) return problem(404, 'ERR-DB-002', 'NotFound', 'sop が存在しません');
    const steps: Step[] = (db.steps ?? []).filter((s: Step) => s.sopVersionId === (sop.currentVersionId ?? ''));
    return HttpResponse.json(envelope(steps));
  }),

  ...route('put', 'master', '/master/sops/:id/steps', async ({ request, params }) => {
    const authErr = requireAuth(request);
    if (authErr) return authErr;
    const sopId = params['id'];
    const sop = db.sops.find((s) => s.id === sopId);
    if (!sop) return problem(404, 'ERR-DB-002', 'NotFound', 'sop が存在しません');
    const body = (await request.json().catch(() => null)) as { steps?: Step[]; flowJson?: string } | null;
    if (!body) return problem(422, 'ERR-VAL-001', 'Required field missing', 'リクエストボディが必要です');
    // Steps を sopVersionId に紐付けて保存する（既存は上書き）
    const sopVersionId = sop.currentVersionId ?? sopId;
    db.steps = [
      ...db.steps.filter((s) => s.sopVersionId !== sopVersionId),
      ...(body.steps ?? []).map((s) => ({ ...s, sopVersionId })),
    ];
    return HttpResponse.json(envelope(body.steps ?? []));
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

  // 廃止前の影響範囲 dry-run: 紐付き作業指示数・作業実行数を返す
  ...route('get', 'master', '/master/sops/:id/impact', ({ params }) => {
    const sopId = params['id'];
    const workOrderCount = db.workOrders.filter((wo) => wo.sopId === sopId).length;
    const workExecutionCount = db.workExecutions.filter((we) =>
      db.workOrders.some((wo) => wo.sopId === sopId && wo.id === we.workOrderId),
    ).length;
    return HttpResponse.json(envelope({ workOrderCount, workExecutionCount }));
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
