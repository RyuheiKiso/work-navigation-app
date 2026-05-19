// XES 互換イベント種別は ProM/Disco でのプロセスマイニング解析を前提とするため列挙型として固定する
export type ActivityType =
  | 'step_completed'
  | 'step_skipped'
  | 'evidence_attached'
  | 'sign_applied'
  | 'measurement_recorded'
  | 'work_started'
  | 'work_completed'
  | 'suspended'
  | 'resumed'
  | 'andon_raised'
  | 'andon_acknowledged'
  | 'andon_resolved'
  | 'nonconformity_registered'
  | 'evidence_uploaded'
  | 'worker_delegated'
  | 'iqc_received'
  | 'iqc_measured'
  | 'iqc_passed'
  | 'iqc_failed'
  | 'iqc_conditional_pass'
  | 'iqc_concession_approved'
  | 'rework_started'
  | 'rework_completed'
  | 'rework_verified'
  | 'disposition_approved'
  | 'scrap_recorded'
  | 'return_recorded';

// RBAC 6 ロールは docs/02_企画/システム化計画/15_セキュリティ深堀り.md の正規体系
export type UserRole =
  | 'operator'
  | 'supervisor'
  | 'quality_admin'
  | 'master_admin'
  | 'system_admin'
  | 'executive';

export type NetworkQuality = 'high' | 'low' | 'disconnected' | 'emergency';

export type QcStatus =
  | 'PENDING'
  | 'SAMPLING'
  | 'INSPECTING'
  | 'PASSED'
  | 'FAILED'
  | 'REJECTED'
  | 'CONDITIONAL_PASS'
  | 'SCREENING_REQUIRED'
  | 'SCRAPPED'
  | 'RETURNED_TO_VENDOR';

export type SopStatus = 'draft' | 'in_review' | 'published' | 'deprecated';
export type SopType = 'STANDARD' | 'REWORK';

export type WorkOrderStatus = 'open' | 'in_progress' | 'completed' | 'cancelled';

export type WorkExecutionStatus = 'in_progress' | 'suspended' | 'completed' | 'cancelled';

export type ReworkStatus =
  | 'OPEN'
  | 'IN_PROGRESS'
  | 'PENDING_VERIFICATION'
  | 'CLOSED'
  | 'SCRAPPED'
  | 'RETURNED';

export type DispositionType = 'REWORK' | 'SCRAP' | 'RETURN_TO_VENDOR' | 'USE_AS_IS';

export type AlertType = 'quality' | 'safety' | 'equipment' | 'process';
export type AlertSeverity = 'low' | 'medium' | 'high' | 'critical';
export type AlertStatus = 'open' | 'acknowledged' | 'resolved';

export type WorkAssignmentStatus = 'pending' | 'dispatched' | 'acknowledged' | 'cancelled';

export type CapaStatus = 'open' | 'in_progress' | 'pending_verification' | 'closed';

export type SeverityState = 'NORMAL' | 'TIGHTENED' | 'REDUCED';

export type Locale = 'ja' | 'en' | 'zh';

// JSONB 多言語テキストは {ja,en,zh} のキーを必須とする（docs/02 §6 データモデル中核設計）
export interface LocalizedText {
  ja: string;
  en: string;
  zh: string;
}

// XES 互換イベント構造（NULL 禁止フィールドは src/CLAUDE.md §XES 互換イベント必須属性に従う）
export interface WorkEvent {
  eventId: string;
  caseId: string;
  activity: ActivityType;
  timestampClient: string;
  timestampServer: string | null;
  resource: string;
  sopVersionId: string;
  stepId: string;
  payload: string;
  prevHash: string;
  contentHash: string;
  terminalId: string;
  synced: boolean;
}

