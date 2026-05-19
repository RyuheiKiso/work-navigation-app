// JWT ペイロードクレーム定義（MOD-BE-005）
// アルゴリズム: RS256（RSA 4096bit）、KEY-001
// 有効期限: 8 時間（CFG-005 / 28800 秒）
// aud クレームによるバイナリ種別判定: §1-1 参照

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// JWT ペイロードの Rust 表現。
///
/// | フィールド | 説明 |
/// |---|---|
/// | sub | ユーザー ID（UUID v7）|
/// | iss | 発行者: "wnav.factory.example" |
/// | aud | バイナリ種別: "terminal-api" または "master-api" |
/// | iat | 発行時刻（Unix 秒）|
/// | exp | 有効期限（Unix 秒、iat + 28800）|
/// | roles | ロール名リスト |
/// | factory_id | 工場 ID |
/// | device_id | 端末 ID（任意）|
/// | jti | JWT ID: 失効チェック用（UUID v7）|
/// | kid | 鍵ローテーション識別子（例: "2026-Q2"）|
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtClaims {
    /// Subject: `user_id`（UUID v7）
    pub sub: Uuid,
    /// Issuer: "wnav.factory.example"
    pub iss: String,
    /// Audience: バイナリ種別を示す文字列
    /// - `terminal-api`: `wnav_terminal_api` 宛て
    /// - `master-api`: `wnav_master_api` 宛て
    pub aud: String,
    /// Issued At（Unix 秒）
    pub iat: i64,
    /// Expiration（Unix 秒、iat + 28800 = 8 時間）
    pub exp: i64,
    /// ロール名リスト（例: `["operator"]`、`RoleId` の `snake_case` 表現）
    pub roles: Vec<String>,
    /// 工場 ID（UUID v7）
    pub factory_id: Uuid,
    /// 端末 ID（任意。terminal-api では必須、master-api では None）
    pub device_id: Option<Uuid>,
    /// JWT ID: 失効チェック用の一意識別子（UUID v7）
    pub jti: Uuid,
    /// 鍵 ID（ローテーション識別用。例: "2026-Q2"）
    pub kid: String,
}

/// JWT の発行に使用する入力コマンド。
/// ログインハンドラから `JwtKeyStore::issue()` に渡す。
#[derive(Debug)]
pub struct JwtIssueCmd {
    /// 発行対象ユーザー ID（UUID v7）
    pub user_id: Uuid,
    /// 付与するロール一覧（`RoleId` の `snake_case` 文字列）
    pub roles: Vec<String>,
    /// 工場 ID（UUID v7）
    pub factory_id: Uuid,
    /// 端末 ID（terminal-api で使用、master-api では None）
    pub device_id: Option<Uuid>,
    /// 鍵 ID（ローテーション識別用。例: "2026-Q2"）
    pub kid: String,
    /// 発行先バイナリの audience 文字列（§1-1 参照）
    /// - `"terminal-api"`: `wnav_terminal_api` 宛て
    /// - `"master-api"`: `wnav_master_api` 宛て
    pub audience: String,
}
