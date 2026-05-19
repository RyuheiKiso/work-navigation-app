// エビデンスは Append-only。SHA-256 ハッシュは Exif 除去後の値で固定する
import { getDataSource } from '../data-source';
import { LocalEvidenceFile } from '../entities/LocalEvidenceFile';

export class EvidenceRepository {
  private get repo() {
    return getDataSource().getRepository(LocalEvidenceFile);
  }

  async append(file: LocalEvidenceFile): Promise<LocalEvidenceFile> {
    return this.repo.save(file);
  }

  async findByStepId(stepId: string): Promise<LocalEvidenceFile[]> {
    return this.repo.find({ where: { stepId }, order: { uploadedAt: 'ASC' } });
  }

  async findUnsynced(limit: number): Promise<LocalEvidenceFile[]> {
    return this.repo.find({ where: { synced: false }, order: { uploadedAt: 'ASC' }, take: limit });
  }

  async markSynced(id: string): Promise<void> {
    await this.repo.update({ id }, { synced: true });
  }
}
