// マスタバージョンサービス（SOP 公開フロー）
// Draft → UnderReview → Published の承認フローを実装する。
// BR-BUS-012: Published への遷移には電子サイン + quality_admin ロールが必須。

use std::sync::Arc;

use uuid::Uuid;

use crate::error::DomainError;
use crate::model::master_version::{MasterVersion, MasterVersionStatus};
use crate::repository::{CreateMasterVersionCmd, MasterVersionRepository, StepRepository};
use crate::rules::br_bus_012_publish_requires_sign;

/// マスタバージョンサービス。
/// SOP 公開フロー（Draft → UnderReview → Published）を管理する。
pub struct MasterVersionService {
    /// マスタバージョンリポジトリ
    pub master_version_repo: Arc<dyn MasterVersionRepository>,
    /// ステップリポジトリ（参照整合性チェックに使用）
    pub step_repo: Arc<dyn StepRepository>,
}

impl MasterVersionService {
    /// 新しいマスタバージョンを作成する（Draft 状態）。
    pub async fn create(
        &self,
        sop_id: Uuid,
        version_number: String,
        created_by: Uuid,
    ) -> Result<MasterVersion, DomainError> {
        let cmd = CreateMasterVersionCmd {
            master_version_id: Uuid::now_v7(),
            sop_id,
            version_number,
            created_by,
        };
        self.master_version_repo.create(cmd).await
    }

    /// Draft → UnderReview への遷移（レビュー開始）。
    pub async fn submit_for_review(
        &self,
        version_id: Uuid,
        submitted_by: Uuid,
    ) -> Result<MasterVersion, DomainError> {
        let version = self
            .master_version_repo
            .find_by_id(version_id)
            .await?
            .ok_or(DomainError::NotFound)?;

        // 状態遷移の妥当性チェック
        if !version
            .status
            .can_transition_to(&MasterVersionStatus::UnderReview)
        {
            return Err(DomainError::InvalidStateTransition {
                current: format!("{:?}", version.status),
                next: "UnderReview".to_string(),
            });
        }

        // dry-run: ステップが 1 件以上あることを確認する（参照整合性）
        let steps = self.step_repo.find_by_sop(version.sop_id).await?;
        if steps.is_empty() {
            return Err(DomainError::Internal(
                "SOP にステップが存在しません。レビュー申請には 1 件以上のステップが必要です"
                    .to_string(),
            ));
        }

        tracing::info!(version_id = %version_id, submitted_by = %submitted_by, "マスタバージョンをレビュー申請しました");

        self.master_version_repo
            .update_status(version_id, MasterVersionStatus::UnderReview, None)
            .await
    }

    /// UnderReview → Published への遷移（公開承認）。
    /// BR-BUS-012: 電子サイン必須。
    pub async fn publish(
        &self,
        version_id: Uuid,
        approved_by: Uuid,
        sign_id: Option<Uuid>,
    ) -> Result<MasterVersion, DomainError> {
        // 電子サイン必須チェック（BR-BUS-012）
        br_bus_012_publish_requires_sign(sign_id)?;

        let version = self
            .master_version_repo
            .find_by_id(version_id)
            .await?
            .ok_or(DomainError::NotFound)?;

        // 状態遷移の妥当性チェック
        if !version
            .status
            .can_transition_to(&MasterVersionStatus::Published)
        {
            return Err(DomainError::InvalidStateTransition {
                current: format!("{:?}", version.status),
                next: "Published".to_string(),
            });
        }

        tracing::info!(version_id = %version_id, approved_by = %approved_by, "マスタバージョンを公開しました");

        self.master_version_repo
            .update_status(
                version_id,
                MasterVersionStatus::Published,
                Some(approved_by),
            )
            .await
    }

    /// Published → Archived への遷移（廃止）。
    pub async fn archive(
        &self,
        version_id: Uuid,
        archived_by: Uuid,
    ) -> Result<MasterVersion, DomainError> {
        let version = self
            .master_version_repo
            .find_by_id(version_id)
            .await?
            .ok_or(DomainError::NotFound)?;

        // 状態遷移の妥当性チェック
        if !version
            .status
            .can_transition_to(&MasterVersionStatus::Archived)
        {
            return Err(DomainError::InvalidStateTransition {
                current: format!("{:?}", version.status),
                next: "Archived".to_string(),
            });
        }

        tracing::info!(version_id = %version_id, archived_by = %archived_by, "マスタバージョンをアーカイブしました");

        self.master_version_repo
            .update_status(version_id, MasterVersionStatus::Archived, None)
            .await
    }

    /// dry-run: 公開前の参照整合性チェック。
    /// ステップ数の確認・condition_dsl の構文チェック等を行う。
    pub async fn dry_run_publish_check(
        &self,
        version_id: Uuid,
    ) -> Result<DryRunResult, DomainError> {
        let version = self
            .master_version_repo
            .find_by_id(version_id)
            .await?
            .ok_or(DomainError::NotFound)?;

        let steps = self.step_repo.find_by_sop(version.sop_id).await?;

        // ステップ数の確認
        let step_count = steps.len();
        let has_steps = step_count > 0;

        // condition_dsl のある全ステップを深度検証する
        let mut dsl_errors = Vec::new();
        for step in &steps {
            if let Some(rule) = &step.condition_dsl {
                if let Err(e) =
                    crate::service::json_logic_evaluator::JsonLogicEvaluator::validate_rule_depth(
                        rule, 0, 5,
                    )
                {
                    dsl_errors.push(format!("ステップ {}: {}", step.step_number, e));
                }
            }
        }

        // dsl_errors の空チェックは移動前に行う
        let is_publishable = has_steps && dsl_errors.is_empty();

        Ok(DryRunResult {
            step_count,
            has_steps,
            dsl_errors,
            is_publishable,
        })
    }
}

/// dry-run チェック結果。
#[derive(Debug)]
pub struct DryRunResult {
    /// ステップ数
    pub step_count: usize,
    /// ステップが 1 件以上あるか
    pub has_steps: bool,
    /// DSL バリデーションエラー（空リストは正常）
    pub dsl_errors: Vec<String>,
    /// 公開可能かどうか
    pub is_publishable: bool,
}
