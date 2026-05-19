// 設定読み込み時に発生しうるすべてのエラーを一元管理する
// thiserror を使い人間が読めるエラーメッセージを自動生成する

/// 設定読み込み・解決・検証で発生するエラー
///
/// `figment::Error` は 208 バイト超のため `Box` でラップしてスタックサイズを抑える
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    // `WNAV_PROFILE` 環境変数が未設定の場合は起動を即座に失敗させる
    #[error("WNAV_PROFILE is not set. Must be one of: local, dev, staging, prod")]
    ProfileNotSet,

    // 必須 YAML ファイルが存在しない場合はパスを示して原因を明確化する
    #[error("Config file not found: {path}")]
    FileNotFound { path: String },

    // `schema_version` の不一致はコードと設定の乖離を示す致命的問題なので明示する
    #[error("Config schema version mismatch: expected {expected}, got {got}")]
    SchemaVersionMismatch { expected: u32, got: u32 },

    // figment のデシリアライズエラーをボックス化してスタックサイズを抑える
    // `figment::Error` は 208 バイト超であり Result のスタックサイズを過大にするため
    #[error("Failed to extract config: {0}")]
    Extract(Box<figment::Error>),

    // `secret_ref` の解決失敗は起動 fail-fast に使うため独立したバリアントにする
    #[error("Failed to resolve secret_ref '{secret_ref}': {reason}")]
    SecretRefResolution { secret_ref: String, reason: String },

    // バリデーションエラーはフィールド名と理由をセットで報告してデバッグを容易にする
    #[error("Invalid config value for '{field}': {reason}")]
    InvalidValue { field: String, reason: String },
}

// figment::Error を直接 Box<figment::Error> に変換できるようにする
// これにより `?` 演算子が自動的に Box 化してから ConfigError::Extract に変換できる
impl From<figment::Error> for ConfigError {
    fn from(e: figment::Error) -> Self {
        Self::Extract(Box::new(e))
    }
}
