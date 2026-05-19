// 設定値の意味的整合性を起動時に検証して fail-fast を実現する
// 型システムで表現できない制約（範囲・大小関係）をここで確認する

use crate::{
    error::ConfigError,
    schema::{MasterApiConfig, TerminalApiConfig},
};

/// ポート番号が有効範囲（1024 〜 65535）であることを検証する
/// Well-known ポート（< 1024）は root 権限が必要なため通常のアプリ用途では不適切
fn validate_port(port: u16, field: &str) -> Result<(), ConfigError> {
    if port < 1024 {
        return Err(ConfigError::InvalidValue {
            field: field.to_string(),
            reason: format!("port {port} is in the well-known range (< 1024). Use 1024-65535."),
        });
    }
    Ok(())
}

/// min_connections <= max_connections を検証する
/// 逆転していると接続プールが初期化時にパニックするため起動前に確認する
fn validate_pool_constraints(
    min_connections: u32,
    max_connections: u32,
    prefix: &str,
) -> Result<(), ConfigError> {
    if min_connections > max_connections {
        return Err(ConfigError::InvalidValue {
            field: format!("{prefix}.min_connections"),
            reason: format!(
                "min_connections ({min_connections}) > max_connections ({max_connections})"
            ),
        });
    }
    Ok(())
}

/// JWT TTL が最低 60 秒以上であることを検証する
/// 60 秒未満のトークンは有効期限切れが頻発して実用に耐えない
fn validate_jwt_ttl(ttl_sec: u64, field: &str) -> Result<(), ConfigError> {
    if ttl_sec < 60 {
        return Err(ConfigError::InvalidValue {
            field: field.to_string(),
            reason: format!("ttl_sec ({ttl_sec}) must be >= 60 seconds"),
        });
    }
    Ok(())
}

/// TerminalApiConfig の整合性を検証する
/// DB ロール・JWT TTL・ポートの妥当性を起動前に一括チェックする
pub fn validate_terminal(cfg: TerminalApiConfig) -> Result<TerminalApiConfig, ConfigError> {
    // ポート番号が有効範囲であることを確認する
    validate_port(cfg.server.terminal_api.port, "server.terminal_api.port")?;

    // 接続プールの min/max 整合性を確認する（逆転は sqlx の panic を引き起こす）
    validate_pool_constraints(
        cfg.database.min_connections,
        cfg.database.max_connections,
        "database",
    )?;

    // JWT TTL が最低限の実用値（60 秒）を満たしていることを確認する
    validate_jwt_ttl(cfg.shared.jwt_public.ttl_sec, "jwt_public.ttl_sec")?;

    Ok(cfg)
}

/// MasterApiConfig の整合性を検証する
/// DB ロール・JWT TTL・ポートの妥当性を起動前に一括チェックする
pub fn validate_master(cfg: MasterApiConfig) -> Result<MasterApiConfig, ConfigError> {
    // ポート番号が有効範囲であることを確認する
    validate_port(cfg.server.master_api.port, "server.master_api.port")?;

    // 接続プールの min/max 整合性を確認する
    validate_pool_constraints(
        cfg.database.min_connections,
        cfg.database.max_connections,
        "database",
    )?;

    // JWT TTL が最低限の実用値（60 秒）を満たしていることを確認する
    validate_jwt_ttl(cfg.shared.jwt_public.ttl_sec, "jwt_public.ttl_sec")?;

    Ok(cfg)
}
