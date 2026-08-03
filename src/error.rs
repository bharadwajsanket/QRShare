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

        let html = format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Error - {status}</title>
    <style>
        :root {{
            --bg: #09090b;
            --fg: #f4f4f5;
            --accent: #ef4444;
            --card-bg: #18181b;
            --border: #27272a;
            --muted: #71717a;
        }}
        body {{
            background-color: var(--bg);
            color: var(--fg);
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
            margin: 0;
            display: flex;
            align-items: center;
            justify-content: center;
            min-height: 100vh;
            padding: 20px;
            box-sizing: border-box;
        }}
        .error-card {{
            background: var(--card-bg);
            border: 1px solid var(--border);
            border-radius: 20px;
            padding: 40px;
            max-width: 480px;
            width: 100%;
            text-align: center;
            box-shadow: 0 10px 30px rgba(0, 0, 0, 0.5);
            animation: fadeIn 0.4s ease-out;
        }}
        @keyframes fadeIn {{
            from {{ opacity: 0; transform: translateY(10px); }}
            to {{ opacity: 1; transform: translateY(0); }}
        }}
        .code {{
            font-size: 72px;
            font-weight: 800;
            color: var(--accent);
            margin: 0 0 16px 0;
            letter-spacing: -2px;
        }}
        .title {{
            font-size: 20px;
            font-weight: 600;
            margin: 0 0 12px 0;
        }}
        .message {{
            color: var(--muted);
            font-size: 14px;
            line-height: 1.6;
            margin-bottom: 24px;
            word-wrap: break-word;
        }}
        .home-btn {{
            display: inline-block;
            background: var(--fg);
            color: var(--bg);
            text-decoration: none;
            padding: 12px 24px;
            border-radius: 12px;
            font-size: 14px;
            font-weight: 500;
            transition: opacity 0.2s, transform 0.2s;
        }}
        .home-btn:hover {{
            opacity: 0.9;
            transform: scale(0.98);
        }}
    </style>
</head>
<body>
    <div class="error-card">
        <div class="code">{status_code}</div>
        <div class="title">{status_canonical}</div>
        <div class="message">{message}</div>
        <a href="/" class="home-btn">Go to Sharing Root</a>
    </div>
</body>
</html>"#,
            status = status,
            status_code = status.as_u16(),
            status_canonical = status.canonical_reason().unwrap_or("Error"),
            message = util::html_escape(&message)
        );

        axum::response::Html(html).into_response()
    }
}
