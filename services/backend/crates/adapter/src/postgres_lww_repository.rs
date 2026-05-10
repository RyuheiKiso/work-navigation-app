//! PostgreSQL LWW-Register（ユーザー設定の決定的競合解決）
//!
//! 対応 §: ロードマップ §10.6 §10.6.1 §10.6.2 §27 F-002 §29 R-016
//!
//! Lamport timestamp と device_id の lex 順比較で値を決定する（INV-02）。

// ドメイン
use wna_domain::{DeviceId, LamportTimestamp};
// sqlx
use sqlx::PgPool;
// 既存エラー
use crate::postgres_repository::PostgresRepositoryError;

/// LWW のエントリ DTO
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LwwEntry {
    /// 設定キー
    pub key: String,
    /// 値（JSON 文字列）
    pub value: String,
    /// Lamport timestamp
    pub lamport: LamportTimestamp,
    /// 端末 ID
    pub device_id: DeviceId,
}

/// PostgreSQL LWW-Register
#[derive(Clone)]
pub struct PostgresLwwRepository {
    /// 接続プール
    pool: PgPool,
}

impl PostgresLwwRepository {
    /// プールから構築する
    #[must_use]
    pub const fn new(pool: PgPool) -> Self {
        // pool を保持
        Self { pool }
    }

    /// LWW で更新を試みる
    ///
    /// 既存値の (lamport, device_id) と比較し、新値が lex 順で大きい場合のみ書き換える。
    /// 戻り値は「実際に書き換わったか」。同値以下の場合は無視（重要: 戻り値で判定可能）。
    pub async fn upsert(&self, entry: &LwwEntry) -> Result<bool, PostgresRepositoryError> {
        // INSERT ... ON CONFLICT DO UPDATE で条件付き更新
        // (lamport, device_id) の lex 順を SQL で表現する
        let result = sqlx::query(
            "INSERT INTO user_settings (setting_key, value, lamport, device_id) \
             VALUES ($1, $2, $3, $4) \
             ON CONFLICT (setting_key) DO UPDATE SET \
               value = EXCLUDED.value, \
               lamport = EXCLUDED.lamport, \
               device_id = EXCLUDED.device_id, \
               updated_at = NOW() \
             WHERE \
               EXCLUDED.lamport > user_settings.lamport \
               OR (EXCLUDED.lamport = user_settings.lamport AND EXCLUDED.device_id > user_settings.device_id)",
        )
        .bind(&entry.key)
        .bind(&entry.value)
        .bind(i64::try_from(entry.lamport.value()).unwrap_or(0))
        .bind(entry.device_id.as_str())
        .execute(&self.pool)
        .await?;
        // 影響行数で書き換わったか判定
        Ok(result.rows_affected() > 0)
    }

    /// 現在値を取得する
    pub async fn get(&self, key: &str) -> Result<Option<LwwEntry>, PostgresRepositoryError> {
        // 単純取得
        let row: Option<(String, String, i64, String)> = sqlx::query_as(
            "SELECT setting_key, value, lamport, device_id \
             FROM user_settings WHERE setting_key = $1",
        )
        .bind(key)
        .fetch_optional(&self.pool)
        .await?;
        // 未存在
        let Some((k, v, l, d)) = row else {
            return Ok(None);
        };
        // 値オブジェクトに射影
        let lamport = LamportTimestamp::from_u64(u64::try_from(l).unwrap_or(0));
        let device = DeviceId::new(d).map_err(PostgresRepositoryError::Domain)?;
        // エントリ
        Ok(Some(LwwEntry {
            key: k,
            value: v,
            lamport,
            device_id: device,
        }))
    }
}
