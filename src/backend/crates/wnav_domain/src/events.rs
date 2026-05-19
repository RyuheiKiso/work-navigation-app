// ドメインイベント定義
// コンテキスト間疎結合のための内部ドメインイベント。
// tokio broadcast channel（容量 100）で配信される。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use uuid::Uuid;

/// コンテキスト間疎結合のための内部ドメインイベント。
/// tokio broadcast channel（容量 100）で配信される。
/// 各イベントはドメインサービスが発行し、リスナー（Outbox Worker・品質モジュール等）が受信する。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DomainEvent {
    /// Execution → Integration: Outbox 登録トリガ
    WorkStarted {
        /// 作業実行 ID
        work_execution_id: Uuid,
        /// SOP バージョン ID
        sop_version_id: Uuid,
        /// 主担当作業員 ID
        primary_worker_id: Uuid,
        /// 発生日時
        occurred_at: DateTime<Utc>,
    },

    /// Execution → Integration: Outbox 登録トリガ
    StepCompleted {
        /// 作業実行 ID
        work_execution_id: Uuid,
        /// 完了したステップ ID
        step_id: Uuid,
        /// 作業員 ID
        worker_id: Uuid,
        /// 添付証拠 ID 一覧
        evidence_ids: Vec<Uuid>,
        /// 発生日時
        occurred_at: DateTime<Utc>,
    },

    /// Execution → Integration: Outbox 登録トリガ
    WorkCompleted {
        /// 作業実行 ID
        work_execution_id: Uuid,
        /// 完了者 ID
        completed_by: Uuid,
        /// 発生日時
        occurred_at: DateTime<Utc>,
    },

    /// Execution → Quality: アンドン連動
    WorkSuspended {
        /// 作業実行 ID
        work_execution_id: Uuid,
        /// 中断者 ID
        suspended_by: Uuid,
        /// 中断理由コード
        reason_code: String,
        /// 発生日時
        occurred_at: DateTime<Utc>,
    },

    /// Authoring → Integration: 端末配信トリガ（MSG-004）
    MasterVersionPublished {
        /// マスタバージョン ID
        master_version_id: Uuid,
        /// SOP ID
        sop_id: Uuid,
        /// 公開者 ID
        published_by: Uuid,
        /// 発生日時
        occurred_at: DateTime<Utc>,
    },

    /// Evidence → Integration: Outbox 登録トリガ
    EvidenceRecorded {
        /// 証拠 ID
        evidence_id: Uuid,
        /// 作業実行 ID
        work_execution_id: Uuid,
        /// 記録者 ID
        recorded_by: Uuid,
        /// 発生日時
        occurred_at: DateTime<Utc>,
    },
}

/// ドメインイベント送信者（tokio broadcast channel の容量は 100）。
pub type DomainEventSender = broadcast::Sender<DomainEvent>;

/// ドメインイベント受信者。
pub type DomainEventReceiver = broadcast::Receiver<DomainEvent>;

/// ドメインイベントチャンネルを作成する（容量 100）。
/// アプリケーション起動時に一度だけ呼び出す。
pub fn create_domain_event_channel() -> (DomainEventSender, DomainEventReceiver) {
    broadcast::channel(100)
}
