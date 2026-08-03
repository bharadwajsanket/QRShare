use axum_extra::extract::cookie::{Cookie, CookieJar};
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct AuthConfig {
    pub password: Option<String>,
    pub session_token: String,
}

impl AuthConfig {
    pub fn new(password: Option<String>) -> Self {
        Self {
            password,
            session_token: Uuid::new_v4().to_string(),
        }
    }

    /// Verifies if the request cookie matches the active server session token.
    pub fn is_authenticated(&self, jar: &CookieJar) -> bool {
        if self.password.is_none() {
            return true;
        }
        if let Some(cookie) = jar.get("qrshare_session") {
            cookie.value() == self.session_token
        } else {
            false
        }
    }

    /// Creates a secure HTTP-Only cookie containing the session token.
    pub fn create_session_cookie(&self) -> Cookie<'static> {
        Cookie::build(("qrshare_session", self.session_token.clone()))
            .path("/")
            .http_only(true)
            .same_site(axum_extra::extract::cookie::SameSite::Lax) // Allow navigation from scans
            .into()
    }
}

/// Constant-time comparison to prevent timing attacks when verifying passwords.
pub fn constant_time_compare(a: &str, b: &str) -> bool {
    let a_bytes = a.as_bytes();
    let b_bytes = b.as_bytes();
    if a_bytes.len() != b_bytes.len() {
        return false;
    }
    let mut result = 0;
    for (x, y) in a_bytes.iter().zip(b_bytes.iter()) {
        result |= x ^ y;
    }
    result == 0
}