// TBL-004 マスタバージョン
export interface MasterVersion {
  id: string;
  sopId: string;
  entityType: string;
  entityId: string;
  version: string;
  status: SopStatus;
  changeSummary: string;
  stepCount: number;
  createdAt: string;
  createdBy: string;
  submittedAt: string | null;
  submittedBy: string | null;
  approvedBy: string | null;
  approvedAt: string | null;
  publishedAt: string | null;
  publishedBy: string | null;
  deprecatedAt: string | null;
  deletedAt: string | null;
}

// TBL-006 作業指示
export interface WorkOrder {
  id: string;
  workOrderNumber: string;
  productId: string;
  sopId: string;
  sopVersionId: string;
  processId: string;
  lotId: string | null;
  scheduledStart: string;
  scheduledEnd: string | null;
  status: WorkOrderStatus;
  assignedTo: string | null;
  createdAt: string;
  updatedAt: string;
  deletedAt: string | null;
}

// TBL-005 作業実行
export interface WorkExecution {
  id: string;
  workOrderId: string;
  operatorId: string;
  deviceId: string;
  status: WorkExecutionStatus;
  currentStepId: string | null;
  completedStepCount: number;
  totalStepCount: number;
  sopVersionSnapshot: {
    sopId: string;
    version: string;
    snapshotHash: string;
  };
  startedAt: string;
  lastEventAt: string;
  completedAt: string | null;
  createdAt: string;
}

// TBL-007 SOP
export interface Sop {
  id: string;
  sopCode: string;
  nameJson: LocalizedText;
  descriptionJson: LocalizedText;
  sopType: SopType;
  processId: string;
  operationId: string;
  currentVersionId: string | null;
  deletedAt: string | null;
}

// TBL-008 Step
export interface Step {
  id: string;
  sopVersionId: string;
  stepNumber: number;
  stepType: 'check' | 'operation' | 'measurement' | 'sign' | 'photo' | 'standard' | 'branching' | 'custom' | 'signature';
  titleJson: LocalizedText;
  instructionJson: LocalizedText;
  payload: string;
  isMandatory: boolean;
  requiresEvidence: boolean;
  requiresSign: boolean;
  skillLevelRequired: number;
  estimatedSeconds: number;
  fallbackType: 'skip' | 'manual' | 'halt';
  flowRules: {
    onComplete: string;
    onSkip: string;
  };
  deletedAt: string | null;
}

// TBL-009 エビデンス
export interface EvidenceFile {
  id: string;
  workExecutionId: string;
  stepId: string;
  evidenceType: 'photo' | 'document' | 'measurement_sheet';
  filePath: string;
  fileHashSha256: string;
  fileSizeBytes: number;
  widthPx: number | null;
  heightPx: number | null;
  description: string;
  uploadedBy: string;
  uploadedAt: string;
}

// TBL-002 電子サイン
export interface ElectronicSign {
  id: string;
  signerId: string;
  signedContentHash: string;
  contextType: 'step_sign' | 'work_complete_sign' | 'approval_sign' | 'quality_check_sign';
  contextId: string;
  stepId: string | null;
  signedAt: string;
  hashChainBlockId: string;
  hashChainValue: string;
  hashChainPrev: string;
  deviceId: string;
}

// TBL-016 ユーザー
export interface User {
  id: string;
  loginId: string;
  username: string;
  displayNameJson: LocalizedText;
  email: string | null;
  role: UserRole;
  roles: UserRole[];
  factoryId: string;
  locale: Locale;
  isActive: boolean;
  createdAt: string;
  deletedAt: string | null;
}

// TBL-018 スキル
export interface Skill {
  id: string;
  skillCode: string;
  nameJson: LocalizedText;
  level: number;
  deletedAt: string | null;
}

// TBL-021 プロセス
export interface Process {
  id: string;
  processCode: string;
  nameJson: LocalizedText;
  descriptionJson: LocalizedText;
  isActive: boolean;
  deletedAt: string | null;
}

// TBL-022 オペレーション
export interface Operation {
  id: string;
  operationCode: string;
  nameJson: LocalizedText;
  processId: string;
  deletedAt: string | null;
}

