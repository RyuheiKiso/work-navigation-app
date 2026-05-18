# 10 wnav_config 詳細設計（MOD-BE-CONFIG）

本章は `crates/wnav_config/` の設定読込クレートの詳細設計を確定する。  
ADR-IMPL-001（設定管理を YAML + figment + secret_ref で一元化）に基づき、  
すべてのバイナリが経由して設定を取得する専用クレートを定義する。

---

## 1. 概要

`wnav_config` クレートは以下の責務を持つ:

1. `src/infra/config/config.base.yml` + `config.{WNAV_PROFILE}.yml` の読込とマージ
2. 環境変数オーバーレイ（`WNAV__SECTION__KEY` 形式）の適用
3. `secret_ref:` の実体解決（env / dpapi / docker_secret / file スキーム）
4. バイナリ別の型安全なデシリアライズ（`TerminalApiConfig` / `MasterApiConfig`）
5. 起動時 fail-fast バリデーション（スキーマ整合性・必須項目・機密解決）
6. 機密フィールドのデバッグダンプマスキング

---

## 2. クレート構成

```
src/backend/crates/wnav_config/
  Cargo.toml
  src/
    lib.rs              # 公開 API: load_terminal_api() / load_master_api()
    profile.rs          # WNAV_PROFILE 解析・プロファイル型
    sources.rs          # figment Provider 合成ロジック
    secret_ref.rs       # SecretRefProvider と SecretResolver trait
    schema.rs           # SharedConfig / TerminalApiConfig / MasterApiConfig 構造体
    validation.rs       # 構造体整合性チェック（validate()）
    redact.rs           # Secret<T> 型と dump_config()
    error.rs            # ConfigError（thiserror）
  tests/
    profile_selection.rs        # プロファイル選択テスト
    merge_rules.rs              # YAML マージ規則テスト
    secret_ref_resolution.rs    # secret_ref 解決テスト
    fail_fast.rs                # fail-fast テスト
    redact.rs                   # Debug マスキングテスト
    compile_fail/
      terminal_no_write.rs      # trybuild コンパイルエラーテスト
```

---

## 3. 依存クレート

```toml
# crates/wnav_config/Cargo.toml
[dependencies]
figment = { version = "0.10", features = ["yaml", "env"] }
serde = { version = "1", features = ["derive"] }
thiserror = "2"
tracing = "0.1"

[dev-dependencies]
tempfile = "3"
trybuild = "1"
```

---

## 4. 公開 API（lib.rs）

```rust
// crates/wnav_config/src/lib.rs

pub use schema::{MasterApiConfig, TerminalApiConfig};
pub use error::ConfigError;

/// wnav_terminal_api 起動時に呼ぶ。WNAV_PROFILE 必須。
/// YAML ファイル欠損・schema_version 不一致・secret_ref 解決失敗時は ConfigError を返す。
pub fn load_terminal_api() -> Result<TerminalApiConfig, ConfigError> {
    sources::build_figment()?.extract().map_err(ConfigError::Extract)
        .and_then(validation::validate_terminal)
}

/// wnav_master_api 起動時に呼ぶ。
pub fn load_master_api() -> Result<MasterApiConfig, ConfigError> {
    sources::build_figment()?.extract().map_err(ConfigError::Extract)
        .and_then(validation::validate_master)
}
```

---

## 5. Provider 合成（sources.rs）

```rust
// crates/wnav_config/src/sources.rs

use figment::{Figment, providers::{Env, Yaml}};
use crate::{profile::Profile, secret_ref::SecretRefProvider, error::ConfigError};

pub fn build_figment() -> Result<Figment, ConfigError> {
    let profile = Profile::from_env()?;                   // WNAV_PROFILE 必須
    let dir = config_dir();                               // WNAV_CONFIG_DIR or 既定パス

    let figment = Figment::new()
        .merge(Yaml::file(dir.join("config.base.yml")))
        .merge(Yaml::file(dir.join(format!("config.{}.yml", profile))))
        .merge(Env::prefixed("WNAV__").split("__"))       // 動的上書き
        .merge(SecretRefProvider::new(&profile)?);        // secret_ref 解決（最終段）

    // schema_version == 1 を起動時に強制する
    let version: u32 = figment.find_value("schema_version")
        .ok()
        .and_then(|v| v.to_i128().map(|i| i as u32))
        .unwrap_or(0);
    if version != 1 {
        return Err(ConfigError::SchemaVersionMismatch { expected: 1, got: version });
    }

    Ok(figment)
}

fn config_dir() -> std::path::PathBuf {
    if let Ok(dir) = std::env::var("WNAV_CONFIG_DIR") {
        return dir.into();
    }
    // 開発環境: リポジトリルートからの相対パス
    if let Ok(dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let candidate = std::path::PathBuf::from(dir)
            .parent().unwrap().parent().unwrap()
            .join("infra/config");
        if candidate.exists() { return candidate; }
    }
    // 本番環境: 標準配置パス
    "/etc/wnav/config".into()
}
```

