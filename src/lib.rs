pub mod auth;
pub mod captcha;
pub mod config;
pub mod email;
pub mod models;
pub mod routes;

use axum::{
    http::{HeaderValue, Method, Request},
    routing::{delete, get, post},
    Router,
};
use sqlx::PgPool;
use std::sync::Arc;
use tower_governor::{
    governor::GovernorConfigBuilder, key_extractor::KeyExtractor, GovernorError, GovernorLayer,
};
use tower_http::cors::{Any, CorsLayer};

pub use config::Config;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub config: Config,
    pub http: reqwest::Client,
}

#[derive(Clone)]
struct ForwardedIpExtractor;

impl KeyExtractor for ForwardedIpExtractor {
    type Key = String;

    fn extract<T>(&self, req: &Request<T>) -> Result<Self::Key, GovernorError> {
        let ip = req
            .headers()
            .get("x-forwarded-for")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.split(',').next())
            .map(str::trim)
            .unwrap_or("0.0.0.0");
        Ok(ip.to_string())
    }
}

pub fn build_app(state: AppState) -> Router {
    let cors = build_cors(&state.config.allowed_origin);

    let auth_rate_limit = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(3)
            .burst_size(30)
            .key_extractor(ForwardedIpExtractor)
            .finish()
            .unwrap(),
    );

    let auth_routes = Router::new()
        .route("/auth/register", post(routes::auth::register))
        .route("/auth/token", post(routes::auth::login))
        .layer(GovernorLayer { config: auth_rate_limit });

    let v1 = Router::new()
        .merge(auth_routes)
        .route("/auth/me", get(routes::auth::me).delete(routes::auth::delete_me))
        .route("/auth/verify", get(routes::auth::verify))
        .route("/reviews", get(routes::reviews::list).post(routes::reviews::create))
        .route("/reviews/:id", delete(routes::reviews::delete));

    Router::new()
        .nest("/v1", v1)
        .layer(cors)
        .with_state(state)
}

pub fn build_cors(allowed_origin: &str) -> CorsLayer {
    let methods = [Method::GET, Method::POST, Method::PUT, Method::DELETE];
    let headers = [
        axum::http::header::AUTHORIZATION,
        axum::http::header::CONTENT_TYPE,
    ];

    if allowed_origin == "*" {
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(methods)
            .allow_headers(headers)
    } else {
        CorsLayer::new()
            .allow_origin(
                allowed_origin
                    .parse::<HeaderValue>()
                    .expect("Invalid ALLOWED_ORIGIN"),
            )
            .allow_methods(methods)
            .allow_headers(headers)
    }
}
