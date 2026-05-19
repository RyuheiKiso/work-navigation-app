// fail-fast 設計: 設定エラーが発生したとき適切な ConfigError が返ることを検証する
// WNAV_PROFILE 未設定 / schema_version 不一致 / secret_ref 解決失敗 の 3 ケースを確認する

// テストコードで環境変数操作のために unsafe を許容する（Edition 2024 の仕様）

use wnav_config::ConfigError;

/// テスト用 YAML を一時ディレクトリに書き込むヘルパー
fn write_config(dir: &tempfile::TempDir, filename: &str, content: &str) {
    std::fs::write(dir.path().join(filename), content)
        .unwrap_or_else(|e| panic!("{filename} 書き込みに失敗: {e}"));
}

// ────────────────────────────────────────────────────────────────
// 1. WNAV_PROFILE 未設定
// ────────────────────────────────────────────────────────────────

#[test]
fn test_fail_fast_when_wnav_profile_not_set() {
    // WNAV_PROFILE が未設定のとき ProfileNotSet エラーが返ることを確認する
    unsafe {
        std::env::remove_var("WNAV_PROFILE");
    }

    let result = wnav_config::sources::build_figment();
    assert!(result.is_err(), "WNAV_PROFILE 未設定でも成功してしまった");

    let err = result.unwrap_err();
    assert!(
        matches!(err, ConfigError::ProfileNotSet),
        "期待するエラー種別ではない: {err:?}"
    );
}

#[test]
fn test_fail_fast_load_terminal_api_when_profile_not_set() {
    // load_terminal_api() でも ProfileNotSet エラーが返ることを確認する
    unsafe {
        std::env::remove_var("WNAV_PROFILE");
    }

    let result = wnav_config::load_terminal_api();
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ConfigError::ProfileNotSet));
}

#[test]
fn test_fail_fast_load_master_api_when_profile_not_set() {
    // load_master_api() でも ProfileNotSet エラーが返ることを確認する
    unsafe {
        std::env::remove_var("WNAV_PROFILE");
    }

    let result = wnav_config::load_master_api();
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ConfigError::ProfileNotSet));
}

// ────────────────────────────────────────────────────────────────
// 2. schema_version 不一致
// ────────────────────────────────────────────────────────────────

#[test]
fn test_fail_fast_when_schema_version_mismatches() {
    // schema_version が 1 以外のとき SchemaVersionMismatch エラーが返ることを確認する
    let dir = tempfile::TempDir::new().expect("TempDir 作成に失敗");

    // schema_version: 2 を持つ不正な YAML を作成する
    write_config(
        &dir,
        "config.base.yml",
        "schema_version: 2\ndatabase:\n  host: 'localhost'\n",
    );
    write_config(&dir, "config.local.yml", "schema_version: 2\n");

    unsafe {
        std::env::set_var("WNAV_PROFILE", "local");
        std::env::set_var("WNAV_CONFIG_DIR", dir.path().to_str().unwrap());
    }

    let result = wnav_config::sources::build_figment();
    assert!(result.is_err(), "schema_version 不一致でも成功してしまった");

    let err = result.unwrap_err();
    assert!(
        matches!(
            err,
            ConfigError::SchemaVersionMismatch {
                expected: 1,
                got: 2,
            }
        ),
        "期待するエラー種別ではない: {err:?}"
    );

    unsafe {
        std::env::remove_var("WNAV_PROFILE");
        std::env::remove_var("WNAV_CONFIG_DIR");
    }
}

#[test]
fn test_fail_fast_when_schema_version_is_missing() {
    // schema_version キーが存在しない（= 0 として扱う）場合もエラーになることを確認する
    let dir = tempfile::TempDir::new().expect("TempDir 作成に失敗");

    write_config(&dir, "config.base.yml", "database:\n  host: 'localhost'\n");
    write_config(&dir, "config.local.yml", "database:\n  host: 'localhost'\n");

    unsafe {
        std::env::set_var("WNAV_PROFILE", "local");
        std::env::set_var("WNAV_CONFIG_DIR", dir.path().to_str().unwrap());
    }

    let result = wnav_config::sources::build_figment();
    assert!(result.is_err(), "schema_version 欠損でも成功してしまった");

    let err = result.unwrap_err();
    assert!(
        matches!(
            err,
            ConfigError::SchemaVersionMismatch {
                expected: 1,
                got: 0,
            }
        ),
        "期待するエラー種別ではない: {err:?}"
    );

    unsafe {
        std::env::remove_var("WNAV_PROFILE");
        std::env::remove_var("WNAV_CONFIG_DIR");
    }
}

