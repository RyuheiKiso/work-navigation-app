// 機密値をラップして Debug 出力時に自動マスキングする
// 秘密情報がログやデバッグ出力に漏れる事故を型システムで防止する

use crate::schema::TerminalApiConfig;

/// 機密値（パスワード・鍵等）を保持するラッパー型
/// Debug 出力では常にマスキングされ、バイト数のみが表示される
#[derive(Clone, serde::Deserialize)]
pub struct Secret(String);

// デバッグ出力でも平文が表示されないようデフォルト実装を上書きする
impl std::fmt::Debug for Secret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // バイト数は公開することで設定値の有無と長さを確認できるようにする
        write!(f, "Secret(***REDACTED*** {} bytes)", self.0.len())
    }
}

impl Secret {
    /// 機密の実値を取得する
    /// 呼び出し元はこの値をログ・レスポンスボディ等に含めてはならない
    pub fn expose(&self) -> &str {
        &self.0
    }
}

/// TerminalApiConfig の非機密フィールドのみをログに出力する
/// 起動時の設定確認用。パスワード・鍵等は絶対に含めない
pub fn dump_config_summary(cfg: &TerminalApiConfig) {
    tracing::info!(
        schema_version = cfg.shared.schema_version,
        port = cfg.server.terminal_api.port,
        db_host = %cfg.database.host,
        db_max_conn = cfg.database.max_connections,
        log_level = %cfg.shared.observability.log_level,
        "configuration loaded"
    );
}
