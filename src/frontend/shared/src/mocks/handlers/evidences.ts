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
  storeIdempotency,
  withIdempotency,
} from '../_helpers';
import { db } from '../db/seed';
import type { EvidenceFile } from '../../types';

interface EvidenceMetadata {
  work_execution_id: string;
  step_id: string;
  evidence_type: 'photo' | 'document' | 'measurement_sheet';
  description?: string;
  timestamp_client: string;
  sha256_client: string;
}

interface ElectronicSignBody {
  signer_id?: string;
  signed_content_hash?: string;
  pin_hash?: string;
  context_type?: 'step_sign' | 'work_complete_sign' | 'approval_sign' | 'quality_check_sign';
  context_id?: string;
  step_id?: string;
  timestamp_client?: string;
  device_signature?: string;
}

export const evidenceHandlers = [
  ...route('post', 'terminal', '/evidences', async ({ request }) => {
    const authErr = requireAuth(request);
    if (authErr) return authErr;
    const formData = await request.formData().catch(() => null);
    if (!formData) return problem(422, 'ERR-VAL-001', 'Required field missing', 'multipart/form-data が必要です');
    const metaRaw = formData.get('metadata');
    const file = formData.get('file');
    if (typeof metaRaw !== 'string' || !file) {
      return problem(422, 'ERR-VAL-001', 'Required field missing', 'metadata と file は必須です');
    }
    let metadata: EvidenceMetadata;
    try {
      metadata = JSON.parse(metaRaw) as EvidenceMetadata;
    } catch {
      return problem(422, 'ERR-VAL-003', 'Invalid format', 'metadata の JSON 形式が不正です');
    }
    const idem = await withIdempotency<EvidenceFile>(request, metadata);
    if (idem.conflict) return idem.conflict;
    if (idem.cached) return HttpResponse.json(envelope(idem.cached.response), { status: idem.cached.status });

    const evidence: EvidenceFile = {
      id: uuidv7(),
      workExecutionId: metadata.work_execution_id,
      stepId: metadata.step_id,
      evidenceType: metadata.evidence_type,
      filePath: `/evidences/${new Date().toISOString().slice(0, 10)}/${uuidv7()}.bin`,
      fileHashSha256: metadata.sha256_client,
      fileSizeBytes: file instanceof File ? file.size : 0,
      widthPx: null,
      heightPx: null,
      description: metadata.description ?? '',
      uploadedBy: '00000000-0000-7000-0000-000000000000',
      uploadedAt: new Date().toISOString(),
    };
    db.evidenceFiles.push(evidence);
    storeIdempotency(idem.key, idem.bodyHash, evidence, 201);
    return HttpResponse.json(envelope(evidence), { status: 201 });
  }),

  // master アプリからの承認サイン（approval_sign）: signer_id をサーバー側で解決する前提でリクエスト形式が異なる
  ...route('post', 'master', '/electronic-signs', async ({ request }) => {
    const authErr = requireAuth(request);
    if (authErr) return authErr;
    const body = (await request.json().catch(() => null)) as { context_type?: string; context_id?: string; signature_base64?: string; pin?: string } | null;
    if (!body?.context_type || !body.context_id) {
      return problem(422, 'ERR-VAL-001', 'Required field missing', 'context_type と context_id は必須です');
    }
    const idem = await withIdempotency<{ id: string }>(request, body);
    if (idem.conflict) return idem.conflict;
    if (idem.cached) return HttpResponse.json(envelope(idem.cached.response), { status: idem.cached.status });

    const signedAt = new Date().toISOString();
    const lastBlock = db.hashChainBlocks[db.hashChainBlocks.length - 1];
    const prevHash = lastBlock?.contentHash ?? '0'.repeat(64);
    const signId = uuidv7();
    const block = {
      id: uuidv7(),
      blockNumber: db.hashChainBlocks.length + 1,
      prevHash,
      contentHash: 'mock-' + signId.replaceAll('-', '').padEnd(64, '0').slice(0, 64),
      payload: JSON.stringify({ signId }),
      createdAt: signedAt,
    };
    db.hashChainBlocks.push(block);
    // ページが期待する形式は { id: string } のため id フィールドで統一する
    const response = { id: signId };
    storeIdempotency(idem.key, idem.bodyHash, response, 201);
    return HttpResponse.json(envelope(response), { status: 201 });
  }),

  ...route('post', 'terminal', '/electronic-signs', async ({ request }) => {
    const authErr = requireAuth(request);
    if (authErr) return authErr;
    const body = (await request.json().catch(() => null)) as ElectronicSignBody | null;
    if (!body?.signer_id || !body.signed_content_hash || !body.context_type || !body.context_id) {
      return problem(422, 'ERR-VAL-001', 'Required field missing', '必須フィールドが不足しています');
    }
    const idem = await withIdempotency<unknown>(request, body);
    if (idem.conflict) return idem.conflict;

    const signedAt = new Date().toISOString();
    const lastBlock = db.hashChainBlocks[db.hashChainBlocks.length - 1];
    const prevHash = lastBlock?.contentHash ?? '0'.repeat(64);
    const signId = uuidv7();
    const block = {
      id: uuidv7(),
      blockNumber: db.hashChainBlocks.length + 1,
      prevHash,
      contentHash: 'mock-' + signId.replaceAll('-', '').padEnd(64, '0').slice(0, 64),
      payload: JSON.stringify({ signId }),
      createdAt: signedAt,
    };
    db.hashChainBlocks.push(block);
    const sign = {
      id: signId,
      signerId: body.signer_id,
      signedContentHash: body.signed_content_hash,
      contextType: body.context_type,
      contextId: body.context_id,
      stepId: body.step_id ?? null,
      signedAt,
      hashChainBlockId: block.id,
      hashChainValue: `sha256:${block.contentHash}`,
      hashChainPrev: `sha256:${prevHash}`,
      deviceId: '',
    };
    db.electronicSigns.push(sign);
    const response = {
      sign_id: signId,
      signer_id: body.signer_id,
      signed_content_hash: body.signed_content_hash,
      context_type: body.context_type,
      context_id: body.context_id,
      signed_at: signedAt,
      hash_chain_block_id: block.id,
      hash_chain_value: `sha256:${block.contentHash}`,
    };
    storeIdempotency(idem.key, idem.bodyHash, response, 201);
    return HttpResponse.json(envelope(response), { status: 201 });
  }),

  ...route('get', 'terminal', '/electronic-signs', ({ request }) => {
    const authErr = requireAuth(request);
    if (authErr) return authErr;
    const u = new URL(request.url);
    const signerId = u.searchParams.get('signer_id');
    const { page, perPage } = parsePagination(request);
    let signs = db.electronicSigns.slice();
    if (signerId) signs = signs.filter((s) => s.signerId === signerId);
    const { slice, total } = paginate(signs, page, perPage);
    return HttpResponse.json(paginatedEnvelope(slice, total, page, perPage));
  }),
];
