import { v7 as uuidv7 } from 'uuid';
import type {
  AndonAlert,
  Capa,
  ConcessionApproval,
  Disposition,
  ElectronicSign,
  EvidenceFile,
  HandyTerminal,
  HashChainBlock,
  IncomingInspection,
  IncomingInspectionMeasurement,
  KaizenProposal,
  LotQcState,
  Lot,
  Material,
  MasterVersion,
  Nonconformity,
  Operation,
  OutboxEvent,
  Process,
  Product,
  Rework,
  ReworkVerification,
  ReturnRecord,
  SamplingPlan,
  ScrapRecord,
  Skill,
  Sop,
  Step,
  Supplier,
  Suspension,
  User,
  WorkAssignment,
  WorkEvent,
  WorkExecution,
  WorkOrder,
} from '../../types';

// 全シードオブジェクトを単一インメモリストアで管理する（プロセス間共有なし）
export interface MockDatabase {
  users: User[];
  terminals: HandyTerminal[];
  skills: Skill[];
  processes: Process[];
  operations: Operation[];
  products: Product[];
  lots: Lot[];
  sops: Sop[];
  sopVersions: MasterVersion[];
  steps: Step[];
  workOrders: WorkOrder[];
  workExecutions: WorkExecution[];
  workEvents: WorkEvent[];
  outboxEvents: OutboxEvent[];
  electronicSigns: ElectronicSign[];
  evidenceFiles: EvidenceFile[];
  suspensions: Suspension[];
  andonAlerts: AndonAlert[];
  nonconformities: Nonconformity[];
  capas: Capa[];
  kaizenProposals: KaizenProposal[];
  materials: Material[];
  suppliers: Supplier[];
  samplingPlans: SamplingPlan[];
  incomingInspections: IncomingInspection[];
  incomingInspectionMeasurements: IncomingInspectionMeasurement[];
  concessionApprovals: ConcessionApproval[];
  lotQcStates: LotQcState[];
  reworks: Rework[];
  dispositions: Disposition[];
  reworkVerifications: ReworkVerification[];
  scrapRecords: ScrapRecord[];
  returnRecords: ReturnRecord[];
  workAssignments: WorkAssignment[];
  hashChainBlocks: HashChainBlock[];
  refreshTokens: Map<string, { userId: string; expiresAt: number }>;
  idempotencyKeys: Map<string, { response: unknown; status: number; expiresAt: number; bodyHash: string }>;
  jtiBlacklist: Set<string>;
}

const factoryId = '019682ab-7c1f-7000-0000-000000000001';

function loc(ja: string, en: string, zh: string) {
  return { ja, en, zh };
}

