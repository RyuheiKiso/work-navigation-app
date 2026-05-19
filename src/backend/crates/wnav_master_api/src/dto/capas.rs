// CAPA DTO（API-capa-001〜002）
//
// CAPA（是正処置・予防処置）の作成・更新エンドポイントの Request/Response 型。
// quality_admin / system_admin のみ作成可。

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// CAPA 作成リクエスト（POST /api/v1/capas）
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct CreateCapaRequest {
    /// 起因となった非適合品 ID（TBL-013、任意）
    pub nonconformity_id: Option<Uuid>,
    /// CAPA タイトル（1〜200 文字）
    pub title: String,
    /// 根本原因分析（1〜5000 文字）
    pub root_cause_analysis: String,
    /// 是正処置内容（1〜5000 文字）
    pub corrective_action: String,
    /// 再発防止処置内容（最大 5000 文字）
    pub preventive_action: Option<String>,
    /// 担当者 ID（TBL-016）
    pub assigned_to: Uuid,
    /// 完了予定日（ISO 8601 date）
    pub due_date: NaiveDate,
    /// 作成者 ID（quality_admin ロール必須）
    pub created_by: Uuid,
    /// クライアント側の作成時刻
    pub timestamp_client: DateTime<Utc>,
}

/// CAPA 更新リクエスト（PATCH /api/v1/capas/{id}）
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct UpdateCapaRequest {
    /// 新しいステータス（open → in_progress → pending_verification → closed）
    pub status: Option<String>,
    /// 進捗メモ（最大 2000 文字）
    pub progress_note: Option<String>,
    /// 是正処置内容の更新（最大 5000 文字）
    pub corrective_action: Option<String>,
    /// 防止処置内容の更新（最大 5000 文字）
    pub preventive_action: Option<String>,
    /// 完了予定日の変更
    pub due_date: Option<NaiveDate>,
    /// 更新者 ID（必須。TBL-016 に存在すること）
    pub updated_by: Uuid,
    /// クライアント側の更新時刻
    pub timestamp_client: DateTime<Utc>,
}

/// CAPA レスポンス
#[derive(Debug, Serialize, ToSchema)]
pub struct CapaResponse {
    pub capa_id: Uuid,
    pub status: String,
    pub title: String,
    pub nonconformity_id: Option<Uuid>,
    pub assigned_to: Uuid,
    pub due_date: NaiveDate,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