---

## 6. 構造体設計（schema.rs）

### 6.1 バイナリ別型分離

```rust
// 共有設定（両バイナリ共通）
#[derive(Debug, serde::Deserialize, Clone)]
pub struct SharedConfig {
    pub schema_version: u32,
    pub observability: ObservabilityConfig,
    pub cors: CorsConfig,
    pub jwt_public: JwtPublicConfig,          // 公開鍵のみ
}

/// wnav_terminal_api が使う設定型
/// database.write / jwt.private_key を型として持たない（コンパイル時強制）
#[derive(Debug, serde::Deserialize, Clone)]
pub struct TerminalApiConfig {
    #[serde(flatten)]
    pub shared: SharedConfig,
    pub server: TerminalServerConfig,
    pub database: TerminalDatabaseConfig,     // event_insert + read のみ
    pub idempotency: IdempotencyConfig,
    pub outbox: OutboxConfig,
    pub rate_limit: RateLimitConfig,
    pub webhook: WebhookConfig,
}

/// wnav_master_api が使う設定型
/// database.event_insert / idempotency / outbox を型として持たない
#[derive(Debug, serde::Deserialize, Clone)]
pub struct MasterApiConfig {
    #[serde(flatten)]
    pub shared: SharedConfig,
    pub server: MasterServerConfig,
    pub database: MasterDatabaseConfig,       // write + read のみ
    pub jwt_private: JwtPrivateConfig,        // 秘密鍵（master のみ）
    pub hash_chain_verify: HashChainVerifyConfig,
    pub frontend_master: FrontendMasterConfig, // /public/config 用
}
```

### 6.2 DB ロール別設定型

```rust
/// TerminalDatabaseConfig には write フィールドが存在しない
#[derive(Debug, serde::Deserialize, Clone)]
pub struct TerminalDatabaseConfig {
    pub host: String,
    pub port: u16,
    pub name: String,
    pub ssl_mode: SslMode,
    pub max_connections: u32,
    pub min_connections: u32,
    pub acquire_timeout_sec: u64,
    pub event_insert: DbRoleConfig,
    pub read: DbRoleConfig,
    // write フィールドは存在しない → コンパイル時に初期化不可
}

/// MasterDatabaseConfig には event_insert フィールドが存在しない
#[derive(Debug, serde::Deserialize, Clone)]
pub struct MasterDatabaseConfig {
    pub host: String,
    pub port: u16,
    pub name: String,
    pub ssl_mode: SslMode,
    pub max_connections: u32,
    pub min_connections: u32,
    pub acquire_timeout_sec: u64,
    pub write: DbRoleConfig,
    pub read: DbRoleConfig,
    // event_insert フィールドは存在しない
}

#[derive(Debug, serde::Deserialize, Clone)]
pub struct DbRoleConfig {
    pub user: String,
    pub password: Secret,     // secret_ref が解決済みの平文値
}
```

---

## 7. Secret 型と機密マスキング（redact.rs）

```rust
// crates/wnav_config/src/redact.rs

/// 機密値を保持する型。Debug 出力で常にマスキングされる。
#[derive(Clone, serde::Deserialize)]
pub struct Secret(String);

impl std::fmt::Debug for Secret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Secret(***REDACTED*** {} bytes)", self.0.len())
    }
}

impl Secret {
    /// 実値を取得する（ログに出力しないこと）
    pub fn expose(&self) -> &str {
        &self.0
    }
}

/// 起動時に設定サマリを1行出力する。機密フィールドは含まない。
pub fn dump_config_summary(cfg: &TerminalApiConfig) {
    tracing::info!(
        profile = %cfg.shared.schema_version,
        port = cfg.server.terminal_api.port,
        db_host = %cfg.database.host,
        db_max_conn = cfg.database.max_connections,
        log_level = %cfg.shared.observability.log_level,
        "configuration loaded"
    );
}
```

