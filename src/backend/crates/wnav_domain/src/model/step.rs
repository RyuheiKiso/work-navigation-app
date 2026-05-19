// SOP ステップのドメインモデル（EN-008）
// SOP 内の単一手順ステップを表す。ロックステップ強制（BR-BUS-001）の適用対象。

use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

/// SOP 内の単一手順ステップ（EN-008）。
/// ロックステップ強制（BR-BUS-001）の適用対象。
/// condition_dsl には JSON Logic ルール（ALG-004/005）を格納する。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Step {
    /// ステップ ID（UUID v7）
    pub step_id: Uuid,
    /// 所属 SOP ID
    pub sop_id: Uuid,
    /// 1 基準のステップ番号（表示順）
    pub step_number: u32,
    /// ステップタイトル
    pub title: String,
    /// 作業指示テキスト（Markdown 可）
    pub instruction: String,
    /// JSON Logic 条件式（スキップ条件・分岐条件。ALG-004/005 で評価する）
    pub condition_dsl: Option<Value>,
    /// 証拠記録が必須かどうか（BR-BUS-003）
    pub evidence_required: bool,
    /// 電子サインが必須かどうか（BR-BUS-004）
    pub sign_required: bool,
    /// スキップ可能かどうか（false の場合 BR-BUS-001 で強制）
    pub skippable: bool,
    /// 推定作業時間（秒。UI 表示用）
    pub estimated_duration_secs: Option<u32>,
    /// ステップ種別
    pub step_type: StepType,
}

/// ステップの種別。
/// ステップ種別ごとに必須属性が異なる（src/CLAUDE.md XES 互換イベント必須属性）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StepType {
    /// 標準手順ステップ
    Standard,
    /// クリティカルステップ（重点管理）
    Critical,
    /// 測定ステップ（数値入力必須）
    Measurement,
    /// 電子サインステップ
    Signature,
    /// QR コードスキャンステップ
    QrScan,
    /// 証拠撮影ステップ
    Evidence,
    /// カスタムステップ（アドオン機構対応）
    Custom,
}
