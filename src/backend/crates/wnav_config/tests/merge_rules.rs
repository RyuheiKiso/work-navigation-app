// YAML マージ規則（map は深いマージ、配列は完全置換）が正しく適用されることを検証する
// figment の仕様通りに動くことを確認して設定上書きの挙動を保証する

// テストコードで環境変数操作のために unsafe を許容する（Edition 2024 の仕様）

use tempfile::TempDir;

/// テスト実行前に WNAV__ プレフィックスの環境変数をすべてクリアするヘルパー
/// 並行テスト実行時の環境変数競合を防ぐために呼び出す
fn clear_wnav_env_vars() {
    // テスト並行実行時に他のテストが設定した WNAV__ 環境変数が残らないよう事前にクリアする
    let keys_to_remove: Vec<String> = std::env::vars()
        .filter(|(k, _)| k.starts_with("WNAV__"))
        .map(|(k, _)| k)
        .collect();
    for key in keys_to_remove {
        unsafe {
            std::env::remove_var(&key);
        }
    }
}

/// テスト用の設定ディレクトリをセットアップして figment を構築するヘルパー
fn build_test_figment(profile: &str, base_extra: &str, profile_extra: &str) -> figment::Figment {
    // 他テストの残余環境変数をクリアしてから開始する
    clear_wnav_env_vars();

    let dir = TempDir::new().expect("TempDir 作成に失敗");
    let dir_path = dir.path().to_path_buf();

    let base_yaml = format!(
        r"
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
  host: 'base-host'
  port: 5432
  name: 'wnav_base'
  ssl_mode: 'prefer'
  max_connections: 20
  min_connections: 2
  acquire_timeout_sec: 10
  idle_timeout_sec: 600
  max_lifetime_sec: 3600
  event_insert:
    user: 'wnav_event_insert'
    password: 'base_event_insert_pass'
  write:
    user: 'wnav_write'
    password: 'base_write_pass'
  read:
    user: 'wnav_read'
    password: 'base_read_pass'
cors:
  allow_origins:
    - 'http://base-origin:5173'
  allow_credentials: true
  max_age_sec: 3600
jwt_public:
  algorithm: 'RS256'
  ttl_sec: 28800
  public_key: 'base_public_key'
jwt_private:
  private_key: 'base_private_key'
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
  hmac_key: 'base_hmac'
webhook_receiver:
  hmac_key: 'base_recv_hmac'
  hmac_timeout_ms: 5000
sse:
  keep_alive_sec: 25
  dispatch_retry_max: 5
integration:
  push_receive_enabled: false
external:
  backup_notification_url: 'http://base.example.com/notify'
frontend_master:
  api_base_url: 'http://base.example.com:8081'
  openapi_url: 'http://base.example.com:8081/api/openapi.json'
  session_timeout_min: 30
  polling_interval_ms: 30000
{base_extra}
"
    );

    let profile_yaml = format!(
        r"
schema_version: 1
{profile_extra}
"
    );

    std::fs::write(dir_path.join("config.base.yml"), &base_yaml)
        .expect("config.base.yml 書き込みに失敗");
    std::fs::write(dir_path.join(format!("config.{profile}.yml")), &profile_yaml)
        .expect("config.{profile}.yml 書き込みに失敗");

    // マージルール検証テストでは環境変数オーバーレイを使わずに figment を直接構築する
    // WNAV__* 環境変数が他のテストスレッドから残留している場合に影響を受けないようにする
    use figment::{providers::{Format, Yaml}, Figment};
    let figment = Figment::new()
        .merge(Yaml::file(dir_path.join("config.base.yml")))
        .merge(Yaml::file(dir_path.join(format!("config.{profile}.yml"))));

    // ファイルを削除しないよう dir を保持する
    std::mem::forget(dir);
    figment
}

#[test]
fn test_map_values_are_deep_merged() {
    // map の深いマージ: プロファイル側のキーが base の同名キーを上書きし他は引き継がれる
    let figment = build_test_figment(
        "local",
        "",
        r"
database:
  host: 'overridden-host'
  port: 5433
",
    );

    // プロファイル側で指定した値が上書きされていることを確認する
    let host: String = figment
        .find_value("database.host")
        .expect("database.host 取得失敗")
        .deserialize()
        .expect("deserialize 失敗");
    assert_eq!(host, "overridden-host", "database.host がプロファイルで上書きされていない");

    // 指定していないキーは base の値が引き継がれることを確認する
    let name: String = figment
        .find_value("database.name")
        .expect("database.name 取得失敗")
        .deserialize()
        .expect("deserialize 失敗");
    assert_eq!(name, "wnav_base", "database.name が base から引き継がれていない");
}

#[test]
fn test_array_values_are_completely_replaced() {
    // 配列の完全置換: プロファイル側の配列が base の配列を完全に置き換える
    let figment = build_test_figment(
        "local",
        "",
        r"
cors:
  allow_origins:
    - 'http://profile-origin:3000'
    - 'http://profile-origin2:3001'
",
    );

    // プロファイル側の配列だけが存在することを確認する（base のエントリは消える）
    let origins: Vec<String> = figment
        .find_value("cors.allow_origins")
        .expect("cors.allow_origins 取得失敗")
        .deserialize()
        .expect("deserialize 失敗");

    assert_eq!(origins.len(), 2, "配列要素数が期待値と異なる");
    assert!(origins.contains(&"http://profile-origin:3000".to_string()));
    assert!(
        !origins.contains(&"http://base-origin:5173".to_string()),
        "base の配列要素が残っている（完全置換されていない）"
    );
}

#[test]
fn test_env_var_overrides_yaml_value() {
    // WNAV__SECTION__KEY 形式の環境変数が YAML の値を上書きすることを確認する
    let dir = TempDir::new().expect("TempDir 作成に失敗");
    let dir_path = dir.path().to_path_buf();

    let base_yaml = r"
schema_version: 1
database:
  host: 'yaml-host'
  port: 5432
  name: 'wnav_base'
  ssl_mode: 'prefer'
  max_connections: 20
  min_connections: 2
  acquire_timeout_sec: 10
  idle_timeout_sec: 600
  max_lifetime_sec: 3600
  event_insert:
    user: 'wnav_event_insert'
    password: 'pass'
  write:
    user: 'wnav_write'
    password: 'pass'
  read:
    user: 'wnav_read'
    password: 'pass'
server:
  terminal_api:
    bind_addr: '0.0.0.0'
    port: 8080
    request_timeout_sec: 30
  master_api:
    bind_addr: '0.0.0.0'
    port: 8081
    request_timeout_sec: 30
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

    // 環境変数で database.host を上書きする
    unsafe {
        std::env::set_var("WNAV_PROFILE", "local");
        std::env::set_var("WNAV_CONFIG_DIR", dir_path.to_str().unwrap());
        std::env::set_var("WNAV__DATABASE__HOST", "env-overridden-host");
    }

    let figment = wnav_config::sources::build_figment().expect("build_figment に失敗");

    let host: String = figment
        .find_value("database.host")
        .expect("database.host 取得失敗")
        .deserialize()
        .expect("deserialize 失敗");
    assert_eq!(host, "env-overridden-host", "環境変数による上書きが反映されていない");

    unsafe {
        std::env::remove_var("WNAV_PROFILE");
        std::env::remove_var("WNAV_CONFIG_DIR");
        std::env::remove_var("WNAV__DATABASE__HOST");
    }
}
