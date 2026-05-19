// wnav_db エラー型定義
// sqlx エラー・マッピングエラー・接続エラーを統一的に扱う。

/// wnav_db クレートの統一エラー型。
#[derive(Debug, thiserror::Error)]
pub enum DbError {
    /// sqlx ドライバレイヤーのエラー
    #[error("sqlx エラー: {0}")]
    Sqlx(#[from] sqlx::Error),

    /// DB 行から Domain モデルへのマッピングに失敗した
    #[error("マッピングエラー: {0}")]
    Mapping(String),

    /// DB 接続の確立に失敗した
    #[error("接続失敗: {0}")]
    Connection(String),
}

/// DbError を DomainError に変換する。
/// Infrastructure 層のエラーを Domain 層の統一エラー型に昇格させる。
impl From<DbError> for wnav_domain::error::DomainError {
    fn from(e: DbError) -> Self {
        wnav_domain::error::DomainError::Internal(e.to_string())
    }
}

/// sqlx::Error を DomainError に変換するヘルパー。
/// リポジトリ実装の .map_err(map_sqlx) で使用する。
pub fn map_sqlx(e: sqlx::Error) -> wnav_domain::error::DomainError {
    wnav_domain::error::DomainError::from(DbError::Sqlx(e))
}
