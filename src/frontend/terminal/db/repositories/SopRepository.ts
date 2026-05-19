// SOP は読み取り中心。マスタ差分取得時にのみ INSERT/UPDATE を許可する
import { getDataSource } from '../data-source';
import { LocalSop } from '../entities/LocalSop';
import { LocalStep } from '../entities/LocalStep';

export class SopRepository {
  private get sopRepo() {
    return getDataSource().getRepository(LocalSop);
  }

  private get stepRepo() {
    return getDataSource().getRepository(LocalStep);
  }

  async findById(id: string): Promise<LocalSop | null> {
    return this.sopRepo.findOne({ where: { id } });
  }

  async findActiveSops(): Promise<LocalSop[]> {
    return this.sopRepo.find({ where: { deletedAt: null as unknown as string }, order: { sopCode: 'ASC' } });
  }

  async findStepsBySopVersionId(sopVersionId: string): Promise<LocalStep[]> {
    return this.stepRepo.find({ where: { sopVersionId }, order: { stepNumber: 'ASC' } });
  }

  async upsertSop(sop: LocalSop): Promise<void> {
    await this.sopRepo.save(sop);
  }

  async upsertStep(step: LocalStep): Promise<void> {
    await this.stepRepo.save(step);
  }
}
