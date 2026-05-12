//! Cloudflare Turnstile CAPTCHA verification.

use serde::Deserialize;

#[derive(Deserialize)]
struct TurnstileResponse {
    success: bool,
}

/// Returns `true` immediately if `secret` is empty (local dev — skips CAPTCHA).
pub async fn verify(client: &reqwest::Client, secret: &str, token: &str) -> bool {
    if secret.is_empty() {
        return true;
    }
    let resp = client
        .post("https://challenges.cloudflare.com/turnstile/v0/siteverify")
        .form(&[("secret", secret), ("response", token)])
        .send()
        .await;

    match resp {
        Ok(r) => r.json::<TurnstileResponse>().await.map(|r| r.success).unwrap_or(false),
        Err(_) => false,
    }
}
