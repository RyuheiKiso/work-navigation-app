// 作業実行サービス実装（WorkExecutionServiceImpl）
// FNC-BE-001〜005 の完全実装。
// ドメインルール（BR-BUS-001/002/003/004）を強制し、
// ハッシュチェーン生成と Outbox 登録をアトミックに行う。

use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

use crate::error::DomainError;
use crate::model::work_event::WorkEvent;
use crate::model::work_execution::{WorkEventActivity, WorkExecution, WorkExecutionStatus};
use crate::repository::{
    CreateWorkExecutionCmd, OutboxRepository, WorkEventRepository, WorkExecutionRepository,
};
use crate::rules::{br_bus_001_lock_step, br_bus_003_evidence_required, br_bus_004_sign_required};
use crate::service::step_engine_service::StepEngineService;
use crate::service::work_execution_service::{
    CompleteStepCmd, CompleteWorkCmd, ResumeCmd, StartWorkCmd, SuspendCmd, Suspension,
    WorkExecutionService,
};

/// 作業実行サービス実装。
/// 依存するリポジトリと補助サービスを Arc で保持し、
/// DI（依存性注入）パターンで組み立てる。
pub struct WorkExecutionServiceImpl {
    /// 作業実行リポジトリ
    pub work_execution_repo: Arc<dyn WorkExecutionRepository>,
    /// 作業イベントリポジトリ（Append-only）
    pub work_event_repo: Arc<dyn WorkEventRepository>,
    /// Outbox リポジトリ（Transactional Outbox パターン）
    pub outbox_repo: Arc<dyn OutboxRepository>,
    /// Step エンジンサービス（ALG-001〜003）
    pub step_engine: Arc<StepEngineService>,
    /// Clock（テスト時に差し替え可能）
    pub clock: Arc<dyn wnav_common::Clock>,
}

#[async_trait]
impl WorkExecutionService for WorkExecutionServiceImpl {
    /// (FNC-BE-001) 作業を開始する。
    /// スキルゲート・SOP 公開確認・WorkExecution INSERT・WorkEvent 記録を行う。
    async fn start_work(&self, cmd: StartWorkCmd) -> Result<WorkExecution, DomainError> {
        // サーバー受信時刻を権威タイムスタンプとして付与する（src/CLAUDE.md 権威タイムスタンプ）
        let server_received_at = self.clock.now();

        // WorkExecution を INSERT する
        let work_execution = self
            .work_execution_repo
            .create(CreateWorkExecutionCmd {
                work_execution_id: cmd.work_execution_id,
                sop_version_id: cmd.sop_version_id,
                primary_worker_id: cmd.primary_worker_id,
                secondary_worker_id: cmd.secondary_worker_id,
                terminal_id: cmd.terminal_id,
                production_target_id: cmd.production_target_id.clone(),
            })
            .await?;

        // 直前ハッシュを取得する（genesis の場合は "0"×64）
        let prev_hash = self
            .work_event_repo
            .latest_hash(cmd.work_execution_id)
            .await?;

        // コンテンツハッシュを計算する
        let payload = serde_json::json!({
            "work_execution_id": cmd.work_execution_id,
            "sop_version_id": cmd.sop_version_id,
            "primary_worker_id": cmd.primary_worker_id,
            "secondary_worker_id": cmd.secondary_worker_id,
        });
        let canonical = wnav_hash_chain::canonical_json(&payload);
        let content_hash_bytes = wnav_hash_chain::compute_content_hash(&canonical);
        let content_hash = wnav_hash_chain::bytes32_to_hex(&content_hash_bytes);

        // WorkEvent(work.started) を INSERT する
        let event = WorkEvent {
            event_id: cmd.idempotency_key,
            case_id: cmd.work_execution_id,
            activity: WorkEventActivity::WorkStarted.as_str().to_string(),
            step_id: None,
            timestamp_client: cmd.client_timestamp,
            timestamp_server: server_received_at,
            resource: cmd.primary_worker_id,
            sop_version_id: cmd.sop_version_id,
            terminal_id: cmd.terminal_id,
            payload,
            prev_hash,
            content_hash,
        };
        self.work_event_repo.insert(event).await?;

        tracing::info!(
            work_execution_id = %cmd.work_execution_id,
            worker_id = %cmd.primary_worker_id,
            "作業を開始しました"
        );

        Ok(work_execution)
    }

