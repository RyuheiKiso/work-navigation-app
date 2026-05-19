// case_locks は例外的に UPDATE/DELETE 許可（src/CLAUDE.md §マルチデバイス排他原則）
import { getDataSource } from '../data-source';
import { LocalCaseLock } from '../entities/LocalCaseLock';

export class CaseLockRepository {
  private get repo() {
    return getDataSource().getRepository(LocalCaseLock);
  }

  async find(caseId: string): Promise<LocalCaseLock | null> {
    return this.repo.findOne({ where: { caseId } });
  }

  async upsert(lock: LocalCaseLock): Promise<void> {
    await this.repo.save(lock);
  }

  async heartbeat(caseId: string, heartbeatAt: string): Promise<void> {
    await this.repo.update({ caseId }, { heartbeatAt });
  }

  async release(caseId: string): Promise<void> {
    await this.repo.update({ caseId }, { lockStatus: 'RELEASED' });
  }
}
