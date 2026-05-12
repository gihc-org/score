//! `/auth/*` — register, login, identity, and email verification endpoints.

use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::Redirect,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    auth::{authenticate, create_token, hash_password, verify_password},
    captcha, email,
    models::User,
    AppState,
};

type ApiError = (StatusCode, Json<serde_json::Value>);

fn err(status: StatusCode, msg: &str) -> ApiError {
    (status, Json(serde_json::json!({"detail": msg})))
}

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
    pub email: String,
    pub turnstile_token: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
}

#[derive(Deserialize)]
pub struct VerifyQuery {
    token: Uuid,
}

pub async fn register(
    State(state): State<AppState>,
    Json(body): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    if body.username.len() < 2 || body.username.len() > 50 {
        return Err(err(StatusCode::BAD_REQUEST, "Brugernavn skal være 2–50 tegn"));
    }
    if body.password.len() < 8 || body.password.len() > 128 {
        return Err(err(StatusCode::BAD_REQUEST, "Adgangskode skal være 8–128 tegn"));
    }
    if body.email.len() > 254 {
        return Err(err(StatusCode::BAD_REQUEST, "Email er for lang"));
    }

    if !captcha::verify(&state.http, &state.config.turnstile_secret, &body.turnstile_token).await {
        return Err(err(StatusCode::BAD_REQUEST, "CAPTCHA-validering fejlede"));
    }

    if sqlx::query("SELECT id FROM users WHERE username = $1")
        .bind(&body.username)
        .fetch_optional(&state.db)
        .await
        .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, "Database error"))?
        .is_some()
    {
        return Err(err(StatusCode::BAD_REQUEST, "Brugernavnet er allerede taget"));
    }

    if sqlx::query("SELECT id FROM users WHERE email = $1")
        .bind(&body.email)
        .fetch_optional(&state.db)
        .await
        .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, "Database error"))?
        .is_some()
    {
        return Err(err(StatusCode::BAD_REQUEST, "Kunne ikke oprette konto med disse oplysninger"));
    }

    let auto_verify = state.config.resend_api_key.is_empty();
    let verification_token = if auto_verify { None } else { Some(Uuid::new_v4()) };

    let user = sqlx::query_as::<_, User>(
        "INSERT INTO users (username, password_hash, email, email_verified, verification_token)
         VALUES ($1, $2, $3, $4, $5) RETURNING *",
    )
    .bind(&body.username)
    .bind(hash_password(&body.password))
    .bind(&body.email)
    .bind(auto_verify)
    .bind(verification_token)
    .fetch_one(&state.db)
    .await
    .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, "Kunne ikke oprette bruger"))?;

    if let Some(token) = verification_token {
        let verify_url = format!("{}/auth/verify?token={}", state.config.base_url, token);
        email::send_verification(
            &state.http,
            &state.config.resend_api_key,
            &state.config.resend_from,
            &body.email,
            &verify_url,
        )
        .await
        .map_err(|e| {
            tracing::error!("Failed to send verification email: {e}");
            err(StatusCode::INTERNAL_SERVER_ERROR, "Kunne ikke sende verifikationsmail")
        })?;
    }

    let msg = if auto_verify {
        "Konto oprettet — du kan nu logge ind."
    } else {
        "Konto oprettet — tjek din email og klik på bekræftelseslinket."
    };

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({"id": user.id, "username": user.username, "message": msg})),
    ))
}

pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> Result<Json<TokenResponse>, ApiError> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = $1")
        .bind(&body.username)
        .fetch_optional(&state.db)
        .await
        .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, "Database error"))?
        .ok_or_else(|| err(StatusCode::UNAUTHORIZED, "Forkert brugernavn eller adgangskode"))?;

    if !verify_password(&body.password, &user.password_hash) {
        return Err(err(StatusCode::UNAUTHORIZED, "Forkert brugernavn eller adgangskode"));
    }

    if !user.email_verified {
        return Err(err(StatusCode::FORBIDDEN, "Bekræft din email før du logger ind"));
    }

    let token = create_token(user.id, &state.config.jwt_secret, state.config.jwt_expire_hours);
    Ok(Json(TokenResponse {
        access_token: token,
        token_type: "bearer".into(),
    }))
}

pub async fn me(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<User>, ApiError> {
    let user = authenticate(&state, &headers).await?;
    Ok(Json(user))
}

pub async fn delete_me(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<StatusCode, ApiError> {
    let user = authenticate(&state, &headers).await?;

    sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user.id)
        .execute(&state.db)
        .await
        .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, "Database error"))?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn verify(
    State(state): State<AppState>,
    Query(params): Query<VerifyQuery>,
) -> Result<Redirect, ApiError> {
    let updated = sqlx::query(
        "UPDATE users SET email_verified = TRUE, verification_token = NULL
         WHERE verification_token = $1",
    )
    .bind(params.token)
    .execute(&state.db)
    .await
    .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, "Database error"))?;

    if updated.rows_affected() == 0 {
        return Err(err(StatusCode::BAD_REQUEST, "Ugyldigt eller allerede brugt verifikationslink"));
    }

    let redirect_url = format!("{}?verified=true", state.config.frontend_url);
    Ok(Redirect::to(&redirect_url))
}
