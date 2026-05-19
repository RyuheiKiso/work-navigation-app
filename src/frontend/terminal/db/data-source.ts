// 端末ローカル DB は SQLite + TypeORM で永続化し全テーブル合計 53 エンティティを管理する
import 'reflect-metadata';
import { DataSource } from 'typeorm';
import { LocalWorkEvent } from './entities/LocalWorkEvent';
import { LocalElectronicSign } from './entities/LocalElectronicSign';
import { LocalOutboxEvent } from './entities/LocalOutboxEvent';
import { LocalMasterVersion } from './entities/LocalMasterVersion';
import { LocalWorkExecution } from './entities/LocalWorkExecution';
import { LocalWorkOrder } from './entities/LocalWorkOrder';
import { LocalSop } from './entities/LocalSop';
import { LocalStep } from './entities/LocalStep';
import { LocalEvidenceFile } from './entities/LocalEvidenceFile';
import { LocalMeasurement } from './entities/LocalMeasurement';
import { LocalSuspension } from './entities/LocalSuspension';
import { LocalAndonAlert } from './entities/LocalAndonAlert';
import { LocalNonconformity } from './entities/LocalNonconformity';
import { LocalCapa } from './entities/LocalCapa';
import { LocalKaizenProposal } from './entities/LocalKaizenProposal';
import { LocalUser } from './entities/LocalUser';
import { LocalRole } from './entities/LocalRole';
import { LocalSkill } from './entities/LocalSkill';
import { LocalUserRole } from './entities/LocalUserRole';
import { LocalUserSkill } from './entities/LocalUserSkill';
import { LocalProcess } from './entities/LocalProcess';
import { LocalOperation } from './entities/LocalOperation';
import { LocalProduct } from './entities/LocalProduct';
import { LocalLot } from './entities/LocalLot';
import { LocalEquipment } from './entities/LocalEquipment';
import { LocalInstrument } from './entities/LocalInstrument';
import { LocalStepTypeDefinition } from './entities/LocalStepTypeDefinition';
import { LocalStepFlowRule } from './entities/LocalStepFlowRule';
import { LocalHashChainBlock } from './entities/LocalHashChainBlock';
import { LocalAuthLog } from './entities/LocalAuthLog';
import { LocalDevice } from './entities/LocalDevice';
import { LocalDeviceSyncState } from './entities/LocalDeviceSyncState';
import { LocalIdempotencyKey } from './entities/LocalIdempotencyKey';
import { LocalMaterial } from './entities/LocalMaterial';
import { LocalSupplier } from './entities/LocalSupplier';
import { LocalIncomingInspection } from './entities/LocalIncomingInspection';
import { LocalSamplingPlan } from './entities/LocalSamplingPlan';
import { LocalIncomingInspectionMeasurement } from './entities/LocalIncomingInspectionMeasurement';
import { LocalConcessionApproval } from './entities/LocalConcessionApproval';
import { LocalLotQcState } from './entities/LocalLotQcState';
import { LocalRework } from './entities/LocalRework';
import { LocalDisposition } from './entities/LocalDisposition';
import { LocalReworkVerification } from './entities/LocalReworkVerification';
import { LocalReworkSopMapping } from './entities/LocalReworkSopMapping';
import { LocalReworkedLotLabel } from './entities/LocalReworkedLotLabel';
import { LocalReworkCostRecord } from './entities/LocalReworkCostRecord';
import { LocalScrapRecord } from './entities/LocalScrapRecord';
import { LocalReturnToVendorRecord } from './entities/LocalReturnToVendorRecord';
import { LocalCaseLock } from './entities/LocalCaseLock';
import { LocalWorkAssignment } from './entities/LocalWorkAssignment';
import { LocalAppSettings } from './entities/LocalAppSettings';

// 全エンティティ配列。data-source.ts と migrations 双方で同じインスタンスを参照する
export const ALL_ENTITIES = [
  LocalWorkEvent,
  LocalElectronicSign,
  LocalOutboxEvent,
  LocalMasterVersion,
  LocalWorkExecution,
  LocalWorkOrder,
  LocalSop,
  LocalStep,
  LocalEvidenceFile,
  LocalMeasurement,
  LocalSuspension,
  LocalAndonAlert,
  LocalNonconformity,
  LocalCapa,
  LocalKaizenProposal,
  LocalUser,
  LocalRole,
  LocalSkill,
  LocalUserRole,
  LocalUserSkill,
  LocalProcess,
  LocalOperation,
  LocalProduct,
  LocalLot,
  LocalEquipment,
  LocalInstrument,
  LocalStepTypeDefinition,
  LocalStepFlowRule,
  LocalHashChainBlock,
  LocalAuthLog,
  LocalDevice,
  LocalDeviceSyncState,
  LocalIdempotencyKey,
  LocalMaterial,
  LocalSupplier,
  LocalIncomingInspection,
  LocalSamplingPlan,
  LocalIncomingInspectionMeasurement,
  LocalConcessionApproval,
  LocalLotQcState,
  LocalRework,
  LocalDisposition,
  LocalReworkVerification,
  LocalReworkSopMapping,
  LocalReworkedLotLabel,
  LocalReworkCostRecord,
  LocalScrapRecord,
  LocalReturnToVendorRecord,
  LocalCaseLock,
  LocalWorkAssignment,
  LocalAppSettings,
];

let dataSource: DataSource | null = null;

// 端末初回起動時に DataSource を初期化し、その後はキャッシュした単一インスタンスを返す
export async function initDatabase(): Promise<DataSource> {
  if (dataSource !== null && dataSource.isInitialized) return dataSource;
  dataSource = new DataSource({
    type: 'expo',
    driver: require('expo-sqlite'),
    database: 'wnav.db',
    entities: ALL_ENTITIES,
    migrations: [],
    // synchronize は本番安全性のため必ず false。スキーマ変更は migration ファイルで管理する
    synchronize: false,
    migrationsRun: false,
    logging: false,
  });
  await dataSource.initialize();
  return dataSource;
}

// 初期化済み DataSource を取得する。未初期化時は明示的に失敗させる
export function getDataSource(): DataSource {
  if (dataSource === null || !dataSource.isInitialized) {
    throw new Error('DataSource 未初期化。initDatabase() を先に呼んでください');
  }
  return dataSource;
}

// テスト・ログアウトリセット用にコネクションを切断する
export async function closeDatabase(): Promise<void> {
  if (dataSource !== null && dataSource.isInitialized) {
    await dataSource.destroy();
    dataSource = null;
  }
}
