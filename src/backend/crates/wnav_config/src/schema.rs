// config.base.yml の全セクションを Rust 型に写像する
// TerminalApiConfig と MasterApiConfig を型分離して存在しないフィールドへのアクセスをコンパイルエラーにする

use crate::redact::Secret;
use serde::Deserialize;

// ────────────────────────────────────────────────────────────────
// SSL モード
// ────────────────────────────────────────────────────────────────

/// PostgreSQL の SSL 接続モード
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SslMode {
    // 接続に TLS を必須とする（本番環境推奨）
    Require,
    // TLS を試みるが失敗しても平文で接続する（開発環境向け）
    Prefer,
    // TLS を使用しない（開発・テスト専用）
    Disable,
}

// ────────────────────────────────────────────────────────────────
// DB ロール設定
// ────────────────────────────────────────────────────────────────

/// DB ロール（write / event_insert / read）の接続情報
/// password は Secret 型でラップして Debug 出力でマスキングされる
#[derive(Debug, Clone, Deserialize)]
pub struct DbRoleConfig {
    // PostgreSQL の接続ユーザー名
    pub user: String,
    // secret_ref が解決済みの平文パスワード（Debug 出力ではマスキングされる）
    pub password: Secret,
}

// ────────────────────────────────────────────────────────────────
// サーバー設定
// ────────────────────────────────────────────────────────────────

/// wnav_terminal_api のリスンアドレス・ポート設定
#[derive(Debug, Clone, Deserialize)]
pub struct TerminalServerConfig {
    // wnav_terminal_api の listen 設定（ハンディ端末向け）
    pub terminal_api: ServerConfig,
}

/// wnav_master_api のリスンアドレス・ポート設定
#[derive(Debug, Clone, Deserialize)]
pub struct MasterServerConfig {
    // wnav_master_api の listen 設定（管理 PC 向け）
    pub master_api: ServerConfig,
}

/// 各 API サーバー共通のリスンパラメータ
#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    // バインドアドレス（例: "0.0.0.0" / "127.0.0.1"）
    pub bind_addr: String,
    // リスンポート番号
    pub port: u16,
    // リクエストタイムアウト（秒）
    pub request_timeout_sec: u64,
}

// ────────────────────────────────────────────────────────────────
// データベース設定（バイナリ別型分離）
// ────────────────────────────────────────────────────────────────

/// TerminalApiConfig 専用のデータベース設定
/// event_insert + read ロールのみを持つ。write ロールは型として存在しない
#[derive(Debug, Clone, Deserialize)]
pub struct TerminalDatabaseConfig {
    // DB ホスト名
    pub host: String,
    // DB ポート番号
    pub port: u16,
    // DB 名
    pub name: String,
    // SSL 接続モード
    pub ssl_mode: SslMode,
    // 最大接続プールサイズ
    pub max_connections: u32,
    // 最小常駐接続数
    pub min_connections: u32,
    // 接続取得タイムアウト（秒）
    pub acquire_timeout_sec: u64,
    // アイドル接続タイムアウト（秒）
    pub idle_timeout_sec: u64,
    // 接続の最大生存期間（秒）
    pub max_lifetime_sec: u64,
    // 作業ログ INSERT 専用ロール。SELECT 不可なので terminal_api が排他的に保有する
    pub event_insert: DbRoleConfig,
    // 読み取り専用ロール（両バイナリで使用）
    pub read: DbRoleConfig,
    // write フィールドは存在しない → TerminalApiConfig から database.write にアクセスするとコンパイルエラーになる
}

/// MasterApiConfig 専用のデータベース設定
/// write + read ロールのみを持つ。event_insert ロールは型として存在しない
#[derive(Debug, Clone, Deserialize)]
pub struct MasterDatabaseConfig {
    // DB ホスト名
    pub host: String,
    // DB ポート番号
    pub port: u16,
    // DB 名
    pub name: String,
    // SSL 接続モード
    pub ssl_mode: SslMode,
    // 最大接続プールサイズ
    pub max_connections: u32,
    // 最小常駐接続数
    pub min_connections: u32,
    // 接続取得タイムアウト（秒）
    pub acquire_timeout_sec: u64,
    // アイドル接続タイムアウト（秒）
    pub idle_timeout_sec: u64,
    // 接続の最大生存期間（秒）
    pub max_lifetime_sec: u64,
    // マスタデータ書き込み専用ロール（master_api 排他）
    pub write: DbRoleConfig,
    // 読み取り専用ロール（両バイナリで使用）
    pub read: DbRoleConfig,
    // event_insert フィールドは存在しない → MasterApiConfig から database.event_insert にアクセスするとコンパイルエラーになる
}

