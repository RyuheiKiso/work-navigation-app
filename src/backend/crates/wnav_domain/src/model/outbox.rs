// Outbox パターンのドメインモデル
// イベントの確実な配信を保証する Transactional Outbox パターンの実装モデル。
// Pending → Processing → Sent / Failed → DeadLettered の状態遷移で管理する。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

/// Outbox イベントエンティティ。
/// Transactional Outbox パターンで外部システムへの確実な配信を保証する。
/// 作業イベント記録と同一トランザクションで INSERT し、
/// Outbox Worker が非同期でリトライ配信する（ALG-004 Outbox リトライ）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutboxEvent {
    /// Outbox イベント ID（UUID v7）
    pub outbox_id: Uuid,
    /// 参照するドメインイベント ID
    pub event_id: Uuid,
    /// 冪等性キー（ドメインイベントの event_id と同一）
    pub idempotency_key: Uuid,
    /// イベント種別（Webhook ペイロードの型を特定するために使用）
    pub event_type: String,
    /// イベントペイロード（JSONB）
    pub payload: Value,
    /// 配信ステータス
    pub status: OutboxStatus,
    /// リトライ回数（最大 5 回。指数バックオフ）
    pub retry_count: u32,
    /// 最終試行日時
    pub last_attempted_at: Option<DateTime<Utc>>,
}

/// Outbox イベントの配信ステータス。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OutboxStatus {
    /// 配信待ち
    Pending,
    /// 配信処理中（Outbox Worker が処理中）
    Processing,
    /// 配信成功
    Sent,
    /// 配信失敗（リトライ中）
    Failed,
    /// デッドレター（最大リトライ超過。手動対応が必要）
    DeadLettered,
}
