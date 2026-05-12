//! Environment-based configuration loaded once at startup.

#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_expire_hours: i64,
    pub allowed_origin: String,
    pub turnstile_secret: String,
    pub resend_api_key: String,
    pub resend_from: String,
    pub base_url: String,
    pub frontend_url: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            database_url: std::env::var("DATABASE_URL").expect("DATABASE_URL required"),
            jwt_secret: std::env::var("JWT_SECRET").expect("JWT_SECRET required"),
            jwt_expire_hours: std::env::var("JWT_EXPIRE_HOURS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(24),
            allowed_origin: std::env::var("ALLOWED_ORIGIN").unwrap_or_else(|_| "*".into()),
            turnstile_secret: std::env::var("TURNSTILE_SECRET").unwrap_or_default(),
            resend_api_key: std::env::var("RESEND_API_KEY").unwrap_or_default(),
            resend_from: std::env::var("RESEND_FROM")
                .unwrap_or_else(|_| "noreply@example.com".into()),
            base_url: std::env::var("BASE_URL")
                .unwrap_or_else(|_| "http://localhost:8080".into()),
            frontend_url: std::env::var("FRONTEND_URL")
                .unwrap_or_else(|_| "http://localhost:8080".into()),
        }
    }
}
