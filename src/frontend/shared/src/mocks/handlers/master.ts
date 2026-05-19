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
  storeIdempotency,
  withIdempotency,
} from '../_helpers';
import { db } from '../db/seed';
import type { LocalizedText, Operation, Process, Product } from '../../types';

function listResource<T extends { deletedAt: string | null }>(items: T[], request: Request) {
  const { page, perPage } = parsePagination(request);
  const active = items.filter((i) => i.deletedAt === null);
  const { slice, total } = paginate(active, page, perPage);
  return HttpResponse.json(paginatedEnvelope(slice, total, page, perPage));
}

const emptyLocalized: LocalizedText = { ja: '', en: '', zh: '' };

export const masterHandlers = [
  ...route('get', 'any', '/master/processes', ({ request }) => listResource(db.processes, request)),

  ...route('post', 'master', '/master/processes', async ({ request }) => {
    const authErr = requireAuth(request);
    if (authErr) return authErr;
    const body = (await request.json().catch(() => null)) as Partial<Process> | null;
    if (!body) return problem(422, 'ERR-VAL-001', 'Required field missing', 'リクエストボディが必要です');
    const idem = await withIdempotency<Process>(request, body);
    if (idem.conflict) return idem.conflict;
    const process: Process = {
      id: uuidv7(),
      processCode: body.processCode ?? `PROC-${Date.now()}`,
      nameJson: body.nameJson ?? emptyLocalized,
      descriptionJson: body.descriptionJson ?? emptyLocalized,
      isActive: body.isActive ?? true,
      deletedAt: null,
    };
    db.processes.push(process);
    storeIdempotency(idem.key, idem.bodyHash, process, 201);
    return HttpResponse.json(envelope(process), { status: 201 });
  }),

  ...route('get', 'any', '/master/processes/:id', ({ params }) => {
    const item = db.processes.find((p) => p.id === params['id']);
    if (!item) return problem(404, 'ERR-DB-002', 'NotFound', 'process が存在しません');
    return HttpResponse.json(envelope(item));
  }),

  ...route('patch', 'master', '/master/processes/:id', async ({ request, params }) => {
    const authErr = requireAuth(request);
    if (authErr) return authErr;
    const item = db.processes.find((p) => p.id === params['id']);
    if (!item) return problem(404, 'ERR-DB-002', 'NotFound', 'process が存在しません');
    const body = (await request.json().catch(() => null)) as Partial<Process> | null;
    if (!body) return problem(422, 'ERR-VAL-001', 'Required field missing', 'リクエストボディが必要です');
    Object.assign(item, body);
    return HttpResponse.json(envelope(item));
  }),

  ...route('get', 'any', '/master/operations', ({ request }) => listResource(db.operations, request)),
  ...route('post', 'master', '/master/operations', async ({ request }) => {
    const authErr = requireAuth(request);
    if (authErr) return authErr;
    const body = (await request.json().catch(() => null)) as Partial<Operation> | null;
    if (!body) return problem(422, 'ERR-VAL-001', 'Required field missing', 'リクエストボディが必要です');
    const op: Operation = {
      id: uuidv7(),
      operationCode: body.operationCode ?? `OP-${Date.now()}`,
      nameJson: body.nameJson ?? emptyLocalized,
      processId: body.processId ?? '',
      deletedAt: null,
    };
    db.operations.push(op);
    return HttpResponse.json(envelope(op), { status: 201 });
  }),
  ...route('get', 'any', '/master/operations/:id', ({ params }) => {
    const item = db.operations.find((o) => o.id === params['id']);
    if (!item) return problem(404, 'ERR-DB-002', 'NotFound', 'operation が存在しません');
    return HttpResponse.json(envelope(item));
  }),
  ...route('patch', 'master', '/master/operations/:id', async ({ request, params }) => {
    const authErr = requireAuth(request);
    if (authErr) return authErr;
    const item = db.operations.find((o) => o.id === params['id']);
    if (!item) return problem(404, 'ERR-DB-002', 'NotFound', 'operation が存在しません');
    const body = (await request.json().catch(() => null)) as Partial<Operation> | null;
    if (!body) return problem(422, 'ERR-VAL-001', 'Required field missing', 'リクエストボディが必要です');
    Object.assign(item, body);
    return HttpResponse.json(envelope(item));
  }),

  ...route('get', 'any', '/master/products', ({ request }) => listResource(db.products, request)),
  ...route('post', 'master', '/master/products', async ({ request }) => {
    const authErr = requireAuth(request);
    if (authErr) return authErr;
    const body = (await request.json().catch(() => null)) as Partial<Product> | null;
    if (!body) return problem(422, 'ERR-VAL-001', 'Required field missing', 'リクエストボディが必要です');
    const product: Product = {
      id: uuidv7(),
      productCode: body.productCode ?? `PRD-${Date.now()}`,
      nameJson: body.nameJson ?? emptyLocalized,
      deletedAt: null,
    };
    db.products.push(product);
    return HttpResponse.json(envelope(product), { status: 201 });
  }),
  ...route('get', 'any', '/master/products/:id', ({ params }) => {
    const item = db.products.find((p) => p.id === params['id']);
    if (!item) return problem(404, 'ERR-DB-002', 'NotFound', 'product が存在しません');
    return HttpResponse.json(envelope(item));
  }),
  ...route('patch', 'master', '/master/products/:id', async ({ request, params }) => {
    const authErr = requireAuth(request);
    if (authErr) return authErr;
    const item = db.products.find((p) => p.id === params['id']);
    if (!item) return problem(404, 'ERR-DB-002', 'NotFound', 'product が存在しません');
    const body = (await request.json().catch(() => null)) as Partial<Product> | null;
    if (!body) return problem(422, 'ERR-VAL-001', 'Required field missing', 'リクエストボディが必要です');
    Object.assign(item, body);
    return HttpResponse.json(envelope(item));
  }),

  ...route('get', 'any', '/master/roles', () =>
    HttpResponse.json(envelope([
      { id: 'operator', label: '作業者' },
      { id: 'supervisor', label: '監督者' },
      { id: 'quality_admin', label: '品質管理者' },
      { id: 'master_admin', label: 'マスタ管理者' },
      { id: 'system_admin', label: 'システム管理者' },
      { id: 'executive', label: '経営層' },
    ])),
  ),

  ...route('get', 'any', '/master/skills', ({ request }) => listResource(db.skills, request)),

  ...route('get', 'any', '/master/materials', ({ request }) => listResource(db.materials, request)),
  ...route('post', 'master', '/master/materials', async ({ request }) => {
    const authErr = requireAuth(request);
    if (authErr) return authErr;
    const body = (await request.json().catch(() => null)) as Partial<typeof db.materials[number]> | null;
    if (!body) return problem(422, 'ERR-VAL-001', 'Required field missing', 'リクエストボディが必要です');
    const material = {
      id: uuidv7(),
      materialCode: body.materialCode ?? `MAT-${Date.now()}`,
      nameJson: body.nameJson ?? emptyLocalized,
      materialType: body.materialType ?? 'general',
      unit: body.unit ?? 'piece',
      deletedAt: null,
    };
    db.materials.push(material);
    return HttpResponse.json(envelope(material), { status: 201 });
  }),
  ...route('patch', 'master', '/master/materials/:id', async ({ request, params }) => {
    const authErr = requireAuth(request);
    if (authErr) return authErr;
    const item = db.materials.find((m) => m.id === params['id']);
    if (!item) return problem(404, 'ERR-DB-002', 'NotFound', 'material が存在しません');
    const body = (await request.json().catch(() => null)) as Partial<typeof item> | null;
    if (!body) return problem(422, 'ERR-VAL-001', 'Required field missing', 'リクエストボディが必要です');
    Object.assign(item, body);
    return HttpResponse.json(envelope(item));
  }),

  ...route('get', 'any', '/master/suppliers', ({ request }) => listResource(db.suppliers, request)),
  ...route('post', 'master', '/master/suppliers', async ({ request }) => {
    const authErr = requireAuth(request);
    if (authErr) return authErr;
    const body = (await request.json().catch(() => null)) as Partial<typeof db.suppliers[number]> | null;
    if (!body) return problem(422, 'ERR-VAL-001', 'Required field missing', 'リクエストボディが必要です');
    const supplier = {
      id: uuidv7(),
      supplierCode: body.supplierCode ?? `SUP-${Date.now()}`,
      nameJson: body.nameJson ?? emptyLocalized,
      contactEmail: body.contactEmail ?? null,
      deletedAt: null,
    };
    db.suppliers.push(supplier);
    return HttpResponse.json(envelope(supplier), { status: 201 });
  }),
  ...route('patch', 'master', '/master/suppliers/:id', async ({ request, params }) => {
    const authErr = requireAuth(request);
    if (authErr) return authErr;
    const item = db.suppliers.find((s) => s.id === params['id']);
    if (!item) return problem(404, 'ERR-DB-002', 'NotFound', 'supplier が存在しません');
    const body = (await request.json().catch(() => null)) as Partial<typeof item> | null;
    if (!body) return problem(422, 'ERR-VAL-001', 'Required field missing', 'リクエストボディが必要です');
    Object.assign(item, body);
    return HttpResponse.json(envelope(item));
  }),

  ...route('get', 'any', '/master/sampling-plans', ({ request }) => listResource(db.samplingPlans, request)),
  ...route('post', 'master', '/master/sampling-plans', async ({ request }) => {
    const authErr = requireAuth(request);
    if (authErr) return authErr;
    const body = (await request.json().catch(() => null)) as Partial<typeof db.samplingPlans[number]> | null;
    if (!body) return problem(422, 'ERR-VAL-001', 'Required field missing', 'リクエストボディが必要です');
    const plan = {
      id: uuidv7(),
      planCode: body.planCode ?? `PLAN-${Date.now()}`,
      nameJson: body.nameJson ?? emptyLocalized,
      aqlValue: body.aqlValue ?? 1.0,
      inspectionLevel: body.inspectionLevel ?? 'II' as const,
      planSnapshot: body.planSnapshot ?? '{}',
      deletedAt: null,
    };
    db.samplingPlans.push(plan);
    return HttpResponse.json(envelope(plan), { status: 201 });
  }),
  ...route('patch', 'master', '/master/sampling-plans/:id', async ({ request, params }) => {
    const authErr = requireAuth(request);
    if (authErr) return authErr;
    const item = db.samplingPlans.find((p) => p.id === params['id']);
    if (!item) return problem(404, 'ERR-DB-002', 'NotFound', 'sampling_plan が存在しません');
    const body = (await request.json().catch(() => null)) as Partial<typeof item> | null;
    if (!body) return problem(422, 'ERR-VAL-001', 'Required field missing', 'リクエストボディが必要です');
    Object.assign(item, body);
    return HttpResponse.json(envelope(item));
  }),
];
