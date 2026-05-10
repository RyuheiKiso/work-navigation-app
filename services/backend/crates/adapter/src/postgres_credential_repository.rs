//! PostgreSQL 認証情報リポジトリ
//!
//! 対応 §: ロードマップ §10.5 §10.5.1 §27 F-006

// ドメイン
use wna_domain::{Credential, PasswordHash, User, UserId};
// ユースケース trait
use wna_usecase::CredentialRepository;
// sqlx
use sqlx::PgPool;
// 既存エラー型を流用
use crate::postgres_repository::PostgresRepositoryError;

/// PostgreSQL 実装の認証情報リポジトリ
#[derive(Clone)]
pub struct PostgresCredentialRepository {
    /// 接続プール
    pool: PgPool,
}

impl PostgresCredentialRepository {
    /// プールから構築する
    #[must_use]
    pub const fn new(pool: PgPool) -> Self {
        // pool を保持
        Self { pool }
    }

    /// 認証情報を upsert する（seed／管理操作専用の補助 API）
    ///
    /// `CredentialRepository` trait には載せない。
    /// trait に upsert を入れると本番 DI でも書き換え経路が露出するため、
    /// 書き込みは inherent に閉じて呼び出し側を限定する（§11.4 信頼境界）。
    ///
    /// # Errors
    /// 接続失敗・制約違反等で sqlx エラーを返す。
    pub async fn upsert_credential(
        &self,
        user_id: &str,
        display_name: &str,
        password_hash_phc: &str,
        enabled: bool,
    ) -> Result<(), PostgresRepositoryError> {
        sqlx::query(
            "INSERT INTO credentials (user_id, display_name, enabled, password_hash) \
             VALUES ($1, $2, $3, $4) \
             ON CONFLICT (user_id) DO UPDATE SET \
               display_name = EXCLUDED.display_name, \
               enabled = EXCLUDED.enabled, \
               password_hash = EXCLUDED.password_hash, \
               updated_at = NOW()",
        )
        .bind(user_id)
        .bind(display_name)
        .bind(enabled)
        .bind(password_hash_phc)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

impl CredentialRepository for PostgresCredentialRepository {
    type Error = PostgresRepositoryError;

    async fn find_credential(
        &self,
        user_id: &UserId,
    ) -> Result<Option<Credential>, Self::Error> {
        // password_hash を取得
        let row: Option<(String, String)> = sqlx::query_as(
            "SELECT user_id, password_hash FROM credentials WHERE user_id = $1",
        )
        .bind(user_id.as_str())
        .fetch_optional(&self.pool)
        .await?;
        // 未存在
        let Some((db_user_id, phc)) = row else {
            return Ok(None);
        };
        // 値オブジェクトに射影
        let id = UserId::new(db_user_id)
            .map_err(PostgresRepositoryError::Domain)?;
        let hash = PasswordHash::from_phc(phc).map_err(|_| {
            PostgresRepositoryError::Domain(
                wna_domain::DomainError::InvalidIdentifier("PasswordHash"),
            )
        })?;
        // Credential を構築
        Ok(Some(Credential::new(id, hash)))
    }

    async fn find_user(&self, user_id: &UserId) -> Result<Option<User>, Self::Error> {
        // 表示名と enabled を取得
        let row: Option<(String, String, bool)> = sqlx::query_as(
            "SELECT user_id, display_name, enabled FROM credentials WHERE user_id = $1",
        )
        .bind(user_id.as_str())
        .fetch_optional(&self.pool)
        .await?;
        // 未存在
        let Some((db_user_id, dn, enabled)) = row else {
            return Ok(None);
        };
        // ID 値オブジェクトに射影
        let id = UserId::new(db_user_id).map_err(PostgresRepositoryError::Domain)?;
        // User Aggregate を構築
        let mut user = User::new(id, dn).map_err(PostgresRepositoryError::Domain)?;
        // 無効化なら disable を呼ぶ
        if !enabled {
            user.disable();
        }
        // 完成
        Ok(Some(user))
    }
}
