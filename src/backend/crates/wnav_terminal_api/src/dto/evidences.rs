// エビデンス API（API-evidences-001）の DTO 定義（04_エビデンス・電子サインAPI仕様.md §1）

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// エビデンスアップロード時の metadata パート（multipart/form-data）
#[derive(Debug, Deserialize, ToSchema)]
pub struct EvidenceMetadata {
    /// 対象作業実行 ID（TBL-005 に in_progress で存在すること）
    pub work_execution_id: Uuid,
    /// 対象ステップ ID（TBL-008 に存在すること）
    pub step_id: Uuid,
    /// エビデンス種別（photo / document / measurement_sheet）
    pub evidence_type: String,
    /// 説明（任意、最大 500 文字）
    pub description: Option<String>,
    /// クライアント側の撮影・生成時刻
    pub timestamp_client: DateTime<Utc>,
    /// クライアントが計算したファイルの SHA-256 ハッシュ（hex 64 文字）
    pub sha256_client: String,
}

/// エビデンスアップロードレスポンスの data フィールド（API-evidences-001）
#[derive(Debug, Serialize, ToSchema)]
pub struct EvidenceData {
    /// エビデンス ID（TBL-009）
    pub evidence_id: Uuid,
    /// SHA-256 ハッシュ（hex、サーバー側で検証済み）
    pub file_hash_sha256: String,
    /// サーバー上の保存パス（相対）
    pub file_path: String,
    /// ファイルサイズ（バイト）
    pub file_size_bytes: i64,
    /// エビデンス種別
    pub evidence_type: String,
    /// 画像幅（px）、画像以外は null
    pub width_px: Option<i32>,
    /// 画像高さ（px）、画像以外は null
    pub height_px: Option<i32>,
    /// 対象作業実行 ID
    pub work_execution_id: Uuid,
    /// 対象ステップ ID
    pub step_id: Uuid,
    /// アップロードしたユーザー ID
    pub uploaded_by: Uuid,
    /// アップロード時刻（サーバー側権威タイムスタンプ）
    pub uploaded_at: DateTime<Utc>,
}
