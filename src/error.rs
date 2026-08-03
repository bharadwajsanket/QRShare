use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::util;

#[derive(Debug)]
pub enum AppError {
    NotFound(String),
    Forbidden(String),
    Unauthorized(String),
    Internal(String),
    BadRequest(String),
    Io(std::io::Error),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::NotFound(msg) => write!(f, "Not Found: {}", msg),
            AppError::Forbidden(msg) => write!(f, "Forbidden: {}", msg),
            AppError::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
            AppError::Internal(msg) => write!(f, "Internal Error: {}", msg),
            AppError::BadRequest(msg) => write!(f, "Bad Request: {}", msg),
            AppError::Io(err) => write!(f, "I/O Error: {}", err),
        }
    }
}

impl std::error::Error for AppError {}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::Io(err)
    }
}

impl From<clap::Error> for AppError {
    fn from(err: clap::Error) -> Self {
        AppError::BadRequest(err.to_string())
    }
}

// ─── Friendly error metadata ────────────────────────────────────────────────

fn error_meta(status: StatusCode) -> (&'static str, &'static str, &'static str) {
    match status {
        StatusCode::NOT_FOUND => (
            // (title, hint, icon_svg)
            "Not Found",
            "This file or share link doesn't exist.",
            r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
               <circle cx="11" cy="11" r="8"/><path d="m21 21-4.35-4.35"/>
               <path d="M11 8v4M11 16h.01"/>
               </svg>"#,
        ),
        StatusCode::FORBIDDEN => (
            "Share Expired",
            "This share has reached its download limit or expiration time.",
            r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
               <circle cx="12" cy="12" r="10"/>
               <path d="M12 8v4M12 16h.01"/>
               </svg>"#,
        ),
        StatusCode::UNAUTHORIZED => (
            "Access Denied",
            "You need the correct password to access this share.",
            r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
               <rect x="3" y="11" width="18" height="11" rx="2"/>
               <path d="M7 11V7a5 5 0 0 1 10 0v4"/>
               </svg>"#,
        ),
        StatusCode::BAD_REQUEST => (
            "Invalid Request",
            "The request could not be understood. Please try again.",
            r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
               <path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"/>
               <line x1="12" y1="9" x2="12" y2="13"/><line x1="12" y1="17" x2="12.01" y2="17"/>
               </svg>"#,
        ),
        _ => (
            "Something Went Wrong",
            "An unexpected error occurred. Please try again.",
            r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
               <circle cx="12" cy="12" r="10"/>
               <path d="M12 8v4M12 16h.01"/>
               </svg>"#,
        ),
    }
}

// ─── IntoResponse ────────────────────────────────────────────────────────────

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            AppError::Io(err) => {
                if err.kind() == std::io::ErrorKind::NotFound {
                    (
                        StatusCode::NOT_FOUND,
                        "File or folder not found".to_string(),
                    )
                } else if err.kind() == std::io::ErrorKind::PermissionDenied {
                    (StatusCode::FORBIDDEN, "Permission denied".to_string())
                } else {
                    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
                }
            }
        };

        let (title, hint, icon_svg) = error_meta(status);

        // Use a hint if it's more informative than the raw message,
        // otherwise fall back to the escaped message.
        let description = if message == hint || message.is_empty() {
            hint.to_string()
        } else {
            util::html_escape(&message)
        };

        // Hex literals cannot appear inside r#"..."# raw strings (Rust 2021 prefix rules).
        // Build the head separately and concatenate.
        let dark_bg = "#09090B";
        let light_bg = "#FAFAFA";

        let head = format!(
            "<meta name=\"theme-color\" media=\"(prefers-color-scheme: dark)\"  content=\"{dark_bg}\">\n    \
             <meta name=\"theme-color\" media=\"(prefers-color-scheme: light)\" content=\"{light_bg}\">",
            dark_bg  = dark_bg,
            light_bg = light_bg,
        );

        let html = format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0, viewport-fit=cover">
    {head}
    <title>{title} — QRShare</title>
    <style>
        {common_css}

        body {{ justify-content: center; }}

        .err-wrap {{
            width: 100%;
            max-width: 420px;
            padding: 24px 20px;
            animation: fadeUp 180ms cubic-bezier(0.16, 1, 0.3, 1) both;
        }}

        .err-body {{
            display: flex;
            flex-direction: column;
            align-items: center;
            gap: 20px;
            padding: 40px 28px;
            text-align: center;
        }}

        .err-icon {{
            width: 52px;
            height: 52px;
            color: var(--muted);
            flex-shrink: 0;
        }}

        .err-title {{
            font-size: 22px;
            font-weight: 700;
            letter-spacing: -0.03em;
            color: var(--fg);
            margin: 0;
        }}

        .err-desc {{
            font-size: 14px;
            color: var(--muted);
            line-height: 1.6;
            max-width: 320px;
            margin: 0;
        }}

        .err-code {{
            font-size: 11px;
            font-weight: 500;
            color: var(--subtle);
            letter-spacing: 0.04em;
            padding: 4px 10px;
            background: var(--border);
            border-radius: 6px;
        }}
    </style>
</head>
<body>
    <div class="err-wrap">
        <div class="card">
            <div class="err-body">
                <div class="err-icon">{icon_svg}</div>
                <h1 class="err-title">{title}</h1>
                <p class="err-desc">{description}</p>
                <span class="err-code">{status_code}</span>
                <a href="/" class="btn btn-secondary">Go Back</a>
            </div>
        </div>
    </div>
</body>
</html>"#,
            head = head,
            common_css = crate::templates::COMMON_STYLE,
            title = title,
            icon_svg = icon_svg,
            description = description,
            status_code = status.as_u16(),
        );

        axum::response::Html(html).into_response()
    }
}
