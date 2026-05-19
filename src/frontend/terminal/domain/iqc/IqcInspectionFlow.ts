// 受入検査フロー。サンプリング計画・測定値登録・AQL 合否判定を担当
import { judgeAql, resolveSamplingPlan } from '@wnav/shared/domain/aql';
import type { AqlVerdict, InspectionLevel, SeverityState } from '@wnav/shared/domain/aql';
import { generateId } from '@wnav/shared/domain/id';
import { getDataSource } from '../../db/data-source';
import { LocalIncomingInspection } from '../../db/entities/LocalIncomingInspection';
import { LocalIncomingInspectionMeasurement } from '../../db/entities/LocalIncomingInspectionMeasurement';

export interface StartInspectionParams {
  lotId: string;
  supplierId: string;
  materialId: string;
  receivedQty: number;
  samplingPlanId: string;
  inspectionLevel: InspectionLevel;
  severityState: SeverityState;
  aqlValue: number;
}

export interface RecordMeasurementParams {
  inspectionId: string;
  sampleNo: number;
  measuredValue: number | null;
  defectFlag: boolean;
  evidenceFileId: string | null;
  recordedBy: string;
}

export class IqcInspectionFlow {
  // 入庫数量・検査水準・AQL からサンプリング計画を解決して検査ヘッダを作成する
  async startInspection(params: StartInspectionParams): Promise<LocalIncomingInspection> {
    const plan = resolveSamplingPlan(
      params.receivedQty,
      params.aqlValue,
      params.inspectionLevel,
      params.severityState,
    );
    const now = new Date().toISOString();
    const entity: LocalIncomingInspection = {
      id: generateId(),
      lotId: params.lotId,
      supplierId: params.supplierId,
      materialId: params.materialId,
      receivedQty: params.receivedQty,
      samplingPlanId: params.samplingPlanId,
      sampleSizeN: plan.sampleSizeN,
      acceptNumberAc: plan.acceptNumberAc,
      rejectNumberRe: plan.rejectNumberRe,
      severityState: plan.severityState,
      qcStatus: 'SAMPLING',
      defectCount: 0,
      inspectedAt: null,
      judgedAt: null,
      judgedBy: null,
      createdAt: now,
    };
    await getDataSource().getRepository(LocalIncomingInspection).save(entity);
    return entity;
  }

  async recordMeasurement(params: RecordMeasurementParams): Promise<void> {
    const entity: LocalIncomingInspectionMeasurement = {
      id: generateId(),
      inspectionId: params.inspectionId,
      sampleNo: params.sampleNo,
      measuredValue: params.measuredValue,
      defectFlag: params.defectFlag,
      evidenceFileId: params.evidenceFileId,
      recordedAt: new Date().toISOString(),
      recordedBy: params.recordedBy,
    };
    await getDataSource().getRepository(LocalIncomingInspectionMeasurement).save(entity);
  }

  // 不良数を集計し Ac/Re 表で AQL 合否を確定する
  async judge(inspectionId: string, judgedBy: string): Promise<AqlVerdict> {
    const inspectionRepo = getDataSource().getRepository(LocalIncomingInspection);
    const measurementRepo = getDataSource().getRepository(LocalIncomingInspectionMeasurement);
    const inspection = await inspectionRepo.findOne({ where: { id: inspectionId } });
    if (inspection === null) {
      throw new Error(`IncomingInspection not found: ${inspectionId}`);
    }
    const measurements = await measurementRepo.find({ where: { inspectionId } });
    const defectCount = measurements.filter((m) => m.defectFlag).length;
    const verdict = judgeAql(defectCount, inspection.acceptNumberAc, inspection.rejectNumberRe);
    const qcStatus = verdict === 'PASSED' ? 'PASSED' : verdict === 'REJECTED' ? 'FAILED' : 'INSPECTING';
    await inspectionRepo.update(
      { id: inspectionId },
      {
        defectCount,
        qcStatus,
        judgedAt: new Date().toISOString(),
        judgedBy,
      },
    );
    return verdict;
  }
}
