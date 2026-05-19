// secret_ref の各スキーム（env / file / docker_secret）が正しく解決されることを検証する
// SecretResolver トレイトの実装を直接テストして期待した値が返ることを確認する

// テストコードで環境変数操作のために unsafe を許容する（Edition 2024 の仕様）

use wnav_config::secret_ref::{
    DockerSecretResolver, EnvSecretResolver, FileSecretResolver, SecretResolver,
};

// ────────────────────────────────────────────────────────────────
// env: スキームのテスト
// ────────────────────────────────────────────────────────────────

#[test]
fn test_env_resolver_returns_value_when_var_exists() {
    // 環境変数が設定されているとき対応する値が返ることを確認する
    unsafe {
        std::env::set_var("WNAV_TEST_SECRET_ENV", "test_secret_value_12345");
    }

    let resolver = EnvSecretResolver;
    let result = resolver.resolve("WNAV_TEST_SECRET_ENV");
    assert!(result.is_ok(), "env resolver が失敗: {:?}", result.err());
    assert_eq!(result.unwrap(), "test_secret_value_12345");

    unsafe {
        std::env::remove_var("WNAV_TEST_SECRET_ENV");
    }
}

#[test]
fn test_env_resolver_returns_error_when_var_not_found() {
    // 存在しない環境変数を指定したとき EnvVarNotFound エラーが返ることを確認する
    unsafe {
        std::env::remove_var("WNAV_TEST_SECRET_NOT_EXISTS");
    }

    let resolver = EnvSecretResolver;
    let result = resolver.resolve("WNAV_TEST_SECRET_NOT_EXISTS");
    assert!(result.is_err(), "存在しない環境変数でエラーが返らなかった");

    // エラー種別を確認する
    let err = result.unwrap_err();
    assert!(
        matches!(
            err,
            wnav_config::secret_ref::SecretResolveError::EnvVarNotFound(_)
        ),
        "期待するエラー種別ではない: {err:?}"
    );
}

#[test]
fn test_env_resolver_scheme_is_env() {
    // スキーム識別子が "env" であることを確認する
    let resolver = EnvSecretResolver;
    assert_eq!(resolver.scheme(), "env");
}

// ────────────────────────────────────────────────────────────────
// file: スキームのテスト
// ────────────────────────────────────────────────────────────────

#[test]
fn test_file_resolver_reads_file_content() {
    // ファイルが存在するとき内容（末尾トリム済み）が返ることを確認する
    let dir = tempfile::TempDir::new().expect("TempDir 作成に失敗");
    let file_path = dir.path().join("test_secret.txt");
    // 末尾改行を含む内容を書き込んでトリムが正しく動くことを確認する
    std::fs::write(&file_path, "file_secret_value\n").expect("ファイル書き込みに失敗");

    let resolver = FileSecretResolver;
    let result = resolver.resolve(file_path.to_str().unwrap());
    assert!(result.is_ok(), "file resolver が失敗: {:?}", result.err());
    // 末尾改行がトリムされていることを確認する
    assert_eq!(result.unwrap(), "file_secret_value");
}

#[test]
fn test_file_resolver_returns_error_when_file_not_found() {
    // 存在しないファイルを指定したとき FileReadFailed エラーが返ることを確認する
    let resolver = FileSecretResolver;
    let result = resolver.resolve("/tmp/wnav_test_nonexistent_secret_file_xyz.txt");
    assert!(result.is_err(), "存在しないファイルでエラーが返らなかった");

    let err = result.unwrap_err();
    assert!(
        matches!(
            err,
            wnav_config::secret_ref::SecretResolveError::FileReadFailed { .. }
        ),
        "期待するエラー種別ではない: {err:?}"
    );
}

#[test]
fn test_file_resolver_scheme_is_file() {
    // スキーム識別子が "file" であることを確認する
    let resolver = FileSecretResolver;
    assert_eq!(resolver.scheme(), "file");
}

// ────────────────────────────────────────────────────────────────
// docker_secret: スキームのテスト
// ────────────────────────────────────────────────────────────────

#[test]
fn test_docker_secret_resolver_returns_error_when_not_available() {
    // /run/secrets/ が存在しない環境（開発 PC）ではエラーが返ることを確認する
    let resolver = DockerSecretResolver;
    let result = resolver.resolve("wnav_test_nonexistent_secret_xyz");
    // CI・開発環境では /run/secrets/ が存在しないためエラーが期待値
    assert!(
        result.is_err(),
        "Docker secret が存在しない環境でエラーが返らなかった"
    );

    let err = result.unwrap_err();
    assert!(
        matches!(
            err,
            wnav_config::secret_ref::SecretResolveError::DockerSecretNotFound(_)
        ),
        "期待するエラー種別ではない: {err:?}"
    );
}

