// トレサビ DTO（API-trace-001〜002）
//
// ケーストレース（順方向）とロットトレース（逆方向）の Response 型。

use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

/// ケーストレースイベント
#[derive(Debug, Serialize, ToSchema)]
pub struct TraceEvent {
    /// イベント ID
    pub id: Uuid,
    /// Case ID
    pub case_id: Uuid,
    /// アクティビティ種別
    pub activity: String,
    /// サーバー受信時刻（UTC ms）
    pub server_received_at: i64,
    /// クライアント記録時刻（UTC ms）
    pub client_recorded_at: Option<i64>,
    /// 作業者 ID
    pub worker_id: Uuid,
    /// 端末 ID
    pub device_id: Option<Uuid>,
    /// ハッシュチェーン整合性（true: 正常、false: 破断）
    pub hash_valid: bool,
    /// イベントペイロード
    pub payload: serde_json::Value,
    /// タイムスタンプ
    pub created_at: DateTime<Utc>,
}

/// ケーストレースレスポンス（API-trace-001）
///
/// 指定された case_id に関連するすべてのイベントを時系列順で返す（順方向トレース）。
#[derive(Debug, Serialize, ToSchema)]
pub struct CaseTraceResponse {
    /// Case ID
    pub case_id: Uuid,
    /// イベント一覧（時系列昇順）
    pub events: Vec<TraceEvent>,
    /// ハッシュチェーン検証結果（true: 全件正常）
    pub chain_integrity: bool,
    /// 破断が検知されたイベント ID 一覧（空の場合は問題なし）
    pub broken_event_ids: Vec<Uuid>,
}

/// ロットトレースノード
#[derive(Debug, Serialize, ToSchema)]
pub struct LotTraceNode {
    /// ロット ID
    pub lot_id: String,
    /// ロット種別（"material", "product", "sub-assembly" 等）
    pub lot_type: String,
    /// このロットに関連する Case ID 一覧
    pub case_ids: Vec<Uuid>,
    /// 上流ロット（原材料方向）
    pub upstream_lots: Vec<String>,
    /// 下流ロット（製品方向）
    pub downstream_lots: Vec<String>,
    /// 工程情報
    pub process_id: Option<String>,
    /// 処理日時範囲
    pub processed_from: Option<DateTime<Utc>>,
    /// 処理日時範囲
    pub processed_to: Option<DateTime<Utc>>,
}

/// ロットトレースレスポンス（API-trace-002）
///
/// 指定された lot_id から遡る逆方向トレース結果を返す。
#[derive(Debug, Serialize, ToSchema)]
pub struct LotTraceResponse {
    /// クエリしたロット ID
    pub lot_id: String,
    /// トレース深度（何段階遡ったか）
    pub depth: u32,
    /// ノード一覧（ロットグラフの全ノード）
    pub nodes: Vec<LotTraceNode>,
    /// 不適合が検知されたロット ID 一覧
    pub nonconformance_lot_ids: Vec<String>,
}
