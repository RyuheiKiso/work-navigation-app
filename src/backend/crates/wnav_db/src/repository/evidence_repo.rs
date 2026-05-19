// PgEvidenceRepository — TBL-009 evidence_files の sqlx 実装
// file_hash は BYTEA 32 バイトとして格納する（ALCOA+ Accurate 原則）。
// SQLX_PREPARE_REQUIRED: cargo sqlx prepare を実行してからビルド可能

use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use wnav_domain::{
    error::DomainError,
    model::evidence::{EvidenceFile, EvidenceType},
    repository::EvidenceRepository,
};

use crate::row_types::EvidenceFileRow;

/// TBL-009 evidence_files のリポジトリ実装。
pub struct PgEvidenceRepository {
    pool: PgPool,
}

impl PgEvidenceRepository {
    /// コネクションプールを受け取ってリポジトリを構築する。
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

/// EvidenceFileRow から EvidenceFile ドメインモデルへの変換。
/// file_hash は Vec<u8> から [u8; 32] に変換する。
impl TryFrom<EvidenceFileRow> for EvidenceFile {
    type Error = DomainError;

    fn try_from(row: EvidenceFileRow) -> Result<Self, Self::Error> {
        let evidence_type = parse_evidence_type(&row.evidence_type)?;
        // BYTEA 32 バイトを固定長配列に変換する
        let file_hash: [u8; 32] = row.file_hash.try_into().map_err(|_| {
            DomainError::Internal("file_hash が 32 バイトではありません".to_string())
        })?;

        Ok(Self {
            evidence_id: row.evidence_id,
            work_execution_id: row.work_execution_id,
            step_id: row.step_id,
            file_hash,
            file_path: row.file_path,
            evidence_type,
            recorded_by: row.recorded_by,
            client_recorded_at: row.client_recorded_at,
            server_received_at: row.server_received_at,
        })
    }
}

/// DB 証拠種別文字列を EvidenceType 列挙型に変換する。
fn parse_evidence_type(s: &str) -> Result<EvidenceType, DomainError> {
    match s {
        "PHOTO" => Ok(EvidenceType::Photo),
        "MEASUREMENT" => Ok(EvidenceType::Measurement),
        "QR_SCAN" => Ok(EvidenceType::QrScan),
        "SIGNATURE" => Ok(EvidenceType::Signature),
        "OTHER" => Ok(EvidenceType::Other),
        other => Err(DomainError::Internal(format!(
            "不明な EvidenceType: {other}"
        ))),
    }
}

/// EvidenceType を DB 格納文字列に変換する。
fn evidence_type_to_str(t: &EvidenceType) -> &'static str {
    match t {
        EvidenceType::Photo => "PHOTO",
        EvidenceType::Measurement => "MEASUREMENT",
        EvidenceType::QrScan => "QR_SCAN",
        EvidenceType::Signature => "SIGNATURE",
        EvidenceType::Other => "OTHER",
    }
}

#[async_trait]
impl EvidenceRepository for PgEvidenceRepository {
    /// 証拠ファイルを INSERT する。
    async fn insert(&self, evidence: EvidenceFile) -> Result<(), DomainError> {
        sqlx::query(
            r#"
            INSERT INTO evidence_files (
                evidence_id, work_execution_id, step_id,
                file_hash, file_path, evidence_type,
                recorded_by, client_recorded_at, server_received_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
        )
        .bind(evidence.evidence_id)
        .bind(evidence.work_execution_id)
        .bind(evidence.step_id)
        .bind(evidence.file_hash.as_ref())
        .bind(evidence.file_path)
        .bind(evidence_type_to_str(&evidence.evidence_type))
        .bind(evidence.recorded_by)
        .bind(evidence.client_recorded_at)
        .bind(evidence.server_received_at)
        .execute(&self.pool)
        .await
        .map_err(crate::error::map_sqlx)?;

        Ok(())
    }

    /// イベント ID に紐づく証拠ファイル一覧を取得する。
    async fn find_by_event(&self, event_id: Uuid) -> Result<Vec<EvidenceFile>, DomainError> {
        // evidence_files は event_id を直接持つテーブル構造を想定する
        let rows = sqlx::query_as::<_, EvidenceFileRow>(
            r#"
            SELECT
                evidence_id, work_execution_id, step_id,
                file_hash, file_path, evidence_type,
                recorded_by, client_recorded_at, server_received_at
            FROM evidence_files
            WHERE event_id = $1
            ORDER BY server_received_at ASC
            "#,
        )
        .bind(event_id)
        .fetch_all(&self.pool)
        .await
        .map_err(crate::error::map_sqlx)?;

        rows.into_iter()
            .map(EvidenceFile::try_from)
            .collect::<Result<Vec<_>, _>>()
    }
}