// TBL-023 製品
export interface Product {
  id: string;
  productCode: string;
  nameJson: LocalizedText;
  deletedAt: string | null;
}

// TBL-024 ロット
export interface Lot {
  id: string;
  lotNumber: string;
  productId: string;
  externalKey: string | null;
  deletedAt: string | null;
}

// TBL-011 中断
export interface Suspension {
  id: string;
  workExecutionId: string;
  reasonCode: 'equipment_breakdown' | 'material_shortage' | 'quality_issue' | 'emergency' | 'other';
  reasonDetail: string;
  suspendedAt: string;
  resumedAt: string | null;
  resumedBy: string | null;
}

// TBL-012 アンドンアラート
export interface AndonAlert {
  id: string;
  alertType: AlertType;
  severity: AlertSeverity;
  status: AlertStatus;
  workExecutionId: string | null;
  stepId: string | null;
  raisedBy: string;
  title: string;
  description: string;
  raisedAt: string;
  acknowledgedBy: string | null;
  acknowledgedAt: string | null;
  resolvedBy: string | null;
  resolvedAt: string | null;
  resolutionNote: string | null;
}

// TBL-013 非適合品
export interface Nonconformity {
  id: string;
  alertId: string | null;
  workExecutionId: string | null;
  lotId: string | null;
  ncType: 'process_deviation' | 'material_defect' | 'measurement_out_of_spec' | 'document_error';
  description: string;
  discoveredBy: string;
  discoveryStepId: string | null;
  evidenceIds: string[];
  createdAt: string;
}

// TBL-014 CAPA
export interface Capa {
  id: string;
  nonconformityId: string | null;
  title: string;
  status: CapaStatus;
  rootCauseAnalysis: string;
  correctiveAction: string;
  preventiveAction: string | null;
  assignedTo: string;
  dueDate: string;
  createdBy: string;
  createdAt: string;
  progressNote: string | null;
  closedAt: string | null;
  closedBy: string | null;
}

// TBL-015 改善提案
export interface KaizenProposal {
  id: string;
  proposerId: string;
  processId: string | null;
  category: 'efficiency' | 'safety' | 'quality' | 'cost' | 'environment';
  title: string;
  currentSituation: string;
  proposalDetail: string;
  expectedBenefit: string | null;
  relatedSopId: string | null;
  evidenceIds: string[];
  status: 'submitted' | 'reviewing' | 'approved' | 'rejected' | 'implemented';
  createdAt: string;
}

// TBL-036 材料
export interface Material {
  id: string;
  materialCode: string;
  nameJson: LocalizedText;
  materialType: string;
  unit: string;
  deletedAt: string | null;
}

// TBL-037 仕入先
export interface Supplier {
  id: string;
  supplierCode: string;
  nameJson: LocalizedText;
  contactEmail: string | null;
  deletedAt: string | null;
}

// TBL-038 受入検査
export interface IncomingInspection {
  id: string;
  lotId: string;
  supplierId: string;
  materialId: string;
  receivedQty: number;
  samplingPlanId: string;
  sampleSizeN: number;
  acceptNumberAc: number;
  rejectNumberRe: number;
  severityState: SeverityState;
  qcStatus: QcStatus;
  defectCount: number;
  inspectedAt: string | null;
  judgedAt: string | null;
  judgedBy: string | null;
  createdAt: string;
}

export interface IncomingInspectionMeasurement {
  id: string;
  inspectionId: string;
  sampleNo: number;
  measuredValue: number | null;
  defectFlag: boolean;
  evidenceFileId: string | null;
  recordedAt: string;
  recordedBy: string;
}

// TBL-039 サンプリング計画
export interface SamplingPlan {
  id: string;
  planCode: string;
  nameJson: LocalizedText;
  aqlValue: number;
  inspectionLevel: 'I' | 'II' | 'III';
  planSnapshot: string;
  deletedAt: string | null;
}

