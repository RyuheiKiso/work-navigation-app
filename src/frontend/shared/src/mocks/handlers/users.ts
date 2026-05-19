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
import type { LocalizedText, User, UserRole } from '../../types';

const emptyLocalized: LocalizedText = { ja: '', en: '', zh: '' };

export const userHandlers = [
  ...route('get', 'master', '/master/users', ({ request }) => {
    const authErr = requireAuth(request);
    if (authErr) return authErr;
    const { page, perPage } = parsePagination(request);
    const users = db.users.filter((u) => u.deletedAt === null);
    const { slice, total } = paginate(users, page, perPage);
    return HttpResponse.json(paginatedEnvelope(slice, total, page, perPage));
  }),

  ...route('post', 'master', '/master/users', async ({ request }) => {
    const authErr = requireAuth(request);
    if (authErr) return authErr;
    const body = (await request.json().catch(() => null)) as {
      login_id?: string;
      display_name?: string;
      email?: string | null;
      factory_id?: string;
      roles?: UserRole[];
    } | null;
    if (!body?.login_id || !body.factory_id || !body.roles?.length) {
      return problem(422, 'ERR-VAL-001', 'Required field missing', '必須フィールドが不足しています');
    }
    const role: UserRole = body.roles[0]!;
    const user: User = {
      id: uuidv7(),
      loginId: body.login_id,
      username: body.login_id,
      displayNameJson: { ja: body.display_name ?? body.login_id, en: '', zh: '' },
      email: body.email ?? null,
      role,
      roles: body.roles,
      factoryId: body.factory_id,
      locale: 'ja',
      isActive: true,
      createdAt: new Date().toISOString(),
      deletedAt: null,
    };
    db.users.push(user);
    return HttpResponse.json(envelope(user), { status: 201 });
  }),

  ...route('get', 'master', '/master/users/:id', ({ request, params }) => {
    const authErr = requireAuth(request);
    if (authErr) return authErr;
    const user = db.users.find((u) => u.id === params['id']);
    if (!user) return problem(404, 'ERR-DB-002', 'NotFound', 'user が存在しません');
    return HttpResponse.json(envelope(user));
  }),

  ...route('patch', 'master', '/master/users/:id', async ({ request, params }) => {
    const authErr = requireAuth(request);
    if (authErr) return authErr;
    const user = db.users.find((u) => u.id === params['id']);
    if (!user) return problem(404, 'ERR-DB-002', 'NotFound', 'user が存在しません');
    const body = (await request.json().catch(() => null)) as Partial<User> | null;
    if (!body) return problem(422, 'ERR-VAL-001', 'Required field missing', 'リクエストボディが必要です');
    Object.assign(user, body);
    return HttpResponse.json(envelope(user));
  }),
];
