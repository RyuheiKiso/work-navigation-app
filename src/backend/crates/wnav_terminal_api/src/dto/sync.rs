// 同期 API（API-sync-001〜002）の DTO 定義（07_運用・監視API仕様.md §5〜6）

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// マスタ差分同期クエリパラメータ（API-sync-001）
#[derive(Debug, Deserialize, ToSchema)]
pub struct MasterSyncQuery {
    /// 指定時刻以降に更新されたデータのみ返す（差分同期）
    pub since: Option<DateTime<Utc>>,
    /// カンマ区切りのリソース種別（sops,processes,users,equipments,instruments）
    pub resource_types: Option<String>,
}

/// マスタ差分同期レスポンスの data フィールド（API-sync-001）
#[derive(Debug, Serialize, ToSchema)]
pub struct MasterSyncData {
    /// サーバー側の同期タイムスタンプ
    pub sync_timestamp: DateTime<Utc>,
    /// SOP データ（差分）
    pub sops: Vec<serde_json::Value>,
    /// 工程データ（差分）
    pub processes: Vec<serde_json::Value>,
    /// ユーザーデータ（差分）
    pub users: Vec<serde_json::Value>,
    /// 追加データが存在するか
    pub has_more: bool,
}

/// Outbox イベント要素（API-sync-002 リクエスト）
#[derive(Debug, Deserialize, ToSchema)]
pub struct OutboxEventItem {
    pub outbox_event_id: Uuid,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub occurred_at: DateTime<Utc>,
}

/// ローカル Outbox 送信リクエスト（API-sync-002）
///
/// 端末から Outbox イベントを一括送信する（最大 100 件 / リクエスト）
#[derive(Debug, Deserialize, ToSchema)]
pub struct OutboxInboundRequest {
    /// 送信元工場 ID
    pub source_factory_id: Uuid,
    /// 送信するイベント一覧（最大 100 件）
    pub events: Vec<OutboxEventItem>,
}

/// Outbox 送信結果サマリ
#[derive(Debug, Serialize, ToSchema)]
pub struct OutboxInboundData {
    /// 受信成功件数
    pub accepted_count: i32,
    /// 重複スキップ件数（Idempotency-Key で既存と判定）
    pub skipped_count: i32,
}