// TBL-041 特採承認
export interface ConcessionApproval {
  id: string;
  incomingInspectionId: string;
  requestedBy: string;
  approvedBy: string | null;
  approvalSign: string | null;
  electronicSignId: string | null;
  conditionNote: string;
  validityScope: string;
  validUntil: string | null;
  status: 'PENDING' | 'APPROVED' | 'REJECTED';
  createdAt: string;
}

// TBL-040 ロット QC 状態
export interface LotQcState {
  lotId: string;
  qcStatus: QcStatus;
  concessionApprovalId: string | null;
  validUntil: string | null;
  updatedAt: string;
}

// TBL-043 リワーク
export interface Rework {
  id: string;
  parentCaseId: string;
  reworkCaseId: string;
  nonconformityId: string;
  sopId: string;
  reworkSopVersionId: string;
  assignedTo: string | null;
  status: ReworkStatus;
  reworkCount: number;
  deadline: string | null;
  createdAt: string;
}

// TBL-044 ディスポジション
export interface Disposition {
  id: string;
  nonconformityId: string;
  dispositionType: DispositionType;
  decision: 'REWORK' | 'SCRAP' | 'RETURN' | 'USE_AS_IS';
  decisionReason: string;
  qualityAdminSignId: string | null;
  supervisorSignId: string | null;
  signedAt: string | null;
  createdAt: string;
}

// TBL-045 再検査
export interface ReworkVerification {
  id: string;
  reworkId: string;
  verifierId: string;
  verifiedAt: string;
  passed: boolean;
  note: string;
  evidenceIds: string[];
}

// TBL-046 廃却記録
export interface ScrapRecord {
  id: string;
  nonconformityId: string;
  scrappedBy: string;
  witnessId: string;
  scrappedAt: string;
  quantity: number;
  note: string;
}

// TBL-047 返却記録
export interface ReturnRecord {
  id: string;
  nonconformityId: string;
  supplierId: string;
  trackingNo: string;
  returnedBy: string;
  returnedAt: string;
  quantity: number;
}

// TBL-052/053 作業指示割当
export interface WorkAssignment {
  id: string;
  externalOrderId: string;
  externalSystem: string;
  sopId: string;
  sopName: string;
  targetTerminalId: string;
  lotId: string | null;
  lotNumber: string | null;
  suggestedWorkerId: string | null;
  suggestedEquipmentId: string | null;
  dueAt: string | null;
  priority: number;
  status: WorkAssignmentStatus;
  receivedAt: string;
  acknowledgedAt: string | null;
  cancelledAt: string | null;
}

// TBL-031 ハッシュチェーンブロック
export interface HashChainBlock {
  id: string;
  blockNumber: number;
  prevHash: string;
  contentHash: string;
  payload: string;
  createdAt: string;
}

// TBL-003 Outbox イベント
export interface OutboxEvent {
  id: string;
  eventType: 'work_event' | 'electronic_sign' | 'webhook_audit_event';
  payload: string;
  status: 'pending' | 'dispatched' | 'failed' | 'dlq';
  retryCount: number;
  lastError: string | null;
  firstFailedAt: string | null;
  lastFailedAt: string | null;
  createdAt: string;
}

// TBL-033 端末
export interface HandyTerminal {
  id: string;
  terminalCode: string;
  externalKey: string | null;
  factoryId: string;
  isActive: boolean;
  deletedAt: string | null;
}

// 認証関連
export interface AuthLoginRequest {
  loginId: string;
  password: string;
  deviceId: string;
  factoryId: string;
}

// API リクエスト関連の補助型（OpenAPI から派生する snake_case フィールドを直接扱えるようにする）
export interface CreateWorkExecutionRequest {
  work_order_id: string;
  operator_id: string;
  device_id: string;
  start_timestamp_client: string;
}

export interface SuspendWorkExecutionRequest {
  reason_code: 'equipment_breakdown' | 'material_shortage' | 'quality_issue' | 'emergency' | 'other';
  reason_detail?: string;
  timestamp_client: string;
}

