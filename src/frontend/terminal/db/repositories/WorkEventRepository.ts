// 作業ログは Append-only のため INSERT のみを提供し UPDATE/DELETE メソッドは意図的に実装しない
import { getDataSource } from '../data-source';
import { LocalWorkEvent } from '../entities/LocalWorkEvent';

export class WorkEventRepository {
  private get repo() {
    return getDataSource().getRepository(LocalWorkEvent);
  }

  async append(event: LocalWorkEvent): Promise<LocalWorkEvent> {
    return this.repo.save(event);
  }

  async findByCaseId(caseId: string): Promise<LocalWorkEvent[]> {
    return this.repo.find({ where: { caseId }, order: { timestampClient: 'ASC' } });
  }

  async findLatestByCaseId(caseId: string): Promise<LocalWorkEvent | null> {
    return this.repo.findOne({ where: { caseId }, order: { timestampClient: 'DESC' } });
  }

  async findUnsynced(limit: number): Promise<LocalWorkEvent[]> {
    return this.repo.find({
      where: { synced: false },
      order: { timestampClient: 'ASC' },
      take: limit,
    });
  }

  async markSynced(eventId: string): Promise<void> {
    // synced フラグだけは送信完了の事実を反映するため例外的に UPDATE する
    await this.repo.update({ eventId }, { synced: true });
  }
}