// ────────────────────────────────────────────────────────────────
// 可観測性・CORS・JWT 設定
// ────────────────────────────────────────────────────────────────

/// 構造化ログ・メトリクスの可観測性設定
#[derive(Debug, Clone, Deserialize)]
pub struct ObservabilityConfig {
    // ログレベル（debug / info / warn / error）
    pub log_level: String,
    // ログフォーマット（json / text）
    pub log_format: String,
    // リクエスト ID ヘッダ名
    pub request_id_header: String,
    // Prometheus メトリクス収集の有効フラグ
    pub metrics_enabled: bool,
    // メトリクスエンドポイントのパス
    pub metrics_path: String,
}

/// CORS ポリシー設定
#[derive(Debug, Clone, Deserialize)]
pub struct CorsConfig {
    // 許可するオリジンの一覧（配列は override で完全置換される）
    pub allow_origins: Vec<String>,
    // 認証情報（Cookie・Authorization ヘッダ）の送信を許可するか
    pub allow_credentials: bool,
    // プリフライトキャッシュの有効時間（秒）
    pub max_age_sec: u64,
}

/// JWT 公開鍵設定（両バイナリで使用）
#[derive(Debug, Clone, Deserialize)]
pub struct JwtPublicConfig {
    // JWT 署名アルゴリズム（RS256 固定）
    pub algorithm: String,
    // トークンの有効時間（秒）。60 秒以上必須
    pub ttl_sec: u64,
    // RS256 公開鍵（PEM 形式）。検証のみに使用する
    pub public_key: Secret,
}

/// JWT 秘密鍵設定（master_api 専用）
/// terminal_api は署名を行わないため TerminalApiConfig には含まれない
#[derive(Debug, Clone, Deserialize)]
pub struct JwtPrivateConfig {
    // RS256 秘密鍵（PEM 形式）。master_api のみが保有する
    pub private_key: Secret,
}

// ────────────────────────────────────────────────────────────────
// terminal_api 専用設定
// ────────────────────────────────────────────────────────────────

/// Idempotency-Key キャッシュ設定（terminal_api 専用）
#[derive(Debug, Clone, Deserialize)]
pub struct IdempotencyConfig {
    // キャッシュの有効期間（秒）。デフォルト 86400 = 24h
    pub ttl_sec: u64,
}

/// Outbox コンシューマ設定（terminal_api 専用）
#[derive(Debug, Clone, Deserialize)]
pub struct OutboxConfig {
    // ポーリング間隔（ミリ秒）
    pub interval_ms: u64,
    // 最大リトライ回数
    pub retry_max: u32,
    // リトライ指数バックオフの上限（秒）
    pub backoff_max_sec: u64,
}

/// レートリミット設定（terminal_api 専用）
#[derive(Debug, Clone, Deserialize)]
pub struct RateLimitConfig {
    // 毎秒のリクエスト上限
    pub rps: u32,
    // バースト許容数（瞬間的なスパイクを吸収する）
    pub burst: u32,
}

/// Webhook 配信設定（terminal_api 専用）
#[derive(Debug, Clone, Deserialize)]
pub struct WebhookConfig {
    // 署名ヘッダ名（例: X-Signature-256）
    pub signature_header: String,
    // 最大リトライ回数
    pub retry_max: u32,
    // リトライバックオフの上限（秒）
    pub retry_backoff_max_sec: u64,
    // HMAC-SHA256 署名用秘密鍵
    pub hmac_key: Secret,
}

/// Webhook 受信設定（外部システムからの Push 受信）
#[derive(Debug, Clone, Deserialize)]
pub struct WebhookReceiverConfig {
    // 受信 HMAC 検証用秘密鍵
    pub hmac_key: Secret,
    // HMAC タイムスタンプ許容誤差（ミリ秒）
    pub hmac_timeout_ms: u64,
}

/// SSE（Server-Sent Events）配信設定
#[derive(Debug, Clone, Deserialize)]
pub struct SseConfig {
    // Keep-Alive 送信間隔（秒）
    pub keep_alive_sec: u64,
    // ディスパッチ最大リトライ回数
    pub dispatch_retry_max: u32,
}

