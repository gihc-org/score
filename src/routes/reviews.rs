//! `/v1/reviews` — list, create, and delete reviews.

use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use uuid::Uuid;

use crate::{
    auth::authenticate,
    models::{CreateReview, ReviewWithAuthor},
    AppState,
};

type ApiError = (StatusCode, Json<serde_json::Value>);

fn err(status: StatusCode, msg: &str) -> ApiError {
    (status, Json(serde_json::json!({"detail": msg})))
}

/// `GET /v1/reviews` — returns all reviews, newest first.
pub async fn list(
    State(state): State<AppState>,
) -> Result<Json<Vec<ReviewWithAuthor>>, ApiError> {
    let reviews = sqlx::query_as::<_, ReviewWithAuthor>(
        "SELECT r.id, r.user_id, u.username, r.title, r.description, r.link, r.rating, r.created_at
         FROM reviews r
         JOIN users u ON u.id = r.user_id
         ORDER BY r.created_at DESC",
    )
    .fetch_all(&state.db)
    .await
    .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, "Database error"))?;

    Ok(Json(reviews))
}

/// `POST /v1/reviews` — creates a review for the authenticated user.
pub async fn create(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<CreateReview>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    let user = authenticate(&state, &headers).await?;

    if body.title.is_empty() || body.title.len() > 200 {
        return Err(err(StatusCode::BAD_REQUEST, "Titel skal være 1–200 tegn"));
    }
    if body.description.is_empty() {
        return Err(err(StatusCode::BAD_REQUEST, "Beskrivelse må ikke være tom"));
    }
    if body.description.len() > 10_000 {
        return Err(err(StatusCode::BAD_REQUEST, "Beskrivelse må maks. være 10.000 tegn"));
    }
    if !(1..=5).contains(&body.rating) {
        return Err(err(StatusCode::BAD_REQUEST, "Rating skal være mellem 1 og 5"));
    }
    if let Some(ref link) = body.link {
        if link.len() > 2000 {
            return Err(err(StatusCode::BAD_REQUEST, "Link er for langt"));
        }
    }

    let review = sqlx::query_as::<_, ReviewWithAuthor>(
        "WITH inserted AS (
             INSERT INTO reviews (user_id, title, description, link, rating)
             VALUES ($1, $2, $3, $4, $5)
             RETURNING *
         )
         SELECT i.id, i.user_id, u.username, i.title, i.description, i.link, i.rating, i.created_at
         FROM inserted i
         JOIN users u ON u.id = i.user_id",
    )
    .bind(user.id)
    .bind(&body.title)
    .bind(&body.description)
    .bind(&body.link)
    .bind(body.rating)
    .fetch_one(&state.db)
    .await
    .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, "Kunne ikke oprette anmeldelse"))?;

    Ok((StatusCode::CREATED, Json(serde_json::json!(review))))
}

/// `DELETE /v1/reviews/:id` — deletes a review owned by the authenticated user.
pub async fn delete(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    let user = authenticate(&state, &headers).await?;

    let result = sqlx::query(
        "DELETE FROM reviews WHERE id = $1 AND user_id = $2",
    )
    .bind(id)
    .bind(user.id)
    .execute(&state.db)
    .await
    .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, "Database error"))?;

    if result.rows_affected() == 0 {
        return Err(err(StatusCode::NOT_FOUND, "Anmeldelse ikke fundet eller du ejer den ikke"));
    }

    Ok(StatusCode::NO_CONTENT)
}