function createInitialDb(): MockDatabase {
  const operatorId = uuidv7();
  const supervisorId = uuidv7();
  const qualityAdminId = uuidv7();
  const masterAdminId = uuidv7();
  const systemAdminId = uuidv7();
  const executiveId = uuidv7();
  const terminalId = uuidv7();
  const processId = uuidv7();
  const operationId = uuidv7();
  const productId = uuidv7();
  const lotId = uuidv7();
  const sopId = uuidv7();
  const sopVersionId = uuidv7();
  const stepId1 = uuidv7();
  const stepId2 = uuidv7();
  const workOrderId = uuidv7();
  const materialId = uuidv7();
  const supplierId = uuidv7();
  const samplingPlanId = uuidv7();
  const skillId = uuidv7();
  const now = new Date().toISOString();

  return {
    users: [
      {
        id: operatorId,
        loginId: 'operator01',
        username: 'operator01',
        displayNameJson: loc('作業者1', 'Operator 1', '操作员1'),
        email: null,
        role: 'operator',
        roles: ['operator'],
        factoryId,
        locale: 'ja',
        isActive: true,
        createdAt: now,
        deletedAt: null,
      },
      {
        id: supervisorId,
        loginId: 'supervisor01',
        username: 'supervisor01',
        displayNameJson: loc('監督者1', 'Supervisor 1', '主管1'),
        email: null,
        role: 'supervisor',
        roles: ['supervisor'],
        factoryId,
        locale: 'ja',
        isActive: true,
        createdAt: now,
        deletedAt: null,
      },
      {
        id: qualityAdminId,
        loginId: 'quality01',
        username: 'quality01',
        displayNameJson: loc('品質管理者', 'Quality Admin', '质量管理员'),
        email: null,
        role: 'quality_admin',
        roles: ['quality_admin'],
        factoryId,
        locale: 'ja',
        isActive: true,
        createdAt: now,
        deletedAt: null,
      },
      {
        id: masterAdminId,
        loginId: 'masteradmin',
        username: 'masteradmin',
        displayNameJson: loc('マスタ管理者', 'Master Admin', '主数据管理员'),
        email: null,
        role: 'master_admin',
        roles: ['master_admin'],
        factoryId,
        locale: 'ja',
        isActive: true,
        createdAt: now,
        deletedAt: null,
      },
      {
        id: systemAdminId,
        loginId: 'sysadmin',
        username: 'sysadmin',
        displayNameJson: loc('システム管理者', 'System Admin', '系统管理员'),
        email: null,
        role: 'system_admin',
        roles: ['system_admin'],
        factoryId,
        locale: 'ja',
        isActive: true,
        createdAt: now,
        deletedAt: null,
      },
      {
        id: executiveId,
        loginId: 'executive',
        username: 'executive',
        displayNameJson: loc('経営層', 'Executive', '高管'),
        email: null,
        role: 'executive',
        roles: ['executive'],
        factoryId,
        locale: 'ja',
        isActive: true,
        createdAt: now,
        deletedAt: null,
      },
    ],
    terminals: [
      {
        id: terminalId,
        terminalCode: 'TERMINAL-LINE1-01',
        externalKey: 'TERMINAL-LINE1-01',
        factoryId,
        isActive: true,
        deletedAt: null,
      },
    ],
    skills: [
      {
        id: skillId,
        skillCode: 'WELDING-BASIC',
        nameJson: loc('溶接基礎', 'Welding Basic', '焊接基础'),
        level: 1,
        deletedAt: null,
      },
    ],
    processes: [
      {
        id: processId,
        processCode: 'WELDING_A',
        nameJson: loc('溶接工程A', 'Welding Process A', '焊接工序A'),
        descriptionJson: loc('溶接工程の標準手順', 'Standard welding procedure', '焊接标准程序'),
        isActive: true,
        deletedAt: null,
      },
    ],
    operations: [
      {
        id: operationId,
        operationCode: 'OP-WELD-001',
        nameJson: loc('溶接操作', 'Welding Operation', '焊接操作'),
        processId,
        deletedAt: null,
      },
    ],
    products: [
      {
        id: productId,
        productCode: 'PRD-001',
        nameJson: loc('部品A', 'Part A', '零件A'),
        deletedAt: null,
      },
    ],
    lots: [
      {
        id: lotId,
        lotNumber: 'LOT-2026-0001',
        productId,
        externalKey: null,
        deletedAt: null,
      },
    ],
    sops: [
      {
        id: sopId,
        sopCode: 'SOP-WELD-A',
        nameJson: loc('溶接工程A 標準手順書', 'Welding SOP A', '焊接SOP A'),
        descriptionJson: loc('標準手順書', 'Standard procedure', '标准程序'),
        sopType: 'STANDARD',
        processId,
        operationId,
        currentVersionId: sopVersionId,
        deletedAt: null,
      },
    ],
    sopVersions: [
      {
        id: sopVersionId,
        sopId,
        entityType: 'sop',
        entityId: sopId,
        version: '1.0.0',
        status: 'published',
        changeSummary: 'Initial publication',
        stepCount: 2,
        createdAt: now,
        createdBy: masterAdminId,
        submittedAt: now,
        submittedBy: masterAdminId,
        approvedBy: qualityAdminId,
        approvedAt: now,
        publishedAt: now,
        publishedBy: qualityAdminId,
        deprecatedAt: null,
        deletedAt: null,
      },
    ],
    steps: [
      {
        id: stepId1,
        sopVersionId,
        stepNumber: 1,
        stepType: 'check',
        titleJson: loc('安全確認', 'Safety Check', '安全确认'),
        instructionJson: loc('作業開始前に安全装備を確認する', 'Verify safety equipment', '检查安全装备'),
        payload: JSON.stringify({ inputType: 'boolean_check' }),
        isMandatory: true,
        requiresEvidence: false,
        requiresSign: false,
        skillLevelRequired: 1,
        estimatedSeconds: 60,
        fallbackType: 'halt',
        flowRules: { onComplete: 'next', onSkip: 'supervisor_approval_required' },
        deletedAt: null,
      },
      {
        id: stepId2,
        sopVersionId,
        stepNumber: 2,
        stepType: 'measurement',
        titleJson: loc('トルク測定', 'Torque Measurement', '扭矩测量'),
        instructionJson: loc('トルクレンチで 25Nm を測定する', 'Measure 25Nm torque', '测量25Nm扭矩'),
        payload: JSON.stringify({ inputType: 'numeric_input', usl: 27, lsl: 23 }),
        isMandatory: true,
        requiresEvidence: true,
        requiresSign: true,
        skillLevelRequired: 2,
        estimatedSeconds: 120,
        fallbackType: 'manual',
        flowRules: { onComplete: 'next', onSkip: 'forbidden' },
        deletedAt: null,
      },
    ],
    workOrders: [
      {
        id: workOrderId,
        workOrderNumber: 'WO-2026-0001',
        productId,
        sopId,
        sopVersionId,
        processId,
        lotId,
        scheduledStart: now,
        scheduledEnd: null,
        status: 'open',
        assignedTo: operatorId,
        createdAt: now,
        updatedAt: now,
        deletedAt: null,
      },
    ],
    materials: [
      {
        id: materialId,
        materialCode: 'MAT-001',
        nameJson: loc('鋼材A', 'Steel A', '钢材A'),
        materialType: 'metal',
        unit: 'kg',
        deletedAt: null,
      },
    ],
    suppliers: [
      {
        id: supplierId,
        supplierCode: 'SUP-001',
        nameJson: loc('仕入先A', 'Supplier A', '供应商A'),
        contactEmail: 'supplier-a@example.com',
        deletedAt: null,
      },
    ],
    samplingPlans: [
      {
        id: samplingPlanId,
        planCode: 'PLAN-AQL-1.0-II',
        nameJson: loc('なみ検査 AQL1.0 Level II', 'Normal AQL 1.0 Level II', '正常AQL1.0 Level II'),
        aqlValue: 1.0,
        inspectionLevel: 'II',
        planSnapshot: JSON.stringify({ severity: 'NORMAL', aql: 1.0, level: 'II' }),
        deletedAt: null,
      },
    ],
    workExecutions: [],
    workEvents: [],
    outboxEvents: [],
    electronicSigns: [],
    evidenceFiles: [],
    suspensions: [],
    andonAlerts: [],
    nonconformities: [],
    capas: [],
    kaizenProposals: [],
    incomingInspections: [],
    incomingInspectionMeasurements: [],
    concessionApprovals: [],
    lotQcStates: [],
    reworks: [],
    dispositions: [],
    reworkVerifications: [],
    scrapRecords: [],
    returnRecords: [],
    workAssignments: [],
    hashChainBlocks: [],
    refreshTokens: new Map(),
    idempotencyKeys: new Map(),
    jtiBlacklist: new Set(),
  };
}

export const db: MockDatabase = createInitialDb();

// テスト間の独立性を保つために mock db を初期状態へ完全に巻き戻す
export function resetDb(): void {
  const fresh = createInitialDb();
  for (const key of Object.keys(db) as Array<keyof MockDatabase>) {
    const next = fresh[key];
    (db as unknown as Record<string, unknown>)[key as string] = next;
  }
}