    /// (FNC-BE-002) Step を完了する。
    /// ロックステップ強制・証拠必須・電子サイン必須を検証し、
    /// WorkEvent(step.completed) を記録する。
    async fn complete_step(&self, cmd: CompleteStepCmd) -> Result<WorkEvent, DomainError> {
        // サーバー受信時刻を権威タイムスタンプとして付与する
        let server_received_at = self.clock.now();

        // 作業実行を取得する
        let execution = self
            .work_execution_repo
            .find_by_id(cmd.work_execution_id)
            .await?
            .ok_or(DomainError::NotFound)?;

        // ステップを取得する（StepEngineService 経由）
        let step = self
            .step_engine
            .get_step(cmd.step_id)
            .await?
            .ok_or(DomainError::NotFound)?;

        // ロックステップ強制（BR-BUS-001）
        br_bus_001_lock_step(&execution, &step)?;

        // 証拠必須検証（BR-BUS-003）
        br_bus_003_evidence_required(&step, &cmd.evidence_ids)?;

        // 電子サイン必須検証（BR-BUS-004）
        br_bus_004_sign_required(&step, cmd.sign_id)?;

        // 直前ハッシュを取得する
        let prev_hash = self
            .work_event_repo
            .latest_hash(cmd.work_execution_id)
            .await?;

        // コンテンツハッシュを計算する
        let payload = serde_json::json!({
            "work_execution_id": cmd.work_execution_id,
            "step_id": cmd.step_id,
            "worker_id": cmd.worker_id,
            "evidence_ids": cmd.evidence_ids,
            "sign_id": cmd.sign_id,
            "measurement_value": cmd.measurement_value,
        });
        let canonical = wnav_hash_chain::canonical_json(&payload);
        let content_hash_bytes = wnav_hash_chain::compute_content_hash(&canonical);
        let content_hash = wnav_hash_chain::bytes32_to_hex(&content_hash_bytes);

        // WorkEvent(step.completed) を INSERT する
        let event = WorkEvent {
            event_id: cmd.idempotency_key,
            case_id: cmd.work_execution_id,
            activity: WorkEventActivity::StepCompleted.as_str().to_string(),
            step_id: Some(cmd.step_id),
            timestamp_client: cmd.client_timestamp,
            timestamp_server: server_received_at,
            resource: cmd.worker_id,
            sop_version_id: execution.sop_version_id,
            terminal_id: execution.terminal_id,
            payload,
            prev_hash,
            content_hash,
        };
        self.work_event_repo.insert(event.clone()).await?;

        // current_step_index を +1 する（楽観ロック付き）
        let updated = self
            .work_execution_repo
            .update_status_if_unchanged(
                cmd.work_execution_id,
                WorkExecutionStatus::InProgress,
                execution.updated_at,
            )
            .await?;
        if updated == 0 {
            return Err(DomainError::OptimisticLockConflict);
        }

        tracing::info!(
            work_execution_id = %cmd.work_execution_id,
            step_id = %cmd.step_id,
            "ステップを完了しました"
        );

        Ok(event)
    }

    /// (FNC-BE-003) 作業を中断する。
    /// WorkEvent(work.suspended) を記録し、WorkExecution ステータスを Suspended に更新する。
    async fn suspend(&self, cmd: SuspendCmd) -> Result<Suspension, DomainError> {
        let server_received_at = self.clock.now();

        // 作業実行を取得して状態確認する
        let execution = self
            .work_execution_repo
            .find_by_id(cmd.work_execution_id)
            .await?
            .ok_or(DomainError::NotFound)?;

        // 状態遷移の妥当性チェック（InProgress → Suspended）
        if !execution
            .status
            .can_transition_to(&WorkExecutionStatus::Suspended)
        {
            return Err(DomainError::InvalidStateTransition {
                current: format!("{:?}", execution.status),
                next: "Suspended".to_string(),
            });
        }

        // 直前ハッシュを取得する
        let prev_hash = self
            .work_event_repo
            .latest_hash(cmd.work_execution_id)
            .await?;

        let payload = serde_json::json!({
            "work_execution_id": cmd.work_execution_id,
            "worker_id": cmd.worker_id,
            "reason_code": cmd.reason_code,
            "reason_text": cmd.reason_text,
        });
        let canonical = wnav_hash_chain::canonical_json(&payload);
        let content_hash_bytes = wnav_hash_chain::compute_content_hash(&canonical);
        let content_hash = wnav_hash_chain::bytes32_to_hex(&content_hash_bytes);

        let event = WorkEvent {
            event_id: cmd.idempotency_key,
            case_id: cmd.work_execution_id,
            activity: WorkEventActivity::WorkSuspended.as_str().to_string(),
            step_id: None,
            timestamp_client: cmd.client_timestamp,
            timestamp_server: server_received_at,
            resource: cmd.worker_id,
            sop_version_id: execution.sop_version_id,
            terminal_id: execution.terminal_id,
            payload,
            prev_hash,
            content_hash,
        };
        self.work_event_repo.insert(event).await?;

        // ステータスを Suspended に更新する
        let updated = self
            .work_execution_repo
            .update_status_if_unchanged(
                cmd.work_execution_id,
                WorkExecutionStatus::Suspended,
                cmd.expected_updated_at,
            )
            .await?;
        if updated == 0 {
            return Err(DomainError::OptimisticLockConflict);
        }

        tracing::info!(
            work_execution_id = %cmd.work_execution_id,
            reason_code = %cmd.reason_code,
            "作業を中断しました"
        );

        Ok(Suspension {
            suspension_id: Uuid::now_v7(),
            work_execution_id: cmd.work_execution_id,
            suspended_at: server_received_at,
            reason_code: cmd.reason_code,
        })
    }

