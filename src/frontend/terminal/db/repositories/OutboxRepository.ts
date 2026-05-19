// Outbox は created_at 昇順で 1 件ずつ取得し ACK 後にレコードのみ削除する（イベント本体は保持）
import { getDataSource } from '../data-source';
import { LocalOutboxEvent } from '../entities/LocalOutboxEvent';

export type OutboxEnqueueInput = Omit<LocalOutboxEvent, 'id'>;

export class OutboxRepository {
  private get repo() {
    return getDataSource().getRepository(LocalOutboxEvent);
  }

  async enqueue(input: OutboxEnqueueInput): Promise<LocalOutboxEvent> {
    const entity = this.repo.create(input);
    return this.repo.save(entity);
  }

  async findOldestPending(): Promise<LocalOutboxEvent | null> {
    return this.repo.findOne({
      where: { sent: false },
      order: { createdAt: 'ASC' },
    });
  }

  async delete(id: number): Promise<void> {
    await this.repo.delete({ id });
  }

  async markRetry(id: number, retryCount: number, nextRetryAt: string, error: string): Promise<void> {
    await this.repo.update({ id }, { retryCount, nextRetryAt, lastError: error });
  }

  async pendingCount(): Promise<number> {
    return this.repo.count({ where: { sent: false } });
  }
}