---

## 8. secret_ref 解決器（secret_ref.rs）

```rust
// crates/wnav_config/src/secret_ref.rs

use figment::{Figment, Profile, Provider, Metadata, Map, Dict};

/// secret_ref: "<scheme>:<id>" を平文値に解決する figment Provider
pub struct SecretRefProvider {
    resolvers: Vec<Box<dyn SecretResolver>>,
}

impl SecretRefProvider {
    pub fn new(profile: &Profile) -> Result<Self, crate::error::ConfigError> {
        let mut resolvers: Vec<Box<dyn SecretResolver>> = vec![
            Box::new(EnvSecretResolver),
            Box::new(FileSecretResolver),
            Box::new(DockerSecretResolver),
        ];
        #[cfg(target_os = "windows")]
        resolvers.push(Box::new(DpapiSecretResolver));
        Ok(Self { resolvers })
    }
}

impl Provider for SecretRefProvider {
    fn metadata(&self) -> Metadata { Metadata::named("secret_ref resolver") }
    fn data(&self) -> Result<Map<Profile, Dict>, figment::Error> {
        // 既マージ済みの Figment ツリーを走査し、
        // { secret_ref: "scheme:id" } 形式のノードを平文値に置換した Dict を返す
        todo!("実装フェーズで完成させる")
    }
}

/// secret_ref スキームを解決するトレイト
pub trait SecretResolver: Send + Sync {
    fn scheme(&self) -> &'static str;
    fn resolve(&self, id: &str) -> Result<String, SecretResolveError>;
}

pub struct EnvSecretResolver;
impl SecretResolver for EnvSecretResolver {
    fn scheme(&self) -> &'static str { "env" }
    fn resolve(&self, id: &str) -> Result<String, SecretResolveError> {
        std::env::var(id).map_err(|_| SecretResolveError::EnvVarNotFound(id.to_string()))
    }
}

pub struct FileSecretResolver;
impl SecretResolver for FileSecretResolver {
    fn scheme(&self) -> &'static str { "file" }
    fn resolve(&self, path: &str) -> Result<String, SecretResolveError> {
        std::fs::read_to_string(path)
            .map(|s| s.trim().to_string())
            .map_err(|e| SecretResolveError::FileReadFailed { path: path.to_string(), source: e })
    }
}

pub struct DockerSecretResolver;
impl SecretResolver for DockerSecretResolver {
    fn scheme(&self) -> &'static str { "docker_secret" }
    fn resolve(&self, name: &str) -> Result<String, SecretResolveError> {
        let path = format!("/run/secrets/{}", name);
        std::fs::read_to_string(&path)
            .map(|s| s.trim().to_string())
            .map_err(|e| SecretResolveError::FileReadFailed { path, source: e })
    }
}
```

---

## 9. 起動時 fail-fast 設計（main.rs での呼び出しパターン）

```rust
// crates/wnav_terminal_api/src/main.rs

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. 設定ロード（fail-fast）
    let config = wnav_config::load_terminal_api().unwrap_or_else(|e| {
        // tracing 初期化前なので eprintln を使う
        eprintln!("FATAL: configuration load failed: {e}");
        std::process::exit(78);  // EX_CONFIG
    });

    // 2. tracing 初期化（設定ロード後）
    init_tracing(config.shared.observability.log_level.as_str());

    // 3. 起動サマリログ（機密マスキング済み）
    wnav_config::redact::dump_config_summary(&config);

    // 4. DB 接続（接続先は config から取得）
    let event_pool = wnav_db::connect_event_insert(
        &config.database.host,
        config.database.port,
        &config.database.name,
        &config.database.event_insert.user,
        config.database.event_insert.password.expose(),
        &config.database,
    ).await?;

    // ...（以降の初期化）
}
```

---

## 10. バリデーション（validation.rs）