/// 統合機能フラグ設定
#[derive(Debug, Clone, Deserialize)]
pub struct IntegrationConfig {
    // 外部 Push 受信の有効フラグ
    pub push_receive_enabled: bool,
}

/// 外部通知設定
#[derive(Debug, Clone, Deserialize)]
pub struct ExternalConfig {
    // バックアップ通知 URL（secret_ref で解決済み）
    pub backup_notification_url: Secret,
}

// ────────────────────────────────────────────────────────────────
// master_api 専用設定
// ────────────────────────────────────────────────────────────────

/// ハッシュチェーン定期検証ジョブ設定（master_api 専用）
#[derive(Debug, Clone, Deserialize)]
pub struct HashChainVerifyConfig {
    // 実行スケジュール（cron 形式）
    pub cron: String,
}

/// フロントエンド（マスタ SPA）への公開設定
/// GET /api/v1/public/config が返す内容として使用する（非機密のみ）
#[derive(Debug, Clone, Deserialize)]
pub struct FrontendMasterConfig {
    // マスタ SPA が接続する API のベース URL
    pub api_base_url: String,
    // OpenAPI スキーマ配信 URL
    pub openapi_url: String,
    // セッションタイムアウト（分）
    pub session_timeout_min: u64,
    // マスタ SPA のポーリング間隔（ミリ秒）
    pub polling_interval_ms: u64,
}

// ────────────────────────────────────────────────────────────────
// 共有設定（両バイナリ共通）
// ────────────────────────────────────────────────────────────────

/// 両バイナリで共有するトップレベル設定
/// jwt_public は両バイナリが検証に使用するため SharedConfig に含める
#[derive(Debug, Clone, Deserialize)]
pub struct SharedConfig {
    // 設定スキーマバージョン（現在は 1 固定）
    pub schema_version: u32,
    // 構造化ログ・メトリクス設定
    pub observability: ObservabilityConfig,
    // CORS ポリシー設定
    pub cors: CorsConfig,
    // JWT 検証用公開鍵（両バイナリが保有）
    pub jwt_public: JwtPublicConfig,
}

// ────────────────────────────────────────────────────────────────
// バイナリ別トップレベル設定型
// ────────────────────────────────────────────────────────────────

/// wnav_terminal_api の設定型
/// database.write および jwt_private_key を型として持たない
/// 誤ったフィールドへのアクセスはコンパイルエラーになる
#[derive(Debug, Clone, Deserialize)]
pub struct TerminalApiConfig {
    // 両バイナリ共通の設定（flatten によりトップレベルキーとして展開される）
    #[serde(flatten)]
    pub shared: SharedConfig,
    // terminal_api の listen 設定（port 8080）
    pub server: TerminalServerConfig,
    // event_insert + read ロールのみ（write ロールは存在しない）
    pub database: TerminalDatabaseConfig,
    // Idempotency-Key キャッシュ（terminal_api 専用）
    pub idempotency: IdempotencyConfig,
    // Outbox コンシューマ（terminal_api 専用）
    pub outbox: OutboxConfig,
    // レートリミット（terminal_api 専用）
    pub rate_limit: RateLimitConfig,
    // Webhook 配信（terminal_api 専用）
    pub webhook: WebhookConfig,
    // Webhook 受信（terminal_api 専用）
    pub webhook_receiver: WebhookReceiverConfig,
    // SSE 配信（terminal_api 専用）
    pub sse: SseConfig,
    // 統合機能フラグ
    pub integration: IntegrationConfig,
    // 外部通知設定
    pub external: ExternalConfig,
}

/// wnav_master_api の設定型
/// database.event_insert および idempotency / outbox を型として持たない
/// 誤ったフィールドへのアクセスはコンパイルエラーになる
#[derive(Debug, Clone, Deserialize)]
pub struct MasterApiConfig {
    // 両バイナリ共通の設定（flatten によりトップレベルキーとして展開される）
    #[serde(flatten)]
    pub shared: SharedConfig,
    // master_api の listen 設定（port 8081）
    pub server: MasterServerConfig,
    // write + read ロールのみ（event_insert ロールは存在しない）
    pub database: MasterDatabaseConfig,
    // RS256 秘密鍵（master_api が JWT を発行するために必要）
    pub jwt_private: JwtPrivateConfig,
    // ハッシュチェーン定期検証ジョブ設定
    pub hash_chain_verify: HashChainVerifyConfig,
    // フロントエンド公開設定
    pub frontend_master: FrontendMasterConfig,
}
