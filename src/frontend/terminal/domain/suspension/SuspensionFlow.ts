// 中断フロー。中断理由・コメントを保存し、再開時に resumedAt を埋める
import { IsNull } from 'typeorm';
import { generateId } from '@wnav/shared/domain/id';
import { getDataSource } from '../../db/data-source';
import { LocalSuspension } from '../../db/entities/LocalSuspension';
import { WorkExecutionRepository } from '../../db/repositories/WorkExecutionRepository';

export type SuspendReason =
  | 'equipment_breakdown'
  | 'material_shortage'
  | 'quality_issue'
  | 'emergency'
  | 'other';

export interface SuspendParams {
  workExecutionId: string;
  reasonCode: SuspendReason;
  reasonDetail: string;
}

export interface ResumeParams {
  suspensionId: string;
  resumedBy: string;
}

export class SuspensionFlow {
  private readonly workExecutionRepo: WorkExecutionRepository;

  constructor() {
    this.workExecutionRepo = new WorkExecutionRepository();
  }

  async suspend(params: SuspendParams): Promise<LocalSuspension> {
    const repo = getDataSource().getRepository(LocalSuspension);
    const entity: LocalSuspension = {
      id: generateId(),
      workExecutionId: params.workExecutionId,
      reasonCode: params.reasonCode,
      reasonDetail: params.reasonDetail,
      suspendedAt: new Date().toISOString(),
      resumedAt: null,
      resumedBy: null,
    };
    const saved = await repo.save(entity);
    await this.workExecutionRepo.updateStatus(params.workExecutionId, 'suspended');
    return saved;
  }

  async resume(params: ResumeParams): Promise<void> {
    const repo = getDataSource().getRepository(LocalSuspension);
    const now = new Date().toISOString();
    await repo.update({ id: params.suspensionId }, { resumedAt: now, resumedBy: params.resumedBy });
    const suspension = await repo.findOne({ where: { id: params.suspensionId } });
    if (suspension !== null) {
      await this.workExecutionRepo.updateStatus(suspension.workExecutionId, 'in_progress');
    }
  }

  async listOpenSuspensions(): Promise<LocalSuspension[]> {
    const repo = getDataSource().getRepository(LocalSuspension);
    return repo.find({ where: { resumedAt: IsNull() }, order: { suspendedAt: 'DESC' } });
  }
}
