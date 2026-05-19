// 証拠ファイル管理。撮影バイナリの SHA-256 を計算してエンティティに永続化する
import { sha256 } from '@noble/hashes/sha2';
import { bytesToHex } from '@noble/hashes/utils';
import { generateId } from '@wnav/shared/domain/id';
import { EvidenceRepository } from '../../db/repositories/EvidenceRepository';
import type { LocalEvidenceFile } from '../../db/entities/LocalEvidenceFile';

export interface CapturePhotoParams {
  workExecutionId: string;
  stepId: string;
  filePath: string;
  fileBytes: Uint8Array;
  width: number | null;
  height: number | null;
  description: string;
  uploadedBy: string;
}

export class EvidenceService {
  private readonly repo: EvidenceRepository;

  constructor() {
    this.repo = new EvidenceRepository();
  }

  // 写真撮影：Exif 除去済みのバイナリから SHA-256 を計算してエンティティを保存する
  async capturePhoto(params: CapturePhotoParams): Promise<LocalEvidenceFile> {
    const hash = bytesToHex(sha256(params.fileBytes));
    const entity: LocalEvidenceFile = {
      id: generateId(),
      workExecutionId: params.workExecutionId,
      stepId: params.stepId,
      evidenceType: 'photo',
      filePath: params.filePath,
      fileHashSha256: hash,
      fileSizeBytes: params.fileBytes.byteLength,
      widthPx: params.width,
      heightPx: params.height,
      description: params.description,
      uploadedBy: params.uploadedBy,
      uploadedAt: new Date().toISOString(),
      synced: false,
    };
    return this.repo.append(entity);
  }

  async findByStepId(stepId: string): Promise<LocalEvidenceFile[]> {
    return this.repo.findByStepId(stepId);
  }
}
