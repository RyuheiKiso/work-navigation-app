// 電子サイン API（API-electronic-signs-001〜003）の DTO 定義（04_エビデンス・電子サインAPI仕様.md §2〜4）

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// 電子サイン作成リクエスト（API-electronic-signs-001）
#[derive(Debug, Deserialize, ToSchema)]
pub struct ElectronicSignatureRequest {
    /// 署名者ユーザー ID（TBL-016 に存在すること）
    pub signer_id: Uuid,
    /// 署名対象コンテンツの SHA-256（"sha256:" プレフィックス + hex 64 文字）
    pub signed_content_hash: String,
    /// PIN の bcrypt ハッシュ（クライアント側でハッシュ化済み）
    pub pin_hash: String,
    /// 署名コンテキスト種別（step_sign / work_complete_sign / approval_sign / quality_check_sign）
    pub context_type: String,
    /// 署名対象リソース ID（context_type に対応するレコードが存在すること）
    pub context_id: Uuid,
    /// 対象ステップ ID（context_type が step_sign の場合は必須）
    pub step_id: Option<Uuid>,
    /// クライアント側の署名時刻
    pub timestamp_client: DateTime<Utc>,
    /// 端末秘密鍵（Ed25519）による本文署名（base64 エンコード）
    pub device_signature: String,
}

/// 電子サイン作成レスポンスの data フィールド（API-electronic-signs-001）
#[derive(Debug, Serialize, ToSchema)]
pub struct ElectronicSignatureData {
    /// 電子サイン ID（TBL-002）
    pub sign_id: Uuid,
    /// 署名者 ID
    pub signer_id: Uuid,
    /// 署名対象コンテンツの SHA-256
    pub signed_content_hash: String,
    /// 署名コンテキスト種別
    pub context_type: String,
    /// 署名対象リソース ID
    pub context_id: Uuid,
    /// サーバー側の署名時刻（権威タイムスタンプ）
    pub signed_at: DateTime<Utc>,
    /// ハッシュチェーンブロック ID（TBL-031）
    pub hash_chain_block_id: Option<Uuid>,
    /// 今回ブロックのハッシュ値
    pub hash_chain_value: Option<String>,
}

/// 電子サイン取得レスポンスの data フィールド（API-electronic-signs-002）
#[derive(Debug, Serialize, ToSchema)]
pub struct ElectronicSignatureDetailData {
    pub sign_id: Uuid,
    pub signer_id: Uuid,
    pub signer_name: Option<String>,
    pub signer_role: Option<String>,
    pub signed_content_hash: String,
    pub context_type: String,
    pub context_id: Uuid,
    pub step_id: Option<Uuid>,
    pub signed_at: DateTime<Utc>,
    pub hash_chain_block_id: Option<Uuid>,
    pub hash_chain_value: Option<String>,
    /// 前ブロックのハッシュ値（連鎖性確認用）
    pub hash_chain_prev: Option<String>,
    /// ハッシュチェーン検証ステータス（valid / hash_chain_broken / device_key_revoked）
    pub verification_status: String,
    pub device_id: Option<Uuid>,
}

/// 電子サイン一覧取得のクエリパラメータ（API-electronic-signs-003）
#[derive(Debug, Deserialize, ToSchema)]
pub struct ElectronicSignatureQuery {
    pub work_execution_id: Option<Uuid>,
    pub signer_id: Option<Uuid>,
    pub context_type: Option<String>,
    pub signed_from: Option<DateTime<Utc>>,
    pub signed_to: Option<DateTime<Utc>>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}
