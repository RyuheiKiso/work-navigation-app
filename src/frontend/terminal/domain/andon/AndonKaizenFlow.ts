// アンドン発報・不適合登録・改善提案を含むカイゼンフロー
import { generateId } from '@wnav/shared/domain/id';
import { getDataSource } from '../../db/data-source';
import { LocalAndonAlert } from '../../db/entities/LocalAndonAlert';
import { LocalNonconformity } from '../../db/entities/LocalNonconformity';
import { LocalKaizenProposal } from '../../db/entities/LocalKaizenProposal';

export interface RaiseAndonParams {
  alertType: string;
  severity: 'low' | 'medium' | 'high' | 'critical';
  raisedBy: string;
  title: string;
  description: string;
  workExecutionId?: string;
  stepId?: string;
}

export interface RegisterNonconformityParams {
  alertId?: string;
  workExecutionId?: string;
  lotId?: string;
  ncType: string;
  description: string;
  discoveredBy: string;
  discoveryStepId?: string;
  evidenceIds: string[];
}

export interface SubmitKaizenParams {
  proposerId: string;
  processId?: string;
  category: string;
  title: string;
  currentSituation: string;
  proposalDetail: string;
  expectedBenefit?: string;
  relatedSopId?: string;
  evidenceIds: string[];
}

export class AndonKaizenFlow {
  async raiseAndon(params: RaiseAndonParams): Promise<LocalAndonAlert> {
    const entity: LocalAndonAlert = {
      id: generateId(),
      alertType: params.alertType,
      severity: params.severity,
      status: 'open',
      workExecutionId: params.workExecutionId ?? null,
      stepId: params.stepId ?? null,
      raisedBy: params.raisedBy,
      title: params.title,
      description: params.description,
      raisedAt: new Date().toISOString(),
      acknowledgedBy: null,
      acknowledgedAt: null,
      resolvedBy: null,
      resolvedAt: null,
      resolutionNote: null,
    };
    return getDataSource().getRepository(LocalAndonAlert).save(entity);
  }

  async registerNonconformity(params: RegisterNonconformityParams): Promise<LocalNonconformity> {
    const entity: LocalNonconformity = {
      id: generateId(),
      alertId: params.alertId ?? null,
      workExecutionId: params.workExecutionId ?? null,
      lotId: params.lotId ?? null,
      ncType: params.ncType,
      description: params.description,
      discoveredBy: params.discoveredBy,
      discoveryStepId: params.discoveryStepId ?? null,
      evidenceIds: JSON.stringify(params.evidenceIds),
      createdAt: new Date().toISOString(),
    };
    return getDataSource().getRepository(LocalNonconformity).save(entity);
  }

  async submitKaizen(params: SubmitKaizenParams): Promise<LocalKaizenProposal> {
    const entity: LocalKaizenProposal = {
      id: generateId(),
      proposerId: params.proposerId,
      processId: params.processId ?? null,
      category: params.category,
      title: params.title,
      currentSituation: params.currentSituation,
      proposalDetail: params.proposalDetail,
      expectedBenefit: params.expectedBenefit ?? null,
      relatedSopId: params.relatedSopId ?? null,
      evidenceIds: JSON.stringify(params.evidenceIds),
      status: 'submitted',
      createdAt: new Date().toISOString(),
    };
    return getDataSource().getRepository(LocalKaizenProposal).save(entity);
  }
}
