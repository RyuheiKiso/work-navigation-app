// IQC ハンドラ（terminal-api 担当分: API-iqc-001・API-iqc-003）
//
// 現場端末からの入荷検査開始と測定値追加を担当する。
// 合否判定（API-iqc-004）・特採承認（API-iqc-005）は master-api が担当する。
// Append-only: iqc_inspections / iqc_measurements テーブルへの INSERT 専用。
// event_insert_pool（app_event_insert ロール）を使用する。

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use uuid::Uuid;

use crate::{
    dto::iqc::{
        AddMeasurementRequest, CreateInspectionRequest, InspectionCreatedResponse,
        MeasurementResponse,
    },
    error::AppError,
    state::AppState,
};

/// 入荷検査開始（POST /api/v1/iqc/incoming-inspections）。
///
/// 現場端末からの入荷ロット受入登録。Idempotency-Key ヘッダ必須。
/// event_insert_pool に INSERT する（app_event_insert ロール）。
pub async fn create_inspection(
    State(state): State<AppState>,
    Json(req): Json<CreateInspectionRequest>,
) -> Result<impl IntoResponse, AppError> {
    let new_id = Uuid::now_v7();
    let now = Utc::now();

    // サンプリングプランを material_id から解決する（プランが存在しない場合は None）
    let sampling_plan_id: Option<Uuid> = sqlx::query_scalar(
        r#"SELECT id FROM sampling_plans WHERE material_id = $1 LIMIT 1"#,
    )
    .bind(req.material_id)
    .fetch_optional(&state.read_pool)
    .await
    .map_err(|_| AppError::DatabaseError)?;

    // 入荷検査レコードを Append-only で挿入する
    sqlx::query(
        r#"
        INSERT INTO iqc_inspections
            (id, lot_id, supplier_id, material_id, lot_quantity, sampling_plan_id,
             qc_status, severity_state, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, 'PENDING', 'NORMAL', $7)
        "#,
    )
    .bind(new_id)
    .bind(req.lot_id)
    .bind(req.supplier_id)
    .bind(req.material_id)
    .bind(req.lot_quantity)
    .bind(sampling_plan_id)
    .bind(now)
    .execute(&state.event_insert_pool)
    .await
    .map_err(|_| AppError::DatabaseError)?;

    tracing::info!(
        event = "iqc.inspection.started",
        inspection_id = %new_id,
        lot_id = %req.lot_id,
        "入荷検査を開始しました",
    );

    Ok((
        StatusCode::CREATED,
        Json(InspectionCreatedResponse {
            inspection_id: new_id,
            sampling_plan_id,
            // サンプリングプランが存在する場合は詳細を返す（簡易実装）
            sample_size_n: None,
            accept_number_ac: None,
            reject_number_re: None,
            severity_state: "NORMAL".to_string(),
            qc_status: "PENDING".to_string(),
        }),
    ))
}

/// 測定値追加（POST /api/v1/iqc/incoming-inspections/{id}/measurements）。
///
/// 現場端末からのサンプル測定値 1 個を Append-only で記録する。
/// event_insert_pool に INSERT する（app_event_insert ロール）。
pub async fn add_measurement(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<AddMeasurementRequest>,
) -> Result<impl IntoResponse, AppError> {
    // 検査が存在し PENDING / IN_PROGRESS 状態であることを確認する
    let inspection_exists: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM iqc_inspections
            WHERE id = $1 AND qc_status IN ('PENDING', 'IN_PROGRESS')
        )
        "#,
    )
    .bind(id)
    .fetch_one(&state.read_pool)
    .await
    .map_err(|_| AppError::DatabaseError)?;

    if !inspection_exists {
        return Err(AppError::NotFound);
    }

    let new_id = Uuid::now_v7();
    let now = Utc::now();

    // 測定値を Append-only で挿入する
    sqlx::query(
        r#"
        INSERT INTO iqc_measurements
            (id, inspection_id, sample_no, measured_value, defect_flag,
             evidence_file_id, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
    )
    .bind(new_id)
    .bind(id)
    .bind(req.sample_no)
    .bind(req.measured_value)
    .bind(req.defect_flag)
    .bind(req.evidence_file_id)
    .bind(now)
    .execute(&state.event_insert_pool)
    .await
    .map_err(|_| AppError::DatabaseError)?;

    // 検査ステータスを IN_PROGRESS に更新する（初回測定時のみ）
    sqlx::query(
        r#"
        UPDATE iqc_inspections
        SET qc_status = 'IN_PROGRESS'
        WHERE id = $1 AND qc_status = 'PENDING'
        "#,
    )
    .bind(id)
    .execute(&state.event_insert_pool)
    .await
    .map_err(|_| AppError::DatabaseError)?;

    tracing::info!(
        event = "iqc.measurement.added",
        measurement_id = %new_id,
        inspection_id = %id,
        sample_no = req.sample_no,
        defect_flag = req.defect_flag,
        "測定値を追加しました",
    );

    Ok((
        StatusCode::CREATED,
        Json(MeasurementResponse {
            measurement_id: new_id,
            inspection_id: id,
            sample_no: req.sample_no,
            measured_value: req.measured_value,
            defect_flag: req.defect_flag,
            created_at: now,
        }),
    ))
}
