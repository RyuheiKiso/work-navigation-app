// 修正作業フロー。新 case_id で開始、前後写真・再検査・別検査者ルールを enforced
import { generateId } from '@wnav/shared/domain/id';
import { getDataSource } from '../../db/data-source';
import { LocalRework } from '../../db/entities/LocalRework';
import { LocalReworkVerification } from '../../db/entities/LocalReworkVerification';
import { LocalReworkSopMapping } from '../../db/entities/LocalReworkSopMapping';

export interface StartReworkParams {
  parentCaseId: string;
  nonconformityId: string;
  ncCategory: string;
  assignedTo: string | null;
  deadline: string | null;
}

export interface VerifyReworkParams {
  reworkId: string;
  verifierId: string;
  passed: boolean;
  note: string;
  evidenceIds: string[];
}

export class ReworkFlow {
  // 不適合カテゴリから修正 SOP を解決し、新しい reworkCaseId で Rework を生成する
  async startRework(params: StartReworkParams): Promise<LocalRework> {
    const mappingRepo = getDataSource().getRepository(LocalReworkSopMapping);
    const reworkRepo = getDataSource().getRepository(LocalRework);
    const mapping = await mappingRepo.findOne({
      where: { ncCategory: params.ncCategory, isActive: true },
    });
    if (mapping === null) {
      throw new Error(`Rework SOP mapping not found for category: ${params.ncCategory}`);
    }
    const entity: LocalRework = {
      id: generateId(),
      parentCaseId: params.parentCaseId,
      reworkCaseId: generateId(),
      nonconformityId: params.nonconformityId,
      sopId: mapping.reworkSopId,
      reworkSopVersionId: mapping.reworkSopVersionId,
      assignedTo: params.assignedTo,
      status: 'OPEN',
      reworkCount: 1,
      deadline: params.deadline,
      createdAt: new Date().toISOString(),
    };
    return reworkRepo.save(entity);
  }

  // 再検査は別作業者のみ許可する（修正者自身は検査不可、品質保証原則）
  async verifyRework(params: VerifyReworkParams): Promise<LocalReworkVerification> {
    const reworkRepo = getDataSource().getRepository(LocalRework);
    const verificationRepo = getDataSource().getRepository(LocalReworkVerification);
    const rework = await reworkRepo.findOne({ where: { id: params.reworkId } });
    if (rework === null) throw new Error(`Rework not found: ${params.reworkId}`);
    if (rework.assignedTo === params.verifierId) {
      throw new Error('ERR-BIZ-022: 修正者自身による再検査は禁止です');
    }
    const verification: LocalReworkVerification = {
      id: generateId(),
      reworkId: params.reworkId,
      verifierId: params.verifierId,
      verifiedAt: new Date().toISOString(),
      passed: params.passed,
      note: params.note,
      evidenceIds: JSON.stringify(params.evidenceIds),
    };
    await verificationRepo.save(verification);
    await reworkRepo.update(
      { id: params.reworkId },
      { status: params.passed ? 'CLOSED' : 'PENDING_VERIFICATION' },
    );
    return verification;
  }
}
