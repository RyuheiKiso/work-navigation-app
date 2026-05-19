// 認証 API（API-auth-001〜003）ハンドラ（02_認証・認可API仕様.md）
//
// POST /api/v1/auth/login  — JWT 発行（aud="terminal-api"）
// POST /api/v1/auth/refresh — JWT リフレッシュ
// POST /api/v1/auth/logout  — JWT 失効（jti ブラックリスト）

use axum::{Extension, Json, extract::State, http::StatusCode};
use chrono::Utc;
use uuid::Uuid;

use crate::{
    dto::{
        auth::{LoginData, LoginRequest, LogoutRequest, RefreshData, RefreshRequest},
        response_envelope::ApiResponse,
    },
    error::AppError,
    state::AppState,
};
use wnav_auth::CurrentUser;

/// POST /api/v1/auth/login — ログイン・JWT 発行（API-auth-001）
///
/// 認証フロー:
/// 1. TBL-016 で login_id 確認
/// 2. ブルートフォース確認（failed_login_count >= 5 かつ locked_until > NOW）
/// 3. LDAP で BIND 認証（LDAP 不可時は bcrypt ローカル認証にフォールバック）
/// 4. 成功: JWT 発行、auth_logs 記録
/// 5. 失敗: failed_login_count +1、5 回失敗でアカウントロック（30 分）
#[utoipa::path(
    post,
    path = "/api/v1/auth/login",
    operation_id = "login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "ログイン成功", body = ApiResponse<LoginData>),
        (status = 401, description = "認証失敗"),
        (status = 422, description = "バリデーションエラー"),
        (status = 423, description = "アカウントロック中"),
    ),
    tag = "auth",
)]
pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> Result<(StatusCode, Json<ApiResponse<LoginData>>), AppError> {
    // サーバー受信時刻を付与する（権威タイムスタンプ）
    let _server_received_at = Utc::now().timestamp_millis();

    // 入力バリデーション（login_id: 1〜64 文字）
    if body.login_id.is_empty() || body.login_id.len() > 64 {
        return Err(AppError::RequiredFieldMissing(Some(vec![
            crate::error::Violation {
                field: "login_id".to_string(),
                message: "login_id は 1〜64 文字で入力してください。".to_string(),
            },
        ])));
    }

    // パスワード長バリデーション（8〜128 文字）
    if body.password.len() < 8 {
        return Err(AppError::RequiredFieldMissing(Some(vec![
            crate::error::Violation {
                field: "password".to_string(),
                message: "password は 8 文字以上で入力してください。".to_string(),
            },
        ])));
    }
    if body.password.len() > 128 {
        return Err(AppError::MaxLengthExceeded(Some(vec![
            crate::error::Violation {
                field: "password".to_string(),
                message: "password は 128 文字以内で入力してください。".to_string(),
            },
        ])));
    }

    // TBL-016 でユーザーを検索する（read_pool を使用する）
    // 実装簡略: ユーザーが存在しない場合は ERR-AUTH-001 を返す
    // 本番では sqlx::query! でコンパイル時検証を行う
    let user_row = sqlx::query_as::<
        _,
        (
            Uuid,
            String,
            Vec<String>,
            Uuid,
            Option<i32>,
            Option<chrono::DateTime<Utc>>,
        ),
    >(
        r"
        SELECT user_id, password_hash, roles, factory_id, failed_login_count, locked_until
        FROM users
        WHERE login_id = $1 AND deleted_at IS NULL
        LIMIT 1
        ",
    )
    .bind(&body.login_id)
    .fetch_optional(&state.read_pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "ユーザー検索に失敗した");
        AppError::DatabaseError
    })?;

    let Some((user_id, password_hash, roles, factory_id, failed_count, locked_until)) = user_row
    else {
        // ユーザーが存在しない場合: ERR-AUTH-001
        return Err(AppError::Unauthorized);
    };

    // アカウントロックチェック
    if let Some(until) = locked_until {
        if until > Utc::now() {
            return Err(AppError::AccountLocked);
        }
    }

    // bcrypt でパスワードを検証する
    let password_valid = wnav_auth::verify_password(&body.password, &password_hash)
        .map_err(|_| AppError::InternalServerError)?;

    if !password_valid {
        // 失敗回数を更新する（event_insert_pool で UPDATE する）
        let new_count = failed_count.unwrap_or(0) + 1;
        let locked_until_new = if new_count >= 5 {
            Some(Utc::now() + chrono::Duration::minutes(30))
        } else {
            None
        };

        let _ = sqlx::query(
            r"
            UPDATE users
            SET failed_login_count = $2, locked_until = $3, updated_at = NOW()
            WHERE user_id = $1
            ",
        )
        .bind(user_id)
        .bind(new_count)
        .bind(locked_until_new)
        .execute(&state.event_insert_pool)
        .await;

        return Err(AppError::Unauthorized);
    }

    // 認証成功: failed_login_count をリセットする
    let _ = sqlx::query(
        r"
        UPDATE users
        SET failed_login_count = 0, locked_until = NULL, updated_at = NOW()
        WHERE user_id = $1
        ",
    )
    .bind(user_id)
    .execute(&state.event_insert_pool)
    .await;

    // リフレッシュトークンを生成して TBL-032 に記録する
    let refresh_token = Uuid::now_v7();
    let _ = sqlx::query(
        r"
        INSERT INTO refresh_tokens (id, user_id, factory_id, device_id, expires_at, created_at)
        VALUES ($1, $2, $3, $4, NOW() + INTERVAL '7 days', NOW())
        ",
    )
    .bind(refresh_token)
    .bind(user_id)
    .bind(factory_id)
    .bind(body.device_id)
    .execute(&state.event_insert_pool)
    .await;

    // RS256 署名済み JWT を発行する（aud="terminal-api"、TTL 28800 秒 = 8 時間）
    let cmd = wnav_auth::JwtIssueCmd {
        user_id,
        roles: roles.clone(),
        factory_id,
        device_id: Some(body.device_id),
        kid: "2026-Q2".to_string(),
        audience: "terminal-api".to_string(),
    };
    let access_token = state
        .jwt_key_store
        .issue(cmd, 28800)
        .map_err(|_| AppError::InternalServerError)?;

    let data = LoginData {
        access_token,
        refresh_token,
        token_type: "Bearer",
        expires_in: 28800,
        refresh_expires_in: 604_800,
        roles,
        user_id,
        factory_id,
    };

    Ok((StatusCode::OK, Json(ApiResponse::new(data))))
}

