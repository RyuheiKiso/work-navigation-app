// 作業指示は更新可能（status が in_progress → completed へ進む）
import { getDataSource } from '../data-source';
import { LocalWorkOrder } from '../entities/LocalWorkOrder';

export class WorkOrderRepository {
  private get repo() {
    return getDataSource().getRepository(LocalWorkOrder);
  }

  async findById(id: string): Promise<LocalWorkOrder | null> {
    return this.repo.findOne({ where: { id } });
  }

  async findOpenAssignedTo(userId: string): Promise<LocalWorkOrder[]> {
    return this.repo.find({
      where: { assignedTo: userId, status: 'open' },
      order: { scheduledStart: 'ASC' },
    });
  }

  async upsert(order: LocalWorkOrder): Promise<void> {
    await this.repo.save(order);
  }

  async updateStatus(id: string, status: string): Promise<void> {
    await this.repo.update({ id }, { status, updatedAt: new Date().toISOString() });
  }
}
