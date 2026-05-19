// 作業指示 Push 受信 DTO（API-sync-003 / MOD-BE-011）
//
// 外部システムから Push 受信する作業指示の Request/Response 型。
// Idempotent 設計: idempotency_key で重複チェックを行う。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// 作業指示 Push 受信リクエスト（API-sync-003）
///
/// X-Signature-256 ヘッダで HMAC-SHA256 署名を検証する。
/// idempotency_key で重複排除を行う（同一キーの再送は 200 を返して無視する）。
#[derive(Debug, Deserialize, ToSchema)]
#[allow(dead_code)]
pub struct WorkAssignmentPushRequest {
    /// 冪等キー（外部システムが UUID v4 で採番する）
    pub idempotency_key: Uuid,
    /// 作業指示 ID（外部システムの識別子）
    pub external_assignment_id: String,
    /// 送信元システム識別子（例: "SAP_PP"）
    pub external_system: String,
    /// 工場 ID（ver1.0.0 では定数 UUID を使用する）
    pub factory_id: Uuid,
    /// 作業パターン ID（内部の work_pattern_id）
    pub work_pattern_id: Uuid,
    /// 配信先端末 ID（target_terminal_id）
    pub target_terminal_id: Uuid,
    /// 計画開始日時
    pub scheduled_start_at: DateTime<Utc>,
    /// 計画終了日時
    pub scheduled_end_at: DateTime<Utc>,
    /// 作業指示データ（JSON 形式・任意フィールド）
    pub payload: serde_json::Value,
}

/// 作業指示 Push 受信レスポンス（API-sync-003）
///
/// 新規登録の場合は 201 Created、重複の場合は 200 OK を返す。
#[derive(Debug, Serialize, ToSchema)]
pub struct WorkAssignmentPushResponse {
    /// 登録された作業指示の内部 ID
    pub assignment_id: Uuid,
    /// 受信した冪等キー（確認用）
    pub idempotency_key: Uuid,
    /// 重複受信の場合は true（この場合は DB に書き込まれていない）
    pub is_duplicate: bool,
    /// サーバー受信時刻（UTC ms）
    pub server_received_at: i64,
}