#[test]
fn test_docker_secret_resolver_scheme_is_docker_secret() {
    // スキーム識別子が "docker_secret" であることを確認する
    let resolver = DockerSecretResolver;
    assert_eq!(resolver.scheme(), "docker_secret");
}

// ────────────────────────────────────────────────────────────────
// SecretRefProvider の統合テスト
// ────────────────────────────────────────────────────────────────

#[test]
fn test_secret_ref_env_is_resolved_in_figment() {
    // YAML 内の secret_ref: "env:VAR" が figment 経由で解決されることを確認する
    let dir = tempfile::TempDir::new().expect("TempDir 作成に失敗");
    let dir_path = dir.path().to_path_buf();

    // テスト用の環境変数を設定する
    unsafe {
        std::env::set_var("WNAV_TEST_DB_PASSWORD", "resolved_db_password_xyz");
    }

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
      secret_ref: 'env:WNAV_TEST_DB_PASSWORD'
  write:
    user: 'wnav_write'
    password: 'plain_write_password'
  read:
    user: 'wnav_read'
    password: 'plain_read_password'
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

    std::fs::write(dir_path.join("config.base.yml"), base_yaml)
        .expect("config.base.yml 書き込みに失敗");
    std::fs::write(dir_path.join("config.local.yml"), "schema_version: 1\n")
        .expect("config.local.yml 書き込みに失敗");

    unsafe {
        std::env::set_var("WNAV_PROFILE", "local");
        std::env::set_var("WNAV_CONFIG_DIR", dir_path.to_str().unwrap());
    }

    let figment = wnav_config::sources::build_figment().expect("build_figment に失敗");

    // secret_ref が解決済みの値でデシリアライズできることを確認する
    let config: wnav_config::TerminalApiConfig = figment
        .extract()
        .expect("TerminalApiConfig のデシリアライズに失敗");

    // password.expose() が解決済みの値を返すことを確認する
    assert_eq!(
        config.database.event_insert.password.expose(),
        "resolved_db_password_xyz",
        "secret_ref が環境変数の値に解決されていない"
    );

    unsafe {
        std::env::remove_var("WNAV_PROFILE");
        std::env::remove_var("WNAV_CONFIG_DIR");
        std::env::remove_var("WNAV_TEST_DB_PASSWORD");
    }
}

#[test]
fn test_secret_ref_file_is_resolved_in_figment() {
    // YAML 内の secret_ref: "file:/path" が figment 経由で解決されることを確認する
    let dir = tempfile::TempDir::new().expect("TempDir 作成に失敗");
    let dir_path = dir.path().to_path_buf();

    // シークレットファイルを作成する
    let secret_file = dir_path.join("test_db_secret.txt");
    std::fs::write(&secret_file, "file_resolved_password\n")
        .expect("シークレットファイル書き込みに失敗");

    let base_yaml = format!(
        r"
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
      secret_ref: 'file:{}'
  write:
    user: 'wnav_write'
    password: 'plain_write_password'
  read:
    user: 'wnav_read'
    password: 'plain_read_password'
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
",
        secret_file.to_str().unwrap()
    );

    std::fs::write(dir_path.join("config.base.yml"), &base_yaml)
        .expect("config.base.yml 書き込みに失敗");
    std::fs::write(dir_path.join("config.local.yml"), "schema_version: 1\n")
        .expect("config.local.yml 書き込みに失敗");

    unsafe {
        std::env::set_var("WNAV_PROFILE", "local");
        std::env::set_var("WNAV_CONFIG_DIR", dir_path.to_str().unwrap());
    }

    let figment = wnav_config::sources::build_figment().expect("build_figment に失敗");
    let config: wnav_config::TerminalApiConfig = figment
        .extract()
        .expect("TerminalApiConfig のデシリアライズに失敗");

    assert_eq!(
        config.database.event_insert.password.expose(),
        "file_resolved_password",
        "secret_ref がファイルの値に解決されていない"
    );

    unsafe {
        std::env::remove_var("WNAV_PROFILE");
        std::env::remove_var("WNAV_CONFIG_DIR");
    }
}
