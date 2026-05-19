// CAPA（是正・予防措置）のドメインモデル
// アンドン・逸脱から発生した是正・予防処置を管理する。
// Investigation → Corrective → Preventive → Closed のフェーズで進行する。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

/// CAPA（是正・予防措置）エンティティ。
/// アンドンまたは逸脱から生成される。
/// 根本原因分析（root_cause_json）は JSONB で柔軟なデータ構造を許容する。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capa {
    /// CAPA ID（UUID v7）
    pub capa_id: Uuid,
    /// 関連するアンドン ID（アンドン起因の場合）
    pub andon_id: Option<Uuid>,
    /// 関連する逸脱 ID（逸脱起因の場合）
    pub deviation_id: Option<Uuid>,
    /// 現在フェーズ
    pub phase: CapaPhase,
    /// 担当者 ID
    pub assignee: Uuid,
    /// 期限日
    pub due_date: Option<DateTime<Utc>>,
    /// 問題説明
    pub description: String,
    /// 根本原因分析結果（JSONB。5Why・FTA 等の分析結果を格納）
    pub root_cause_json: Option<Value>,
    /// 是正処置の内容
    pub corrective_action: Option<String>,
    /// クローズ日時（Closed フェーズで設定）
    pub closed_at: Option<DateTime<Utc>>,
}

/// CAPA フェーズ。
/// Investigation → Corrective → Preventive → Closed の順で進行する。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CapaPhase {
    /// 調査フェーズ（根本原因分析）
    Investigation,
    /// 是正フェーズ（問題の修正）
    Corrective,
    /// 予防フェーズ（再発防止）
    Preventive,
    /// クローズ（完了）
    Closed,
}
