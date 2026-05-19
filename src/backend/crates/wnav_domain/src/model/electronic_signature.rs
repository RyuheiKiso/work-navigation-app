// 電子サインのドメインモデル
// 作業記録の電子署名を管理する。BR-BUS-004 電子サイン必須ステップで使用する。
// 電子サインは改ざん防止のため verified フラグで検証状態を管理する。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 電子サインエンティティ。
/// 電子サイン必須ステップ（BR-BUS-004）で記録する。
/// verified フラグで署名の有効性を管理する。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElectronicSignature {
    /// サイン ID（UUID v7）
    pub sign_id: Uuid,
    /// 関連する作業実行 ID
    pub work_execution_id: Uuid,
    /// 関連するステップ ID
    pub step_id: Uuid,
    ///署名者 ID
    pub signer_id: Uuid,
    /// 署名データ（Base64 エンコード等）
    pub signature_data: String,
    /// 署名日時
    pub signed_at: DateTime<Utc>,
    /// 検証済みフラグ（サーバー側で署名を検証後に true に設定する）
    pub verified: bool,
}
