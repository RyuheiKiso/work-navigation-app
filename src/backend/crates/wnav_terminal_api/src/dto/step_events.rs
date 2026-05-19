// ステップイベント API（API-step-events-001）の DTO 定義（03_作業実行API仕様.md §8）

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// ステップイベント記録リクエスト（API-step-events-001）
///
/// activity フィールドで種別を判定する:
/// - step_completed / step_skipped / evidence_attached / sign_applied / measurement_recorded
#[derive(Debug, Deserialize, ToSchema)]
#[allow(dead_code)]
pub struct StepEventRequest {
    /// activity タイプ（列挙値、自由文字列は不可）
    pub activity: String,
    /// 対象ステップ ID（current_step_id と一致する必要がある）
    pub step_id: Uuid,
    /// ステップ番号（順序検証用）
    pub step_number: Option<i32>,
    /// クライアント側のタイムスタンプ
    pub timestamp_client: DateTime<Utc>,
    /// 実施時間（秒、step_completed 専用）
    pub duration_seconds: Option<i32>,
    /// 備考（step_completed 専用、最大 500 文字）
    pub remarks: Option<String>,
    /// スキップ理由コード（step_skipped 専用）
    pub skip_reason: Option<String>,
    /// スキップ理由補足（step_skipped 専用、最大 500 文字）
    pub skip_reason_detail: Option<String>,
    /// 添付エビデンス ID（evidence_attached 専用、TBL-009 に存在すること）
    pub evidence_id: Option<Uuid>,
    /// 電子サイン ID（sign_applied 専用、TBL-002 に存在すること）
    pub electronic_sign_id: Option<Uuid>,
    /// 計測項目 ID（measurement_recorded 専用、TBL-029 の定義と一致すること）
    pub measurement_item_id: Option<Uuid>,
    /// 計測値（measurement_recorded 専用）
    pub value: Option<f64>,
    /// 計測単位（measurement_recorded 専用、1〜20 文字）
    pub unit: Option<String>,
    /// 使用計測器 ID（measurement_recorded 専用、TBL-026 に存在すること）
    pub instrument_id: Option<Uuid>,
}

/// ステップイベント記録レスポンスの data フィールド（API-step-events-001）
#[derive(Debug, Serialize, ToSchema)]
pub struct StepEventData {
    /// 記録されたイベント ID（TBL-001）
    pub event_id: Uuid,
    /// 対象作業実行 ID
    pub work_execution_id: Uuid,
    /// activity タイプ
    pub activity: String,
    /// 対象ステップ ID
    pub step_id: Uuid,
    /// サーバー側のタイムスタンプ（権威タイムスタンプ）
    pub timestamp_server: DateTime<Utc>,
    /// 前ブロックのハッシュ値
    pub hash_chain_prev: String,
    /// 今回ブロックのハッシュ値
    pub hash_chain_current: String,
    /// 次のステップ ID（完了後は null）
    pub next_step_id: Option<Uuid>,
}
