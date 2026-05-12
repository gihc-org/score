//! SQLx row types — one struct per database table.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(sqlx::FromRow, Serialize, Clone)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    #[serde(skip)]
    pub password_hash: String,
    pub email: Option<String>,
    pub email_verified: bool,
    #[serde(skip)]
    pub verification_token: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(sqlx::FromRow, Serialize)]
pub struct Review {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub description: String,
    pub link: Option<String>,
    pub rating: i16,
    pub created_at: DateTime<Utc>,
}

/// Review joined with the author's username.
#[derive(sqlx::FromRow, Serialize)]
pub struct ReviewWithAuthor {
    pub id: Uuid,
    pub user_id: Uuid,
    pub username: String,
    pub title: String,
    pub description: String,
    pub link: Option<String>,
    pub rating: i16,
    pub created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct CreateReview {
    pub title: String,
    pub description: String,
    pub link: Option<String>,
    pub rating: i16,
}
