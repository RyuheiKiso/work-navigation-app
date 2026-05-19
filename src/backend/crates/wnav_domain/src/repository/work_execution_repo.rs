// 作業実行リポジトリ Trait（FNC-BE-006/007）
// 実装は `crates/wnav_db/` の PgWorkExecutionRepository が担う。
// Domain 層はこの Trait のみを参照し、sqlx 等の実装詳細に依存しない。

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::error::DomainError;
use crate::model::pagination::{Page, Pagination};
use crate::model::work_execution::{WorkExecution, WorkExecutionStatus};

/// 作業実行リポジトリ Trait。
/// 実装は `crates/wnav_db/` の PgWorkExecutionRepository が担う。
#[async_trait]
pub trait WorkExecutionRepository: Send + Sync + 'static {
    /// (FNC-BE-006) ID で単一の作業実行を検索する。
    async fn find_by_id(&self, id: Uuid) -> Result<Option<WorkExecution>, DomainError>;

    /// 作業実行の一覧を取得する（作業員 ID・ステータスでフィルタ可）
    async fn list(
        &self,
        filter: WorkExecutionFilter,
        page: Pagination,
    ) -> Result<Page<WorkExecution>, DomainError>;

    /// (FNC-BE-007) 新規作業実行を INSERT する。
    async fn create(&self, cmd: CreateWorkExecutionCmd) -> Result<WorkExecution, DomainError>;

    /// 楽観ロック付きステータス更新。変更された行数を返す（0 なら競合）。
    /// OptimisticLockConflict を返す場合は呼び出し元がリトライする。
    async fn update_status_if_unchanged(
        &self,
        id: Uuid,
        new_status: WorkExecutionStatus,
        expected_updated_at: DateTime<Utc>,
    ) -> Result<u64, DomainError>;
}

/// 作業実行一覧のフィルタ条件。
#[derive(Debug, Default)]
pub struct WorkExecutionFilter {
    /// 作業員 ID でフィルタ
    pub worker_id: Option<Uuid>,
    /// ステータスでフィルタ
    pub status: Option<WorkExecutionStatus>,
    /// SOP バージョン ID でフィルタ
    pub sop_version_id: Option<Uuid>,
}

/// 新規作業実行作成コマンド。
#[derive(Debug)]
pub struct CreateWorkExecutionCmd {
    /// 作業実行 ID（UUID v7。クライアントが事前生成）
    pub work_execution_id: Uuid,
    /// SOP バージョン ID
    pub sop_version_id: Uuid,
    /// 主担当作業員 ID
    pub primary_worker_id: Uuid,
    /// 補助担当作業員 ID（任意）
    pub secondary_worker_id: Option<Uuid>,
    /// 端末 ID
    pub terminal_id: Uuid,
    /// 生産対象 ID（ロット・シリアル等）
    pub production_target_id: Option<String>,
}