// ────────────────────────────────────────────────────────────────
// 3. secret_ref 解決失敗
// ────────────────────────────────────────────────────────────────

#[test]
fn test_fail_fast_when_secret_ref_env_not_found() {
    // 存在しない環境変数を参照する secret_ref は SecretRefResolution エラーになることを確認する
    let dir = tempfile::TempDir::new().expect("TempDir 作成に失敗");

    // 存在しない環境変数を参照する YAML を作成する
    let base_yaml = r"
schema_version: 1
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
    password:
      secret_ref: 'env:WNAV_DEFINITELY_NOT_EXISTS_SECRET_XYZ_ABC_123'
  write:
    user: 'wnav_write'
    password: 'pass'
  read:
    user: 'wnav_read'
    password: 'pass'
cors:
  allow_origins:
    - 'http://localhost:5173'
  allow_credentials: true
  max_age_sec: 3600
jwt_public:
  algorithm: 'RS256'
  ttl_sec: 28800
  public_key: 'pub_key'
jwt_private:
  private_key: 'priv_key'
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
  hmac_key: 'hmac'
webhook_receiver:
  hmac_key: 'recv_hmac'
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

    write_config(&dir, "config.base.yml", base_yaml);
    write_config(&dir, "config.local.yml", "schema_version: 1\n");

    // 参照する環境変数が存在しないことを確認する
    unsafe {
        std::env::remove_var("WNAV_DEFINITELY_NOT_EXISTS_SECRET_XYZ_ABC_123");
        std::env::set_var("WNAV_PROFILE", "local");
        std::env::set_var("WNAV_CONFIG_DIR", dir.path().to_str().unwrap());
    }

    let result = wnav_config::sources::build_figment();
    assert!(
        result.is_err(),
        "secret_ref 解決失敗でも build_figment が成功してしまった"
    );

    let err = result.unwrap_err();
    assert!(
        matches!(err, ConfigError::SecretRefResolution { .. }),
        "期待するエラー種別ではない: {err:?}"
    );

    unsafe {
        std::env::remove_var("WNAV_PROFILE");
        std::env::remove_var("WNAV_CONFIG_DIR");
    }
}

#[test]
fn test_fail_fast_when_secret_ref_file_not_found() {
    // 存在しないファイルを参照する secret_ref は SecretRefResolution エラーになることを確認する
    let dir = tempfile::TempDir::new().expect("TempDir 作成に失敗");

    let base_yaml = r"
schema_version: 1
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
    password:
      secret_ref: 'file:/tmp/wnav_test_nonexistent_secret_fail_fast_xyz.txt'
  write:
    user: 'wnav_write'
    password: 'pass'
  read:
    user: 'wnav_read'
    password: 'pass'
cors:
  allow_origins:
    - 'http://localhost:5173'
  allow_credentials: true
  max_age_sec: 3600
jwt_public:
  algorithm: 'RS256'
  ttl_sec: 28800
  public_key: 'pub_key'
jwt_private:
  private_key: 'priv_key'
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
  hmac_key: 'hmac'
webhook_receiver:
  hmac_key: 'recv_hmac'
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

    write_config(&dir, "config.base.yml", base_yaml);
    write_config(&dir, "config.local.yml", "schema_version: 1\n");

    unsafe {
        std::env::set_var("WNAV_PROFILE", "local");
        std::env::set_var("WNAV_CONFIG_DIR", dir.path().to_str().unwrap());
    }

    let result = wnav_config::sources::build_figment();
    assert!(
        result.is_err(),
        "存在しないファイルを参照する secret_ref でも成功してしまった"
    );

    let err = result.unwrap_err();
    assert!(
        matches!(err, ConfigError::SecretRefResolution { .. }),
        "期待するエラー種別ではない: {err:?}"
    );

    unsafe {
        std::env::remove_var("WNAV_PROFILE");
        std::env::remove_var("WNAV_CONFIG_DIR");
    }
}
