// 作業イベントのドメインモデル（EN-012）
// Append-only のイベントストア（TBL-001 work_events）に記録される。
// XES 互換必須属性を全て含み、SHA-256 ハッシュチェーンで改ざん検出を保証する。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

/// 作業イベント。Append-only のイベントストア（TBL-001 work_events）に記録される。
/// 一度 INSERT されたレコードは UPDATE・DELETE しない（src/CLAUDE.md Append-only 原則）。
///
/// # XES 互換必須属性
/// - case_id: 作業実行 ID（XES の Case ID）
/// - activity: アクティビティ名（XES の Activity）
/// - timestamp_server: サーバー受信時刻（XES の Timestamp・権威タイムスタンプ）
/// - resource: 作業員 ID（XES の Resource）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkEvent {
    /// イベント ID（UUID v7、Idempotency Key と同一値）
    pub event_id: Uuid,
    /// 作業実行 ID（XES での case_id）
    pub case_id: Uuid,
    /// XES アクティビティ名（例: "step.completed"・"work.started"）
    pub activity: String,
    /// Step ID（任意。Step 関連イベントのみ設定）
    pub step_id: Option<Uuid>,
    /// クライアント記録日時（申告値。権威タイムスタンプではない）
    pub timestamp_client: DateTime<Utc>,
    /// サーバー受信日時（権威タイムスタンプ。サーバー側で付与する）
    pub timestamp_server: DateTime<Utc>,
    /// 作業員 ID（XES での resource）
    pub resource: Uuid,
    /// 使用 SOP バージョン ID（時点参照。過去の記録が参照したマスタ版を保存する）
    pub sop_version_id: Uuid,
    /// 端末 ID
    pub terminal_id: Uuid,
    /// イベント固有データ（JSONB。ステップ入力・測定値・証拠 ID 等を格納する）
    pub payload: Value,
    /// 前イベントのチェーンハッシュ（SHA-256 hex 64 桁。genesis は "0"×64）
    pub prev_hash: String,
    /// 本イベントのコンテンツハッシュ（SHA-256 hex 64 桁）
    pub content_hash: String,
}
