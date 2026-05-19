// マスタバージョン管理 DTO（API-master-001〜007）
//
// マスタバージョンの Draft 作成・編集・承認申請・承認・ロールバック・Dry-run の型。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// マスタバージョンの状態
#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MasterVersionStatus {
    /// 下書き（編集中）
    Draft,
    /// 承認待ち
    PendingApproval,
    /// 承認済み・公開中
    Published,
    /// ロールバック済み（アーカイブ）
    Archived,
}

/// マスタバージョン一覧エントリ（API-master-001）
#[derive(Debug, Serialize, ToSchema)]
pub struct MasterVersionSummary {
    /// バージョン ID（UUID v7）
    pub id: Uuid,
    /// バージョン番号（例: "v1.2.3"）
    pub version: String,
    /// 現在の状態
    pub status: MasterVersionStatus,
    /// 作成者ユーザー ID
    pub created_by: Uuid,
    /// 作成日時
    pub created_at: DateTime<Utc>,
    /// 最終更新日時
    pub updated_at: DateTime<Utc>,
    /// 説明文
    pub description: Option<String>,
}

/// マスタバージョン詳細レスポンス（API-master-001〜007）
#[derive(Debug, Serialize, ToSchema)]
pub struct MasterVersionResponse {
    /// バージョン ID（UUID v7）
    pub id: Uuid,
    /// バージョン番号（例: "v1.2.3"）
    pub version: String,
    /// 現在の状態
    pub status: MasterVersionStatus,
    /// マスタデータ本体（JSON 形式）
    pub data: serde_json::Value,
    /// 作成者ユーザー ID
    pub created_by: Uuid,
    /// 承認者ユーザー ID（承認後に設定される）
    pub approved_by: Option<Uuid>,
    /// 作成日時
    pub created_at: DateTime<Utc>,
    /// 最終更新日時
    pub updated_at: DateTime<Utc>,
    /// 説明文
    pub description: Option<String>,
}

/// Draft バージョン作成リクエスト（API-master-002）
///
/// MasterEditorRole 以上が必要。
#[derive(Debug, Deserialize, ToSchema)]
#[allow(dead_code)]
pub struct CreateMasterVersionRequest {
    /// ベースにするバージョン ID（None の場合は空の Draft を作成する）
    pub base_version_id: Option<Uuid>,
    /// マスタデータ本体（JSON 形式）
    pub data: serde_json::Value,
    /// このバージョンの説明
    pub description: Option<String>,
}

/// Draft バージョン編集リクエスト（API-master-003）
///
/// MasterEditorRole 以上・Draft 状態のみ編集可能。
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateMasterVersionRequest {
    /// 更新するマスタデータ本体（JSON 形式）
    pub data: serde_json::Value,
    /// 説明文の更新（None の場合は変更しない）
    pub description: Option<String>,
}

/// 承認申請リクエスト（API-master-004）
///
/// MasterEditorRole 以上・Draft → PendingApproval 状態遷移。
#[derive(Debug, Deserialize, ToSchema)]
#[allow(dead_code)]
pub struct SubmitVersionRequest {
    /// 承認申請コメント
    pub comment: Option<String>,
}

/// 承認・公開リクエスト（API-master-005）
///
/// ApproverRole 以上・PendingApproval → Published 状態遷移。
/// 申請者と異なるユーザーであることが必要。
#[derive(Debug, Deserialize, ToSchema)]
#[allow(dead_code)]
pub struct ApproveVersionRequest {
    /// 承認コメント
    pub comment: Option<String>,
}

/// ロールバックリクエスト（API-master-006）
///
/// AdminRole 必須。Published → Archived 状態遷移。
#[derive(Debug, Deserialize, ToSchema)]
pub struct RollbackVersionRequest {
    /// ロールバック理由（必須）
    pub reason: String,
}

/// Dry-run 結果（API-master-007）
///
/// 参照整合性確認の結果。エラーがあれば details に含まれる。
#[derive(Debug, Serialize, ToSchema)]
pub struct DryRunResult {
    /// 参照整合性に問題がない場合は true
    pub is_valid: bool,
    /// 検証エラーの詳細一覧
    pub errors: Vec<DryRunError>,
    /// 警告一覧（エラーではないが注意が必要な項目）
    pub warnings: Vec<String>,
}

/// Dry-run 検証エラー
#[derive(Debug, Serialize, ToSchema)]
pub struct DryRunError {
    /// エラー種別
    pub error_type: String,
    /// 影響を受けるフィールドまたはエンティティ
    pub field: String,
    /// エラーメッセージ
    pub message: String,
}

/// マスタバージョン一覧レスポンス（API-master-001）
#[derive(Debug, Serialize, ToSchema)]
pub struct MasterVersionListResponse {
    /// バージョン一覧
    pub items: Vec<MasterVersionSummary>,
    /// 総件数
    pub total: i64,
}