```rust
// crates/wnav_config/src/validation.rs

pub fn validate_terminal(cfg: TerminalApiConfig) -> Result<TerminalApiConfig, ConfigError> {
    // ポート範囲
    validate_port(cfg.server.terminal_api.port)?;
    // 接続プール整合性
    if cfg.database.min_connections > cfg.database.max_connections {
        return Err(ConfigError::InvalidValue {
            field: "database.min_connections".to_string(),
            reason: "min_connections > max_connections".to_string(),
        });
    }
    // JWT TTL 最小値（60 秒以上）
    if cfg.shared.jwt_public.ttl_sec < 60 {
        return Err(ConfigError::InvalidValue {
            field: "jwt.ttl_sec".to_string(),
            reason: "must be >= 60 seconds".to_string(),
        });
    }
    Ok(cfg)
}
```

---

## 11. テスト方針

| テスト種別 | テストファイル | 確認内容 |
|---|---|---|
| プロファイル選択 | `tests/profile_selection.rs` | `WNAV_PROFILE=dev` で `config.dev.yml` がマージされること |
| マージ規則 | `tests/merge_rules.rs` | 配列が完全置換・map が深いマージされること |
| secret_ref (env:) | `tests/secret_ref_resolution.rs` | 環境変数から値が解決されること |
| secret_ref (file:) | `tests/secret_ref_resolution.rs` | ファイルから値が解決されること |
| secret_ref (docker_secret:) | `tests/secret_ref_resolution.rs` | `/run/secrets/` から値が解決されること |
| secret_ref 欠損 | `tests/fail_fast.rs` | 解決失敗時に `ConfigError` が返ること |
| `WNAV_PROFILE` 未設定 | `tests/fail_fast.rs` | `ConfigError::ProfileNotSet` が返ること |
| schema_version 不一致 | `tests/fail_fast.rs` | `ConfigError::SchemaVersionMismatch` が返ること |
| Debug マスキング | `tests/redact.rs` | `format!("{:?}", config)` に平文パスワードが含まれないこと |
| 型分離（コンパイル失敗） | `tests/compile_fail/terminal_no_write.rs` | `TerminalApiConfig` から `database.write` にアクセスするとコンパイルエラーになること（trybuild） |

---

## 12. エラー型（error.rs）

```rust
// crates/wnav_config/src/error.rs

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("WNAV_PROFILE is not set. Must be one of: local, dev, staging, prod")]
    ProfileNotSet,

    #[error("Config file not found: {path}")]
    FileNotFound { path: String },

    #[error("Config schema version mismatch: expected {expected}, got {got}")]
    SchemaVersionMismatch { expected: u32, got: u32 },

    #[error("Failed to extract config: {0}")]
    Extract(#[from] figment::Error),

    #[error("Failed to resolve secret_ref '{secret_ref}': {reason}")]
    SecretRefResolution { secret_ref: String, reason: String },

    #[error("Invalid config value for '{field}': {reason}")]
    InvalidValue { field: String, reason: String },
}
```

---

## 13. 整合性（既存クレートへの影響）

| 影響クレート | 変更内容 |
|---|---|
| `wnav_terminal_api` | `envy::from_env()` を `wnav_config::load_terminal_api()` に差し替え。`AppConfig` を `TerminalApiConfig` に型変更 |
| `wnav_master_api` | `envy::from_env()` を `wnav_config::load_master_api()` に差し替え。`AppConfig` を `MasterApiConfig` に型変更 |
| `wnav_db` | `DbConfig` の出処を「envy で読む」→「`wnav_config` から渡される」に変更。クレートが `envy` に依存するのをやめる |
| その他クレート | 変更なし |

---

## 参照

- ADR: `docs/06_実装/ADR/ADR-IMPL-001_設定管理を_YAML_figment_secret_refで一元化.md`
- YAML スキーマ: `src/infra/config/schema/config.schema.json`
- YAML 配置・運用: `src/infra/config/README.md`
- 上位設計: `docs/04_概要設計/02_ソフトウェア方式設計/06_共通基盤コンポーネント設計.md` §6
- 関連詳細設計（terminal-api）: `docs/05_詳細設計/02_バックエンド詳細設計/01_wnav_terminal_api詳細設計.md`
- 関連詳細設計（master-api）: `docs/05_詳細設計/02_バックエンド詳細設計/01a_wnav_master_api詳細設計.md`
- 関連詳細設計: `docs/05_詳細設計/02_バックエンド詳細設計/03_wnav_db詳細設計.md`
