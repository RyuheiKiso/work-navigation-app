// bcrypt パスワードハッシュ化・検証モジュール（MOD-BE-005 §5）
// LDAP 不可時のローカル認証フォールバック（BR-BUS: LDAP フォールバック）で使用する。
// bcrypt cost は 12 以上を使用してブルートフォース耐性を確保する。

use crate::error::AuthError;

/// bcrypt コスト係数（デフォルト 12: 十分な計算コストと実用的な速度のバランス）
const BCRYPT_COST: u32 = 12;

/// 平文パスワードを bcrypt でハッシュ化して返す。
///
/// # 引数
/// - `plain`: ハッシュ化対象の平文パスワード
///
/// # 戻り値
/// - bcrypt ハッシュ文字列（例: "$2b$12$..."）
#[tracing::instrument(skip(plain), err)]
pub fn hash_password(plain: &str) -> Result<String, AuthError> {
    // bcrypt でパスワードをハッシュ化する（cost=12 でブルートフォース耐性を確保）
    bcrypt::hash(plain, BCRYPT_COST).map_err(|e| AuthError::PasswordHashError(e.to_string()))
}

/// bcrypt ハッシュに対して平文パスワードを検証する。
///
/// # 引数
/// - `plain`: 検証対象の平文パスワード
/// - `hash`: bcrypt ハッシュ文字列（DB から取得）
///
/// # 戻り値
/// - `true`: パスワードが一致する
/// - `false`: パスワードが不一致
#[tracing::instrument(skip(plain, hash), err)]
pub fn verify_password(plain: &str, hash: &str) -> Result<bool, AuthError> {
    // bcrypt で平文とハッシュを照合する（定数時間比較で timing attack を防止）
    bcrypt::verify(plain, hash).map_err(|e| AuthError::PasswordHashError(e.to_string()))
}
