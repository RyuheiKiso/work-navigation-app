// 作業指示割当は外部システム連携から受信し、ACK 時に acknowledgedAt を更新する
import { getDataSource } from '../data-source';
import { LocalWorkAssignment } from '../entities/LocalWorkAssignment';

export class WorkAssignmentRepository {
  private get repo() {
    return getDataSource().getRepository(LocalWorkAssignment);
  }

  async findById(id: string): Promise<LocalWorkAssignment | null> {
    return this.repo.findOne({ where: { id } });
  }

  async findPendingForTerminal(terminalId: string): Promise<LocalWorkAssignment[]> {
    return this.repo.find({
      where: { targetTerminalId: terminalId, status: 'pending' },
      order: { priority: 'DESC', receivedAt: 'ASC' },
    });
  }

  async upsert(assignment: LocalWorkAssignment): Promise<void> {
    await this.repo.save(assignment);
  }

  async acknowledge(id: string): Promise<void> {
    await this.repo.update({ id }, { status: 'acknowledged', acknowledgedAt: new Date().toISOString() });
  }

  async cancel(id: string): Promise<void> {
    await this.repo.update({ id }, { status: 'cancelled', cancelledAt: new Date().toISOString() });
  }
}