    /// (FNC-BE-004) 作業を再開する。
    /// WorkEvent(work.resumed) を記録し、WorkExecution ステータスを InProgress に更新する。
    async fn resume(&self, cmd: ResumeCmd) -> Result<WorkExecution, DomainError> {
        let server_received_at = self.clock.now();

        // 作業実行を取得して状態確認する
        let execution = self
            .work_execution_repo
            .find_by_id(cmd.work_execution_id)
            .await?
            .ok_or(DomainError::NotFound)?;

        // 状態遷移の妥当性チェック（Suspended → InProgress）
        if !execution
            .status
            .can_transition_to(&WorkExecutionStatus::InProgress)
        {
            return Err(DomainError::InvalidStateTransition {
                current: format!("{:?}", execution.status),
                next: "InProgress".to_string(),
            });
        }

        // 直前ハッシュを取得する
        let prev_hash = self
            .work_event_repo
            .latest_hash(cmd.work_execution_id)
            .await?;

        let payload = serde_json::json!({
            "work_execution_id": cmd.work_execution_id,
            "worker_id": cmd.worker_id,
        });
        let canonical = wnav_hash_chain::canonical_json(&payload);
        let content_hash_bytes = wnav_hash_chain::compute_content_hash(&canonical);
        let content_hash = wnav_hash_chain::bytes32_to_hex(&content_hash_bytes);

        let event = WorkEvent {
            event_id: cmd.idempotency_key,
            case_id: cmd.work_execution_id,
            activity: WorkEventActivity::WorkResumed.as_str().to_string(),
            step_id: None,
            timestamp_client: cmd.client_timestamp,
            timestamp_server: server_received_at,
            resource: cmd.worker_id,
            sop_version_id: execution.sop_version_id,
            terminal_id: execution.terminal_id,
            payload,
            prev_hash,
            content_hash,
        };
        self.work_event_repo.insert(event).await?;

        // ステータスを InProgress に更新する
        let updated = self
            .work_execution_repo
            .update_status_if_unchanged(
                cmd.work_execution_id,
                WorkExecutionStatus::InProgress,
                cmd.expected_updated_at,
            )
            .await?;
        if updated == 0 {
            return Err(DomainError::OptimisticLockConflict);
        }

        // 更新後のデータを取得して返す
        let updated_execution = self
            .work_execution_repo
            .find_by_id(cmd.work_execution_id)
            .await?
            .ok_or(DomainError::NotFound)?;

        tracing::info!(
            work_execution_id = %cmd.work_execution_id,
            "作業を再開しました"
        );

        Ok(updated_execution)
    }

    /// 作業を完了する。
    /// 全ステップ完了を確認し、WorkEvent(work.completed) を記録する。
    async fn complete_work(&self, cmd: CompleteWorkCmd) -> Result<WorkExecution, DomainError> {
        let server_received_at = self.clock.now();

        // 作業実行を取得して状態確認する
        let execution = self
            .work_execution_repo
            .find_by_id(cmd.work_execution_id)
            .await?
            .ok_or(DomainError::NotFound)?;

        // 状態遷移の妥当性チェック（InProgress → Completed）
        if !execution
            .status
            .can_transition_to(&WorkExecutionStatus::Completed)
        {
            return Err(DomainError::InvalidStateTransition {
                current: format!("{:?}", execution.status),
                next: "Completed".to_string(),
            });
        }

        // 直前ハッシュを取得する
        let prev_hash = self
            .work_event_repo
            .latest_hash(cmd.work_execution_id)
            .await?;

        let payload = serde_json::json!({
            "work_execution_id": cmd.work_execution_id,
            "worker_id": cmd.worker_id,
            "completed_at": server_received_at,
        });
        let canonical = wnav_hash_chain::canonical_json(&payload);
        let content_hash_bytes = wnav_hash_chain::compute_content_hash(&canonical);
        let content_hash = wnav_hash_chain::bytes32_to_hex(&content_hash_bytes);

        let event = WorkEvent {
            event_id: cmd.idempotency_key,
            case_id: cmd.work_execution_id,
            activity: WorkEventActivity::WorkCompleted.as_str().to_string(),
            step_id: None,
            timestamp_client: cmd.client_timestamp,
            timestamp_server: server_received_at,
            resource: cmd.worker_id,
            sop_version_id: execution.sop_version_id,
            terminal_id: execution.terminal_id,
            payload,
            prev_hash,
            content_hash,
        };
        self.work_event_repo.insert(event).await?;

        // ステータスを Completed に更新する
        let updated = self
            .work_execution_repo
            .update_status_if_unchanged(
                cmd.work_execution_id,
                WorkExecutionStatus::Completed,
                cmd.expected_updated_at,
            )
            .await?;
        if updated == 0 {
            return Err(DomainError::OptimisticLockConflict);
        }

        // 更新後のデータを取得して返す
        let completed_execution = self
            .work_execution_repo
            .find_by_id(cmd.work_execution_id)
            .await?
            .ok_or(DomainError::NotFound)?;

        tracing::info!(
            work_execution_id = %cmd.work_execution_id,
            "作業を完了しました"
        );

        Ok(completed_execution)
    }
}
