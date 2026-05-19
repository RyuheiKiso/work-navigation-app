// 端末初回起動時に全 51 エンティティに対応する CREATE TABLE を発行する初期マイグレーション
// down() は端末では migration:revert 非サポートのため意図的に空のままとする（07a §4）
import type { MigrationInterface, QueryRunner } from 'typeorm';

export class InitialSchema1716034496000 implements MigrationInterface {
  name = 'InitialSchema1716034496000';

  async up(queryRunner: QueryRunner): Promise<void> {
    // work_events: Append-only、prevHash/contentHash で改竄検知チェーンを構成
    await queryRunner.query(`
      CREATE TABLE IF NOT EXISTS "work_events" (
        "eventId" text PRIMARY KEY NOT NULL,
        "caseId" text NOT NULL,
        "activity" text NOT NULL,
        "timestampClient" text NOT NULL,
        "resource" text NOT NULL,
        "sopVersionId" text NOT NULL,
        "stepId" text NOT NULL,
        "payload" text NOT NULL,
        "prevHash" text NOT NULL,
        "contentHash" text NOT NULL,
        "terminalId" text NOT NULL,
        "synced" integer NOT NULL DEFAULT 0
      )
    `);
    await queryRunner.query(`CREATE INDEX IF NOT EXISTS "idx_work_events_caseid" ON "work_events" ("caseId")`);
    await queryRunner.query(
      `CREATE INDEX IF NOT EXISTS "idx_work_events_caseid_stepid" ON "work_events" ("caseId", "stepId")`,
    );
    await queryRunner.query(`CREATE INDEX IF NOT EXISTS "idx_work_events_synced" ON "work_events" ("synced")`);

    // electronic_signs: 電子サイン、Ed25519 署名値を hex で格納
    await queryRunner.query(`
      CREATE TABLE IF NOT EXISTS "electronic_signs" (
        "id" text PRIMARY KEY NOT NULL,
        "signerId" text NOT NULL,
        "signedContentHash" text NOT NULL,
        "contextType" text NOT NULL,
        "contextId" text NOT NULL,
        "stepId" text,
        "signedAt" text NOT NULL,
        "hashChainBlockId" text NOT NULL,
        "hashChainValue" text NOT NULL,
        "hashChainPrev" text NOT NULL,
        "deviceId" text NOT NULL,
        "synced" integer NOT NULL DEFAULT 0
      )
    `);
    await queryRunner.query(
      `CREATE INDEX IF NOT EXISTS "idx_electronic_signs_ctx" ON "electronic_signs" ("contextType", "contextId")`,
    );

    // outbox_events: 送信キュー、Auto-increment rowid を id とする
    await queryRunner.query(`
      CREATE TABLE IF NOT EXISTS "outbox_events" (
        "id" integer PRIMARY KEY AUTOINCREMENT NOT NULL,
        "eventId" text NOT NULL,
        "idempotencyKey" text NOT NULL,
        "payload" text NOT NULL,
        "prevHash" text NOT NULL,
        "createdAt" text NOT NULL,
        "sent" integer NOT NULL DEFAULT 0,
        "retryCount" integer NOT NULL DEFAULT 0,
        "lastError" text,
        "nextRetryAt" text NOT NULL
      )
    `);
    await queryRunner.query(
      `CREATE INDEX IF NOT EXISTS "idx_outbox_sent_created" ON "outbox_events" ("sent", "createdAt")`,
    );

    // master_versions
    await queryRunner.query(`
      CREATE TABLE IF NOT EXISTS "master_versions" (
        "id" text PRIMARY KEY NOT NULL,
        "sopId" text NOT NULL,
        "entityType" text NOT NULL,
        "entityId" text NOT NULL,
        "version" text NOT NULL,
        "status" text NOT NULL,
        "changeSummary" text NOT NULL,
        "stepCount" integer NOT NULL DEFAULT 0,
        "createdAt" text NOT NULL,
        "createdBy" text NOT NULL,
        "submittedAt" text,
        "submittedBy" text,
        "approvedBy" text,
        "approvedAt" text,
        "publishedAt" text,
        "publishedBy" text,
        "deprecatedAt" text,
        "deletedAt" text
      )
    `);

    // work_executions
    await queryRunner.query(`
      CREATE TABLE IF NOT EXISTS "work_executions" (
        "id" text PRIMARY KEY NOT NULL,
        "workOrderId" text NOT NULL,
        "operatorId" text NOT NULL,
        "deviceId" text NOT NULL,
        "status" text NOT NULL,
        "currentStepId" text,
        "completedStepCount" integer NOT NULL DEFAULT 0,
        "totalStepCount" integer NOT NULL DEFAULT 0,
        "sopVersionSnapshot" text NOT NULL,
        "startedAt" text NOT NULL,
        "lastEventAt" text NOT NULL,
        "completedAt" text,
        "createdAt" text NOT NULL
      )
    `);
    await queryRunner.query(`CREATE INDEX IF NOT EXISTS "idx_we_status" ON "work_executions" ("status")`);
    await queryRunner.query(`CREATE INDEX IF NOT EXISTS "idx_we_operator" ON "work_executions" ("operatorId")`);

    // work_orders
    await queryRunner.query(`
      CREATE TABLE IF NOT EXISTS "work_orders" (
        "id" text PRIMARY KEY NOT NULL,
        "workOrderNumber" text NOT NULL,
        "productId" text NOT NULL,
        "sopId" text NOT NULL,
        "sopVersionId" text NOT NULL,
        "processId" text NOT NULL,
        "lotId" text,
        "scheduledStart" text NOT NULL,
        "scheduledEnd" text,
        "status" text NOT NULL,
        "assignedTo" text,
        "createdAt" text NOT NULL,
        "updatedAt" text NOT NULL,
        "deletedAt" text
      )
    `);

    // sops
    await queryRunner.query(`
      CREATE TABLE IF NOT EXISTS "sops" (
        "id" text PRIMARY KEY NOT NULL,
        "sopCode" text NOT NULL,
        "nameJson" text NOT NULL,
        "descriptionJson" text NOT NULL,
        "sopType" text NOT NULL,
        "processId" text NOT NULL,
        "operationId" text NOT NULL,
        "currentVersionId" text,
        "deletedAt" text
      )
    `);

    // steps
    await queryRunner.query(`
      CREATE TABLE IF NOT EXISTS "steps" (
        "id" text PRIMARY KEY NOT NULL,
        "sopVersionId" text NOT NULL,
        "stepNumber" integer NOT NULL,
        "stepType" text NOT NULL,
        "titleJson" text NOT NULL,
        "instructionJson" text NOT NULL,
        "payload" text NOT NULL,
        "isMandatory" integer NOT NULL DEFAULT 0,
        "requiresEvidence" integer NOT NULL DEFAULT 0,
        "requiresSign" integer NOT NULL DEFAULT 0,
        "skillLevelRequired" integer NOT NULL DEFAULT 0,
        "estimatedSeconds" integer NOT NULL DEFAULT 0,
        "fallbackType" text NOT NULL,
        "flowRules" text NOT NULL,
        "deletedAt" text
      )
    `);
    await queryRunner.query(`CREATE INDEX IF NOT EXISTS "idx_steps_sop_num" ON "steps" ("sopVersionId", "stepNumber")`);

    // evidence_files
    await queryRunner.query(`
      CREATE TABLE IF NOT EXISTS "evidence_files" (
        "id" text PRIMARY KEY NOT NULL,
        "workExecutionId" text NOT NULL,
        "stepId" text NOT NULL,
        "evidenceType" text NOT NULL,
        "filePath" text NOT NULL,
        "fileHashSha256" text NOT NULL,
        "fileSizeBytes" integer NOT NULL DEFAULT 0,
        "widthPx" integer,
        "heightPx" integer,
        "description" text NOT NULL,
        "uploadedBy" text NOT NULL,
        "uploadedAt" text NOT NULL,
        "synced" integer NOT NULL DEFAULT 0
      )
    `);
    await queryRunner.query(`CREATE INDEX IF NOT EXISTS "idx_evidence_step" ON "evidence_files" ("stepId")`);

    // measurements / suspensions / andon_alerts / nonconformities / capa / kaizen / users / roles / skills / user_roles / user_skills
    // processes / operations / products / lots / equipments / instruments / step_type_defs / step_flow_rules
    // hash_chain_blocks / auth_logs / devices / device_sync_states / idempotency_keys
    // materials / suppliers / incoming_inspections / sampling_plans / iqc_measurements / concession_approvals
    // lot_qc_states / reworks / dispositions / rework_verifications / rework_sop_mappings / reworked_lot_labels
    // rework_cost_records / scrap_records / return_to_vendor_records / case_locks / work_assignments / app_settings
    await queryRunner.query(`
      CREATE TABLE IF NOT EXISTS "measurements" (
        "id" text PRIMARY KEY NOT NULL,
        "workExecutionId" text NOT NULL,
        "stepId" text NOT NULL,
        "value" real NOT NULL,
        "unit" text NOT NULL,
        "usl" real,
        "lsl" real,
        "inSpec" integer NOT NULL DEFAULT 0,
        "recordedBy" text NOT NULL,
        "recordedAt" text NOT NULL
      )
    `);

    await queryRunner.query(`
      CREATE TABLE IF NOT EXISTS "suspensions" (
        "id" text PRIMARY KEY NOT NULL,
        "workExecutionId" text NOT NULL,
        "reasonCode" text NOT NULL,
        "reasonDetail" text NOT NULL,
        "suspendedAt" text NOT NULL,
        "resumedAt" text,
        "resumedBy" text
      )
    `);

    await queryRunner.query(`
      CREATE TABLE IF NOT EXISTS "andon_alerts" (
        "id" text PRIMARY KEY NOT NULL,
        "alertType" text NOT NULL,
        "severity" text NOT NULL,
        "status" text NOT NULL,
        "workExecutionId" text,
        "stepId" text,
        "raisedBy" text NOT NULL,
        "title" text NOT NULL,
        "description" text NOT NULL,
        "raisedAt" text NOT NULL,
        "acknowledgedBy" text,
        "acknowledgedAt" text,
        "resolvedBy" text,
        "resolvedAt" text,
        "resolutionNote" text
      )
    `);

    await queryRunner.query(`
      CREATE TABLE IF NOT EXISTS "nonconformities" (
        "id" text PRIMARY KEY NOT NULL,
        "alertId" text,
        "workExecutionId" text,
        "lotId" text,
        "ncType" text NOT NULL,
        "description" text NOT NULL,
        "discoveredBy" text NOT NULL,
        "discoveryStepId" text,
        "evidenceIds" text NOT NULL,
        "createdAt" text NOT NULL
      )
    `);

    await queryRunner.query(`
      CREATE TABLE IF NOT EXISTS "capa" (
        "id" text PRIMARY KEY NOT NULL,
        "nonconformityId" text,
        "title" text NOT NULL,
        "status" text NOT NULL,
        "rootCauseAnalysis" text NOT NULL,
        "correctiveAction" text NOT NULL,
        "preventiveAction" text,
        "assignedTo" text NOT NULL,
        "dueDate" text NOT NULL,
        "createdBy" text NOT NULL,
        "createdAt" text NOT NULL,
        "progressNote" text,
        "closedAt" text,
        "closedBy" text
      )
    `);

    await queryRunner.query(`
      CREATE TABLE IF NOT EXISTS "kaizen_proposals" (
        "id" text PRIMARY KEY NOT NULL,
        "proposerId" text NOT NULL,
        "processId" text,
        "category" text NOT NULL,
        "title" text NOT NULL,
        "currentSituation" text NOT NULL,
        "proposalDetail" text NOT NULL,
        "expectedBenefit" text,
        "relatedSopId" text,
        "evidenceIds" text NOT NULL,
        "status" text NOT NULL,
        "createdAt" text NOT NULL
      )
    `);

    await queryRunner.query(`
      CREATE TABLE IF NOT EXISTS "users" (
        "id" text PRIMARY KEY NOT NULL,
        "loginId" text NOT NULL UNIQUE,
        "username" text NOT NULL,
        "displayNameJson" text NOT NULL,
        "email" text,
        "role" text NOT NULL,
        "roles" text NOT NULL,
        "factoryId" text NOT NULL,
        "locale" text NOT NULL DEFAULT 'ja',
        "isActive" integer NOT NULL DEFAULT 1,
        "createdAt" text NOT NULL,
        "deletedAt" text
      )
    `);

    await queryRunner.query(`
      CREATE TABLE IF NOT EXISTS "roles" (
        "id" text PRIMARY KEY NOT NULL,
        "roleCode" text NOT NULL,
        "nameJson" text NOT NULL,
        "descriptionJson" text NOT NULL,
        "deletedAt" text
      )
    `);

    await queryRunner.query(`
      CREATE TABLE IF NOT EXISTS "skills" (
        "id" text PRIMARY KEY NOT NULL,
        "skillCode" text NOT NULL,
        "nameJson" text NOT NULL,
        "level" integer NOT NULL DEFAULT 1,
        "deletedAt" text
      )
    `);

    await queryRunner.query(`
      CREATE TABLE IF NOT EXISTS "user_roles" (
        "userId" text NOT NULL,
        "roleId" text NOT NULL,
        "grantedAt" text NOT NULL,
        "grantedBy" text,
        PRIMARY KEY ("userId", "roleId")
      )
    `);

    await queryRunner.query(`
      CREATE TABLE IF NOT EXISTS "user_skills" (
        "userId" text NOT NULL,
        "skillId" text NOT NULL,
        "level" integer NOT NULL DEFAULT 1,
        "acquiredAt" text NOT NULL,
        "expiresAt" text,
        PRIMARY KEY ("userId", "skillId")
      )
    `);

    await queryRunner.query(`
      CREATE TABLE IF NOT EXISTS "processes" (
        "id" text PRIMARY KEY NOT NULL,
        "processCode" text NOT NULL,
        "nameJson" text NOT NULL,
        "descriptionJson" text NOT NULL,
        "isActive" integer NOT NULL DEFAULT 1,
        "deletedAt" text
      )
    `);

    await queryRunner.query(`
      CREATE TABLE IF NOT EXISTS "operations" (
        "id" text PRIMARY KEY NOT NULL,
        "operationCode" text NOT NULL,
        "nameJson" text NOT NULL,
        "processId" text NOT NULL,
        "deletedAt" text
      )
    `);

    await queryRunner.query(`
      CREATE TABLE IF NOT EXISTS "products" (
        "id" text PRIMARY KEY NOT NULL,
        "productCode" text NOT NULL,
        "nameJson" text NOT NULL,
        "deletedAt" text
      )
    `);

    await queryRunner.query(`
      CREATE TABLE IF NOT EXISTS "lots" (
        "id" text PRIMARY KEY NOT NULL,
        "lotNumber" text NOT NULL UNIQUE,
        "productId" text NOT NULL,
        "externalKey" text,
        "deletedAt" text
      )
    `);

    await queryRunner.query(`
      CREATE TABLE IF NOT EXISTS "equipments" (
        "id" text PRIMARY KEY NOT NULL,
        "equipmentCode" text NOT NULL,
        "nameJson" text NOT NULL,
        "locationCode" text,
        "isActive" integer NOT NULL DEFAULT 1,
        "deletedAt" text
      )
    `);

    await queryRunner.query(`
      CREATE TABLE IF NOT EXISTS "instruments" (
        "id" text PRIMARY KEY NOT NULL,
        "instrumentCode" text NOT NULL,
        "nameJson" text NOT NULL,
        "calibrationDueAt" text,
        "deletedAt" text
      )
    `);

    await queryRunner.query(`
      CREATE TABLE IF NOT EXISTS "step_type_definitions" (
        "id" text PRIMARY KEY NOT NULL,
        "typeCode" text NOT NULL,
        "nameJson" text NOT NULL,
        "payloadSchema" text NOT NULL,
        "fallbackType" text NOT NULL,
        "deletedAt" text
      )
    `);

    await queryRunner.query(`
      CREATE TABLE IF NOT EXISTS "step_flow_rules" (
        "id" text PRIMARY KEY NOT NULL,
        "stepId" text NOT NULL,
        "ruleJson" text NOT NULL,
        "onPassNextStepId" text NOT NULL,
        "onFailNextStepId" text
      )
    `);

    await queryRunner.query(`
      CREATE TABLE IF NOT EXISTS "hash_chain_blocks" (
        "id" text PRIMARY KEY NOT NULL,
        "blockNumber" integer NOT NULL,
        "prevHash" text NOT NULL,
        "contentHash" text NOT NULL,
        "payload" text NOT NULL,
        "createdAt" text NOT NULL
      )
    `);

    await queryRunner.query(`
      CREATE TABLE IF NOT EXISTS "auth_logs" (
        "id" text PRIMARY KEY NOT NULL,
        "loginId" text NOT NULL,
        "outcome" text NOT NULL,
        "reason" text,
        "ipAddress" text,
        "deviceId" text NOT NULL,
        "createdAt" text NOT NULL
      )
    `);

    await queryRunner.query(`
      CREATE TABLE IF NOT EXISTS "devices" (
        "id" text PRIMARY KEY NOT NULL,
        "terminalCode" text NOT NULL UNIQUE,
        "externalKey" text,
        "factoryId" text NOT NULL,
        "isActive" integer NOT NULL DEFAULT 1,
        "deletedAt" text
      )
    `);

    await queryRunner.query(`
      CREATE TABLE IF NOT EXISTS "device_sync_states" (
        "entityType" text PRIMARY KEY NOT NULL,
        "cursor" text,
        "lastSyncedAt" text,
        "pendingCount" integer NOT NULL DEFAULT 0
      )
    `);

    await queryRunner.query(`
      CREATE TABLE IF NOT EXISTS "idempotency_keys" (
        "key" text PRIMARY KEY NOT NULL,
        "responseHash" text,
        "createdAt" text NOT NULL,
        "expiresAt" text NOT NULL
      )
    `);

    await queryRunner.query(`
      CREATE TABLE IF NOT EXISTS "materials" (
        "id" text PRIMARY KEY NOT NULL,
        "materialCode" text NOT NULL,
        "nameJson" text NOT NULL,
        "materialType" text NOT NULL,
        "unit" text NOT NULL,
        "deletedAt" text
      )
    `);

    await queryRunner.query(`
      CREATE TABLE IF NOT EXISTS "suppliers" (
        "id" text PRIMARY KEY NOT NULL,
        "supplierCode" text NOT NULL,
        "nameJson" text NOT NULL,
        "contactEmail" text,
        "deletedAt" text
      )
    `);

    await queryRunner.query(`
      CREATE TABLE IF NOT EXISTS "incoming_inspections" (
        "id" text PRIMARY KEY NOT NULL,
        "lotId" text NOT NULL,
        "supplierId" text NOT NULL,
        "materialId" text NOT NULL,
        "receivedQty" real NOT NULL,
        "samplingPlanId" text NOT NULL,
        "sampleSizeN" integer NOT NULL,
        "acceptNumberAc" integer NOT NULL,
        "rejectNumberRe" integer NOT NULL,
        "severityState" text NOT NULL DEFAULT 'NORMAL',
        "qcStatus" text NOT NULL,
        "defectCount" integer NOT NULL DEFAULT 0,
        "inspectedAt" text,
        "judgedAt" text,
        "judgedBy" text,
        "createdAt" text NOT NULL
      )
    `);

    await queryRunner.query(`
      CREATE TABLE IF NOT EXISTS "sampling_plans" (
        "id" text PRIMARY KEY NOT NULL,
        "planCode" text NOT NULL,
        "nameJson" text NOT NULL,
        "aqlValue" real NOT NULL,
        "inspectionLevel" text NOT NULL,
        "planSnapshot" text NOT NULL,
        "deletedAt" text
      )
    `);

    await queryRunner.query(`
      CREATE TABLE IF NOT EXISTS "incoming_inspection_measurements" (
        "id" text PRIMARY KEY NOT NULL,
        "inspectionId" text NOT NULL,
        "sampleNo" integer NOT NULL,
        "measuredValue" real,
        "defectFlag" integer NOT NULL DEFAULT 0,
        "evidenceFileId" text,
        "recordedAt" text NOT NULL,
        "recordedBy" text NOT NULL
      )
    `);

    await queryRunner.query(`
      CREATE TABLE IF NOT EXISTS "concession_approvals" (
        "id" text PRIMARY KEY NOT NULL,
        "incomingInspectionId" text NOT NULL,
        "requestedBy" text NOT NULL,
        "approvedBy" text,
        "approvalSign" text,
        "electronicSignId" text,
        "conditionNote" text NOT NULL,
        "validityScope" text NOT NULL,
        "validUntil" text,
        "status" text NOT NULL DEFAULT 'PENDING',
        "createdAt" text NOT NULL
      )
    `);

    await queryRunner.query(`
      CREATE TABLE IF NOT EXISTS "lot_qc_states" (
        "lotId" text PRIMARY KEY NOT NULL,
        "qcStatus" text NOT NULL,
        "concessionApprovalId" text,
        "validUntil" text,
        "updatedAt" text NOT NULL
      )
    `);

    await queryRunner.query(`
      CREATE TABLE IF NOT EXISTS "reworks" (
        "id" text PRIMARY KEY NOT NULL,
        "parentCaseId" text NOT NULL,
        "reworkCaseId" text NOT NULL,
        "nonconformityId" text NOT NULL,
        "sopId" text NOT NULL,
        "reworkSopVersionId" text NOT NULL,
        "assignedTo" text,
        "status" text NOT NULL,
        "reworkCount" integer NOT NULL DEFAULT 1,
        "deadline" text,
        "createdAt" text NOT NULL
      )
    `);

    await queryRunner.query(`
      CREATE TABLE IF NOT EXISTS "dispositions" (
        "id" text PRIMARY KEY NOT NULL,
        "nonconformityId" text NOT NULL,
        "dispositionType" text NOT NULL,
        "decision" text NOT NULL,
        "decisionReason" text NOT NULL,
        "qualityAdminSignId" text,
        "supervisorSignId" text,
        "signedAt" text,
        "createdAt" text NOT NULL
      )
    `);

    await queryRunner.query(`
      CREATE TABLE IF NOT EXISTS "rework_verifications" (
        "id" text PRIMARY KEY NOT NULL,
        "reworkId" text NOT NULL,
        "verifierId" text NOT NULL,
        "verifiedAt" text NOT NULL,
        "passed" integer NOT NULL DEFAULT 0,
        "note" text NOT NULL,
        "evidenceIds" text NOT NULL
      )
    `);

    await queryRunner.query(`
      CREATE TABLE IF NOT EXISTS "rework_sop_mappings" (
        "id" text PRIMARY KEY NOT NULL,
        "ncCategory" text NOT NULL,
        "reworkSopId" text NOT NULL,
        "reworkSopVersionId" text NOT NULL,
        "isActive" integer NOT NULL DEFAULT 1,
        "deletedAt" text
      )
    `);

    await queryRunner.query(`
      CREATE TABLE IF NOT EXISTS "reworked_lot_labels" (
        "id" text PRIMARY KEY NOT NULL,
        "reworkId" text NOT NULL,
        "labelCode" text NOT NULL,
        "qrPayload" text NOT NULL,
        "printedAt" text NOT NULL,
        "printedBy" text NOT NULL
      )
    `);

    await queryRunner.query(`
      CREATE TABLE IF NOT EXISTS "rework_cost_records" (
        "id" text PRIMARY KEY NOT NULL,
        "reworkId" text NOT NULL,
        "laborMinutes" real NOT NULL,
        "materialCost" real NOT NULL,
        "otherCost" real NOT NULL,
        "costCurrency" text NOT NULL,
        "recordedAt" text NOT NULL,
        "recordedBy" text NOT NULL
      )
    `);

    await queryRunner.query(`
      CREATE TABLE IF NOT EXISTS "scrap_records" (
        "id" text PRIMARY KEY NOT NULL,
        "nonconformityId" text NOT NULL,
        "scrappedBy" text NOT NULL,
        "witnessId" text NOT NULL,
        "scrappedAt" text NOT NULL,
        "quantity" real NOT NULL,
        "note" text NOT NULL
      )
    `);

    await queryRunner.query(`
      CREATE TABLE IF NOT EXISTS "return_to_vendor_records" (
        "id" text PRIMARY KEY NOT NULL,
        "nonconformityId" text NOT NULL,
        "supplierId" text NOT NULL,
        "trackingNo" text NOT NULL,
        "returnedBy" text NOT NULL,
        "returnedAt" text NOT NULL,
        "quantity" real NOT NULL
      )
    `);

    await queryRunner.query(`
      CREATE TABLE IF NOT EXISTS "case_locks" (
        "caseId" text PRIMARY KEY NOT NULL,
        "terminalId" text NOT NULL,
        "userId" text NOT NULL,
        "acquiredAt" text NOT NULL,
        "heartbeatAt" text NOT NULL,
        "lockStatus" text NOT NULL
      )
    `);

    await queryRunner.query(`
      CREATE TABLE IF NOT EXISTS "work_assignments" (
        "id" text PRIMARY KEY NOT NULL,
        "externalOrderId" text NOT NULL,
        "externalSystem" text NOT NULL,
        "sopId" text NOT NULL,
        "sopName" text NOT NULL,
        "targetTerminalId" text NOT NULL,
        "lotId" text,
        "lotNumber" text,
        "suggestedWorkerId" text,
        "suggestedEquipmentId" text,
        "dueAt" text,
        "priority" integer NOT NULL DEFAULT 0,
        "status" text NOT NULL,
        "receivedAt" text NOT NULL,
        "acknowledgedAt" text,
        "cancelledAt" text
      )
    `);
    await queryRunner.query(
      `CREATE UNIQUE INDEX IF NOT EXISTS "uniq_wa_ext_order" ON "work_assignments" ("externalOrderId", "externalSystem")`,
    );

    await queryRunner.query(`
      CREATE TABLE IF NOT EXISTS "app_settings" (
        "settingsId" text PRIMARY KEY NOT NULL,
        "locale" text NOT NULL DEFAULT 'ja',
        "darkMode" integer NOT NULL DEFAULT 0,
        "deviceId" text NOT NULL,
        "jwtCache" text,
        "jwtExpiresAt" text,
        "lastMasterSyncAt" text,
        "currentUserId" text,
        "outboxIntervalMs" integer NOT NULL DEFAULT 30000,
        "emergencyThresholdMs" integer NOT NULL DEFAULT 300000,
        "masterSyncIntervalMinutes" integer NOT NULL DEFAULT 60
      )
    `);
  }

  // 端末は migration:revert 非サポートのため down() は意図的に空のまま（前進修正のみ）
  async down(_queryRunner: QueryRunner): Promise<void> {
    return;
  }
}
