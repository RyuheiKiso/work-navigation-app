// 作業指示リポジトリ Trait
// 外部システムから受信する作業指示の永続化・配信 Trait。

use async_trait::async_trait;
use uuid::Uuid;

use crate::error::DomainError;
use crate::model::work_assignment::WorkAssignment;

/// 作業指示リポジトリ Trait。
#[async_trait]
pub trait WorkAssignmentRepository: Send + Sync + 'static {
    /// 作業指示を INSERT する（冪等性キー重複時は DuplicateExternalKey エラー）。
    async fn insert(&self, assignment: WorkAssignment) -> Result<(), DomainError>;

    /// 端末への配信待ち作業指示を取得する。
    async fn find_pending_for_terminal(
        &self,
        terminal_id: Uuid,
    ) -> Result<Vec<WorkAssignment>, DomainError>;

    /// 作業指示を配信済みに更新する。
    async fn mark_dispatched(&self, assignment_id: Uuid) -> Result<(), DomainError>;

    /// 冪等性キーで作業指示を検索する（重複受信チェック）。
    async fn find_by_idempotency_key(
        &self,
        key: Uuid,
    ) -> Result<Option<WorkAssignment>, DomainError>;
}
