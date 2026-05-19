// PgAuthLogRepository — TBL-032 auth_logs の sqlx 実装（Append-only）
// ログイン・ログアウト・認証失敗イベントを不変ログとして記録する。
// INSERT のみを提供し、UPDATE・DELETE は提供しない（Append-only 原則）。
// SQLX_PREPARE_REQUIRED: cargo sqlx prepare を実行してからビルド可能

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

/// 認証ログのイベント種別。
#[derive(Debug, Clone)]
pub enum AuthLogEventType {
    /// ログイン成功
    LoginSuccess,
    /// ログアウト
    Logout,
    /// ログイン失敗（認証エラー）
    LoginFailure,
}

impl AuthLogEventType {
    /// DB 格納文字列に変換する。
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::LoginSuccess => "LOGIN_SUCCESS",
            Self::Logout => "LOGOUT",
            Self::LoginFailure => "LOGIN_FAILURE",
        }
    }
}

/// 認証ログ挿入コマンド。
#[derive(Debug)]
pub struct InsertAuthLogCmd {
    /// ログ ID（UUID v7）
    pub log_id: Uuid,
    /// ユーザー ID（認証失敗の場合は None）
    pub user_id: Option<Uuid>,
    /// ログイン試行 ID（認証失敗時は入力値）
    pub login_id: Option<String>,
    /// イベント種別
    pub event_type: AuthLogEventType,
    /// クライアント IP アドレス（IPv4/IPv6 文字列）
    pub ip_address: Option<String>,
    /// User-Agent ヘッダ値
    pub user_agent: Option<String>,
    /// 記録日時（サーバー側で付与する権威タイムスタンプ）
    pub created_at: DateTime<Utc>,
}

/// TBL-032 auth_logs の Append-only リポジトリ実装。
/// INSERT のみを提供し、UPDATE・DELETE は提供しない（Append-only 原則）。
pub struct PgAuthLogRepository {
    pool: PgPool,
}

impl PgAuthLogRepository {
    /// コネクションプールを受け取ってリポジトリを構築する。
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Append-only: 認証ログを INSERT する。
    /// UPDATE・DELETE は Append-only 原則により提供しない。
    pub async fn insert(
        &self,
        cmd: InsertAuthLogCmd,
    ) -> Result<(), wnav_domain::error::DomainError> {
        sqlx::query(
            r#"
            INSERT INTO auth_logs (
                log_id, user_id, login_id,
                event_type, ip_address, user_agent, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(cmd.log_id)
        .bind(cmd.user_id)
        .bind(cmd.login_id)
        .bind(cmd.event_type.as_str())
        .bind(cmd.ip_address)
        .bind(cmd.user_agent)
        .bind(cmd.created_at)
        .execute(&self.pool)
        .await
        .map_err(crate::error::map_sqlx)?;

        Ok(())
    }
}
