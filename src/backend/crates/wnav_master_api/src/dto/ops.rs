// 運用・監視 API DTO（API-ops-001〜002）
//
// DLQ（Dead Letter Queue）照会・再キュー・ハッシュチェーン検証の型。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// DLQ エントリ（Outbox Dead Letter Queue）
#[derive(Debug, Serialize, ToSchema)]
pub struct DlqEntry {
    /// DLQ エントリ ID
    pub id: Uuid,
    /// 元のイベント ID
    pub event_id: Uuid,
    /// イベント種別
    pub event_type: String,
    /// 最後のエラーメッセージ
    pub last_error: String,
    /// リトライ回数
    pub retry_count: i32,
    /// Dead Letter に移動した日時
    pub dead_lettered_at: DateTime<Utc>,
}

/// DLQ 一覧レスポンス
#[derive(Debug, Serialize, ToSchema)]
pub struct DlqListResponse {
    /// DLQ エントリ一覧
    pub items: Vec<DlqEntry>,
    /// 総件数
    pub total: i64,
}

/// 再キューリクエスト（DLQ エントリを再送する）
#[derive(Debug, Deserialize, ToSchema)]
pub struct RequeueRequest {
    /// 再キュー理由
    pub reason: Option<String>,
}

/// 再キューレスポンス
#[derive(Debug, Serialize, ToSchema)]
pub struct RequeueResponse {
    /// 再キューされたイベント ID
    pub event_id: Uuid,
    /// 再キュー成功メッセージ
    pub message: String,
}

/// ハッシュチェーン手動検証リクエスト（BAT-001 の手動実行）
#[derive(Debug, Deserialize, ToSchema)]
pub struct HashChainVerifyRequest {
    /// 検証開始 case_id（None の場合は全件検証）
    pub case_id: Option<Uuid>,
    /// 検証件数上限（None の場合は無制限）
    pub limit: Option<i64>,
}

/// ハッシュチェーン検証レスポンス
#[derive(Debug, Serialize, ToSchema)]
pub struct HashChainVerifyResponse {
    /// 検証済み件数
    pub verified_count: i64,
    /// 破断検知件数
    pub broken_count: i64,
    /// 破断が検知された case_id 一覧
    pub broken_case_ids: Vec<Uuid>,
    /// 検証完了日時
    pub verified_at: DateTime<Utc>,
}

/// マスタ同期トリガーレスポンス
#[derive(Debug, Serialize, ToSchema)]
pub struct MasterSyncResponse {
    /// 同期開始メッセージ
    pub message: String,
    /// 同期開始日時
    pub started_at: DateTime<Utc>,
}