export interface ResumeWorkExecutionRequest {
  resumed_by: string;
  timestamp_client: string;
}

export interface CompleteWorkExecutionRequest {
  completed_by: string;
  timestamp_client: string;
  final_remarks?: string;
}

export interface AuthLoginResponse {
  accessToken: string;
  refreshToken: string;
  tokenType: 'Bearer';
  expiresIn: number;
  refreshExpiresIn: number;
  roles: UserRole[];
  userId: string;
  factoryId: string;
}

export interface AuthRefreshRequest {
  refreshToken: string;
}

export interface AuthRefreshResponse {
  accessToken: string;
  tokenType: 'Bearer';
  expiresIn: number;
}

export interface Jwks {
  keys: Array<{
    kty: 'RSA';
    use: 'sig';
    kid: string;
    alg: 'RS256';
    n: string;
    e: string;
  }>;
}

// レスポンスエンベロープ型（docs/05_詳細設計/03_API詳細設計/01_OpenAPI共通仕様.md §3-1）
export interface ResponseMeta {
  request_id: string;
  server_time: string;
  api_version: 'v1';
}

export interface ApiResponse<T> {
  data: T;
  meta: ResponseMeta;
}

export interface PageMeta {
  total: number;
  page: number;
  per_page: number;
  total_pages: number;
}

export interface PaginatedResponse<T> {
  data: T[];
  meta: ResponseMeta & PageMeta;
}

export interface CursorMeta {
  limit: number;
  has_more: boolean;
  next_cursor: string | null;
}

export interface CursorPaginatedResponse<T> {
  data: T[];
  meta: ResponseMeta & CursorMeta;
}

// ProblemDetails（RFC 9457、docs/05_詳細設計/03_API詳細設計/01_OpenAPI共通仕様.md §3-2）
export interface ProblemDetails {
  type: string;
  title: string;
  status: number;
  detail: string;
  instance?: string;
  error_id?: string;
  violations?: Array<{
    field: string;
    message: string;
    value?: unknown;
    min?: number;
    max?: number;
    actual?: number;
  }>;
}

export type ErrorCode =
  | 'ERR-AUTH-001'
  | 'ERR-AUTH-002'
  | 'ERR-AUTH-003'
  | 'ERR-AUTH-004'
  | 'ERR-VAL-001'
  | 'ERR-VAL-002'
  | 'ERR-VAL-003'
  | 'ERR-VAL-004'
  | 'ERR-VAL-027'
  | 'ERR-VAL-028'
  | 'ERR-VAL-029'
  | 'ERR-VAL-030'
  | 'ERR-VAL-031'
  | 'ERR-VAL-032'
  | 'ERR-BIZ-001'
  | 'ERR-BIZ-002'
  | 'ERR-BIZ-003'
  | 'ERR-BIZ-004'
  | 'ERR-BIZ-005'
  | 'ERR-BIZ-006'
  | 'ERR-BIZ-007'
  | 'ERR-BIZ-008'
  | 'ERR-BIZ-015'
  | 'ERR-BIZ-016'
  | 'ERR-BIZ-017'
  | 'ERR-BIZ-018'
  | 'ERR-BIZ-019'
  | 'ERR-BIZ-020'
  | 'ERR-BIZ-021'
  | 'ERR-BIZ-022'
  | 'ERR-BIZ-023'
  | 'ERR-BIZ-024'
  | 'ERR-BIZ-025'
  | 'ERR-BIZ-026'
  | 'ERR-BIZ-027'
  | 'ERR-DB-001'
  | 'ERR-DB-002'
  | 'ERR-DB-003'
  | 'ERR-DB-004'
  | 'ERR-EXT-001'
  | 'ERR-EXT-002'
  | 'ERR-SYS-001'
  | 'ERR-SYS-002'
  | 'ERR-SYS-003'
  | 'ERR-SYS-004'
  | 'ERR-SYS-005';
