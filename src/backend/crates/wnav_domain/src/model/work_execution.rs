// 作業実行セッションのドメインモデル（EN-011）
// 1 件の WorkExecution が 1 件の実作業に対応し、TBL-005 work_executions に永続化される。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 作業実行セッション。1 件の WorkExecution が 1 件の実作業に対応する。
/// TBL-005 work_executions に永続化される。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkExecution {
    /// 作業実行 ID（UUID v7）
    pub work_execution_id: Uuid,
    /// 対象 SOP バージョン ID
    pub sop_version_id: Uuid,
    /// 主担当作業員 ID
    pub primary_worker_id: Uuid,
    /// 補助担当作業員 ID（任意）
    pub secondary_worker_id: Option<Uuid>,
    /// 端末 ID
    pub terminal_id: Uuid,
    /// 生産対象 ID（ロット・シリアル等）
    pub production_target_id: Option<String>,
    /// 現在ステータス
    pub status: WorkExecutionStatus,
    /// 現在の進行 Step 番号（0 基準）
    pub current_step_index: u32,
    /// 開始日時（UTC）
    pub started_at: Option<DateTime<Utc>>,
    /// 完了日時（UTC）
    pub completed_at: Option<DateTime<Utc>>,
    /// 楽観ロック用最終更新日時
    pub updated_at: DateTime<Utc>,
}

/// 作業実行の状態機械。
/// 合法な遷移: NotStarted→InProgress, InProgress→{Suspended,Completed,Cancelled}, Suspended→InProgress
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WorkExecutionStatus {
    /// 作業割付済み・未着手
    NotStarted,
    /// 作業実行中
    InProgress,
    /// 中断（休憩・引継ぎ）
    Suspended,
    /// 全ステップ完了（終端）
    Completed,
    /// 監督者による中止（終端）
    Cancelled,
}

impl WorkExecutionStatus {
    /// (FNC-BE-005) 現在状態から次状態への遷移が合法かどうかを返す。
    /// ALG-003 ステートマシンに従い合法な遷移のみ true を返す。
    pub fn can_transition_to(&self, next: &Self) -> bool {
        matches!(
            (self, next),
            (Self::NotStarted, Self::InProgress)
                | (Self::InProgress, Self::Suspended)
                | (Self::InProgress, Self::Completed)
                | (Self::InProgress, Self::Cancelled)
                | (Self::Suspended, Self::InProgress)
                | (Self::Suspended, Self::Cancelled)
        )
    }
}

/// ドメインイベント: WorkEvent に追加するアクティビティの種類を定義する値オブジェクト
/// XES 互換アクティビティ名（XES での lifecycle:transition）に対応する。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkEventActivity {
    /// 作業開始
    WorkStarted,
    /// ステップ完了
    StepCompleted,
    /// ステップスキップ（ALCOA+ Complete 原則によりスキップも記録する）
    StepSkipped,
    /// 証拠記録
    EvidenceRecorded,
    /// 電子サイン記録
    ElectronicSigned,
    /// 作業中断
    WorkSuspended,
    /// 作業再開
    WorkResumed,
    /// 作業完了
    WorkCompleted,
    /// 作業キャンセル
    WorkCancelled,
    /// アンドン発報
    AndonTriggered,
}

impl WorkEventActivity {
    /// XES 互換のアクティビティ文字列を返す。
    /// 自由文字列禁止（src/CLAUDE.md データ整合性の鉄則）のため列挙型で管理する。
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::WorkStarted => "work.started",
            Self::StepCompleted => "step.completed",
            Self::StepSkipped => "step.skipped",
            Self::EvidenceRecorded => "evidence.recorded",
            Self::ElectronicSigned => "electronic_sign.recorded",
            Self::WorkSuspended => "work.suspended",
            Self::WorkResumed => "work.resumed",
            Self::WorkCompleted => "work.completed",
            Self::WorkCancelled => "work.cancelled",
            Self::AndonTriggered => "andon.triggered",
        }
    }
}
