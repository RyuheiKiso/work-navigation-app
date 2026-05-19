// 作業実行は status・currentStepId・completedStepCount を進行に応じて更新する
import { getDataSource } from '../data-source';
import { LocalWorkExecution } from '../entities/LocalWorkExecution';

export class WorkExecutionRepository {
  private get repo() {
    return getDataSource().getRepository(LocalWorkExecution);
  }

  async findById(id: string): Promise<LocalWorkExecution | null> {
    return this.repo.findOne({ where: { id } });
  }

  async create(exec: LocalWorkExecution): Promise<LocalWorkExecution> {
    return this.repo.save(exec);
  }

  async updateStatus(id: string, status: string): Promise<void> {
    await this.repo.update({ id }, { status, lastEventAt: new Date().toISOString() });
  }

  async setCurrentStep(id: string, stepId: string): Promise<void> {
    await this.repo.update({ id }, { currentStepId: stepId, lastEventAt: new Date().toISOString() });
  }

  async incrementCompletedSteps(id: string): Promise<void> {
    const current = await this.findById(id);
    if (current === null) return;
    await this.repo.update(
      { id },
      { completedStepCount: current.completedStepCount + 1, lastEventAt: new Date().toISOString() },
    );
  }
}