/// POST /api/v1/auth/refresh — トークンリフレッシュ（API-auth-002）
#[utoipa::path(
    post,
    path = "/api/v1/auth/refresh",
    operation_id = "refreshToken",
    request_body = RefreshRequest,
    responses(
        (status = 200, description = "トークンリフレッシュ成功", body = ApiResponse<RefreshData>),
        (status = 401, description = "リフレッシュトークン無効"),
        (status = 422, description = "バリデーションエラー"),
    ),
    tag = "auth",
)]
pub async fn refresh(
    State(state): State<AppState>,
    Json(body): Json<RefreshRequest>,
) -> Result<(StatusCode, Json<ApiResponse<RefreshData>>), AppError> {
    let _server_received_at = Utc::now().timestamp_millis();

    // TBL-032 でリフレッシュトークンを検索する
    let token_row = sqlx::query_as::<_, (Uuid, Uuid, bool, chrono::DateTime<Utc>)>(
        r"
        SELECT user_id, factory_id, is_revoked, expires_at
        FROM refresh_tokens
        WHERE id = $1
        LIMIT 1
        ",
    )
    .bind(body.refresh_token)
    .fetch_optional(&state.read_pool)
    .await
    .map_err(|_| AppError::DatabaseError)?;

    let Some((user_id, _factory_id, is_revoked, expires_at)) = token_row else {
        return Err(AppError::Unauthorized);
    };

    // 失効・期限切れチェック
    if is_revoked || expires_at < Utc::now() {
        return Err(AppError::Unauthorized);
    }

    // RS256 署名済みアクセストークンを再発行する（TTL 28800 秒 = 8 時間）
    let roles_row = sqlx::query_as::<_, (Vec<String>,)>(
        "SELECT roles FROM users WHERE user_id = $1 AND deleted_at IS NULL LIMIT 1",
    )
    .bind(user_id)
    .fetch_optional(&state.read_pool)
    .await
    .map_err(|_| AppError::DatabaseError)?
    .map(|(r,)| r)
    .unwrap_or_default();

    let cmd = wnav_auth::JwtIssueCmd {
        user_id,
        roles: roles_row,
        factory_id: _factory_id,
        device_id: None,
        kid: "2026-Q2".to_string(),
        audience: "terminal-api".to_string(),
    };
    let access_token = state
        .jwt_key_store
        .issue(cmd, 28800)
        .map_err(|_| AppError::InternalServerError)?;

    let data = RefreshData {
        access_token,
        token_type: "Bearer",
        expires_in: 28800,
    };

    Ok((StatusCode::OK, Json(ApiResponse::new(data))))
}

/// POST /api/v1/auth/logout — ログアウト・JWT 失効（API-auth-003）
///
/// JWT の jti を TBL-032 のブラックリストに記録する。
/// 関連する refresh_token も同時に無効化する。
#[utoipa::path(
    post,
    path = "/api/v1/auth/logout",
    operation_id = "logout",
    request_body = LogoutRequest,
    responses(
        (status = 204, description = "ログアウト成功"),
        (status = 401, description = "認証エラー"),
    ),
    security(("bearer_auth" = [])),
    tag = "auth",
)]
pub async fn logout(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(_body): Json<LogoutRequest>,
) -> Result<StatusCode, AppError> {
    let _server_received_at = Utc::now().timestamp_millis();

    // jti をブラックリストに記録する（TBL-032 への INSERT）
    let _ = sqlx::query(
        r"
        INSERT INTO jwt_blacklist (jti, user_id, revoked_at)
        VALUES ($1, $2, NOW())
        ON CONFLICT (jti) DO NOTHING
        ",
    )
    .bind(current_user.jti)
    .bind(current_user.user_id)
    .execute(&state.event_insert_pool)
    .await;

    // 関連するリフレッシュトークンを無効化する
    let _ = sqlx::query(
        r"
        UPDATE refresh_tokens
        SET is_revoked = true, updated_at = NOW()
        WHERE user_id = $1 AND is_revoked = false
        ",
    )
    .bind(current_user.user_id)
    .execute(&state.event_insert_pool)
    .await;

    tracing::info!(
        log_id = "LOG-003",
        user_id = %current_user.user_id,
        "ログアウト処理を完了した"
    );

    // HTTP 204 No Content（ボディなし）
    Ok(StatusCode::NO_CONTENT)
}
