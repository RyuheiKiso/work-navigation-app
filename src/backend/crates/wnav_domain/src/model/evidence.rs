// 証拠ファイルのドメインモデル
// 作業記録の証拠（写真・測定値・QR スキャン・電子サイン等）を管理する。
// file_hash はバイナリ整合性の確認に使用する（ALCOA+ Accurate 原則）。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 証拠ファイルエンティティ。
/// 証拠種別ごとに必須属性が異なる（ステップ種別と連動）。
/// file_hash（SHA-256）でファイル整合性を保証する。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceFile {
    /// 証拠 ID（UUID v7）
    pub evidence_id: Uuid,
    /// 関連する作業実行 ID
    pub work_execution_id: Uuid,
    /// 関連するステップ ID
    pub step_id: Uuid,
    /// ファイルの SHA-256 ハッシュ（32 バイト。ファイル改ざん検出用）
    pub file_hash: [u8; 32],
    /// ファイル保存パス
    pub file_path: String,
    /// 証拠種別
    pub evidence_type: EvidenceType,
    /// 記録者 ID
    pub recorded_by: Uuid,
    /// クライアント記録日時（申告値）
    pub client_recorded_at: DateTime<Utc>,
    /// サーバー受信日時（権威タイムスタンプ）
    pub server_received_at: DateTime<Utc>,
}

/// 証拠ファイルの種別。
/// ステップ種別（StepType）と対応関係を持つ。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EvidenceType {
    /// 写真（Evidence ステップ）
    Photo,
    /// 測定値（Measurement ステップ）
    Measurement,
    /// QR コードスキャン結果（QrScan ステップ）
    QrScan,
    /// 電子サイン（Signature ステップ）
    Signature,
    /// その他
    Other,
}
