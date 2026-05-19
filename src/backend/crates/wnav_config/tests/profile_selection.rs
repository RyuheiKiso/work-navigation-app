// WNAV_PROFILE 環境変数によるプロファイル選択が正しく機能することを検証する
// tempfile で一時 YAML ファイルを作成して実ファイルシステムへの依存を排除する

// テストコードでは環境変数操作のために unsafe を許容する
// Edition 2024 以降 set_var / remove_var は unsafe 関数になったため

use std::path::PathBuf;
use tempfile::TempDir;
use wnav_config::profile::Profile;

/// テスト用の設定ディレクトリと YAML ファイルを作成するヘルパー
fn setup_config_dir(profile: &str, profile_override: &str) -> (TempDir, PathBuf) {
    let dir = TempDir::new().expect("TempDir 作成に失敗");
    let dir_path = dir.path().to_path_buf();

    // 最低限有効な base YAML を作成する（schema_version は必須）
    let base_yaml = r"
schema_version: 1
profile: base
server:
  terminal_api:
    bind_addr: '0.0.0.0'
    port: 8080
    request_timeout_sec: 30
  master_api:
    bind_addr: '0.0.0.0'
    port: 8081
    request_timeout_sec: 30
database:
  host: 'localhost'
  port: 5432
  name: 'wnav_test'
  ssl_mode: 'disable'
  max_connections: 10
  min_connections: 1
  acquire_timeout_sec: 5
  idle_timeout_sec: 300
  max_lifetime_sec: 1800
  event_insert:
    user: 'wnav_event_insert'
    password: 'test_password_event_insert'
  write:
    user: 'wnav_write'
    password: 'test_password_write'
  read:
    user: 'wnav_read'
    password: 'test_password_read'
cors:
  allow_origins:
    - 'http://localhost:5173'
  allow_credentials: true
  max_age_sec: 3600
jwt_public:
  algorithm: 'RS256'
  ttl_sec: 28800
  public_key: 'test_public_key'
jwt_private:
  private_key: 'test_private_key'
idempotency:
  ttl_sec: 86400
outbox:
  interval_ms: 5000
  retry_max: 5
  backoff_max_sec: 300
hash_chain_verify:
  cron: '0 2 * * *'
rate_limit:
  rps: 100
  burst: 200
observability:
  log_level: 'info'
  log_format: 'json'
  request_id_header: 'X-Request-Id'
  metrics_enabled: true
  metrics_path: '/metrics'
webhook:
  signature_header: 'X-Signature-256'
  retry_max: 5
  retry_backoff_max_sec: 86400
  hmac_key: 'test_hmac_key'
webhook_receiver:
  hmac_key: 'test_hmac_key_recv'
  hmac_timeout_ms: 5000
sse:
  keep_alive_sec: 25
  dispatch_retry_max: 5
integration:
  push_receive_enabled: false
external:
  backup_notification_url: 'http://example.com/notify'
frontend_master:
  api_base_url: 'http://localhost:8081'
  openapi_url: 'http://localhost:8081/api/openapi.json'
  session_timeout_min: 30
  polling_interval_ms: 30000
";

    // プロファイル固有の上書き YAML を作成する
    let profile_yaml = format!(
        r"
schema_version: 1
profile: {profile}
{profile_override}
"
    );

    std::fs::write(dir_path.join("config.base.yml"), base_yaml)
        .expect("config.base.yml 書き込みに失敗");
    std::fs::write(
        dir_path.join(format!("config.{profile}.yml")),
        profile_yaml,
    )
    .expect("config.{profile}.yml 書き込みに失敗");

    (dir, dir_path)
}

#[test]
fn test_profile_local_is_selected_when_env_is_local() {
    // WNAV_PROFILE=local を設定したとき Profile::Local が返ることを確認する
    // Edition 2024 では set_var / remove_var は unsafe 関数
    unsafe {
        std::env::set_var("WNAV_PROFILE", "local");
    }
    let profile = Profile::from_env().expect("Profile::from_env に失敗");
    assert_eq!(profile, Profile::Local);
    unsafe {
        std::env::remove_var("WNAV_PROFILE");
    }
}

#[test]
fn test_profile_dev_is_selected_when_env_is_dev() {
    unsafe {
        std::env::set_var("WNAV_PROFILE", "dev");
    }
    let profile = Profile::from_env().expect("Profile::from_env に失敗");
    assert_eq!(profile, Profile::Dev);
    unsafe {
        std::env::remove_var("WNAV_PROFILE");
    }
}

#[test]
fn test_profile_staging_is_selected_when_env_is_staging() {
    unsafe {
        std::env::set_var("WNAV_PROFILE", "staging");
    }
    let profile = Profile::from_env().expect("Profile::from_env に失敗");
    assert_eq!(profile, Profile::Staging);
    unsafe {
        std::env::remove_var("WNAV_PROFILE");
    }
}

#[test]
fn test_profile_prod_is_selected_when_env_is_prod() {
    unsafe {
        std::env::set_var("WNAV_PROFILE", "prod");
    }
    let profile = Profile::from_env().expect("Profile::from_env に失敗");
    assert_eq!(profile, Profile::Prod);
    unsafe {
        std::env::remove_var("WNAV_PROFILE");
    }
}

#[test]
fn test_profile_not_set_error_when_env_is_missing() {
    // WNAV_PROFILE が未設定のとき ProfileNotSet エラーが返ることを確認する
    unsafe {
        std::env::remove_var("WNAV_PROFILE");
    }
    let result = Profile::from_env();
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        wnav_config::ConfigError::ProfileNotSet
    ));
}

#[test]
fn test_profile_not_set_error_when_env_is_unknown() {
    // 不明なプロファイル名は ProfileNotSet として扱うことを確認する
    unsafe {
        std::env::set_var("WNAV_PROFILE", "unknown_profile");
    }
    let result = Profile::from_env();
    assert!(result.is_err());
    unsafe {
        std::env::remove_var("WNAV_PROFILE");
    }
}

#[test]
fn test_profile_case_insensitive() {
    // プロファイル名は大文字小文字を区別しないことを確認する
    unsafe {
        std::env::set_var("WNAV_PROFILE", "LOCAL");
    }
    let profile = Profile::from_env().expect("Profile::from_env に失敗");
    assert_eq!(profile, Profile::Local);
    unsafe {
        std::env::remove_var("WNAV_PROFILE");
    }
}

#[test]
fn test_config_dir_is_used_when_env_is_set() {
    // WNAV_CONFIG_DIR が設定されている場合にそのディレクトリを使うことを確認する
    let (_dir, dir_path) = setup_config_dir("local", "");

    unsafe {
        std::env::set_var("WNAV_PROFILE", "local");
        std::env::set_var("WNAV_CONFIG_DIR", dir_path.to_str().unwrap());
    }

    // build_figment が成功することを確認（schema_version 検証も通過する）
    let result = wnav_config::sources::build_figment();
    assert!(result.is_ok(), "build_figment failed: {:?}", result.err());

    unsafe {
        std::env::remove_var("WNAV_PROFILE");
        std::env::remove_var("WNAV_CONFIG_DIR");
    }
}
