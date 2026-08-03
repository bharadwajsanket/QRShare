use axum::{
    body::Body,
    extract::{ConnectInfo, Path, State},
    http::{header, Method, Request},
    response::{sse::Event, Html, IntoResponse, Redirect, Response, Sse},
    routing::get,
    Form, Router,
};
use axum_extra::extract::CookieJar;
use std::collections::HashSet;
use std::net::SocketAddr;
use std::path::{Path as StdPath, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tower::Service;
use tower_http::services::ServeFile;

use crate::error::AppError;
use crate::security::safe_resolve_path;
use crate::session::{constant_time_compare, AuthConfig};
use crate::templates;
use crate::util;

#[derive(Debug, Clone)]
pub enum ShareTarget {
    File(PathBuf),
    Folder(PathBuf),
    Url(String),
}

pub struct ServerState {
    pub target: ShareTarget,
    pub auth: AuthConfig,
    pub shutdown_tx: tokio::sync::watch::Sender<bool>,
    pub limit: Option<usize>,
    pub active_downloads: Mutex<HashSet<String>>,
    pub expired: std::sync::atomic::AtomicBool,
}

struct TrackingBody {
    inner: axum::body::Body,
    filename: String,
    state: Arc<ServerState>,
    session_id: String,
    is_download: bool,
    content_length: Option<u64>,
    bytes_written: u64,
}

impl http_body::Body for TrackingBody {
    type Data = bytes::Bytes;
    type Error = axum::Error;

    fn is_end_stream(&self) -> bool {
        self.inner.is_end_stream()
    }

    fn size_hint(&self) -> http_body::SizeHint {
        self.inner.size_hint()
    }

    fn poll_frame(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Result<http_body::Frame<Self::Data>, Self::Error>>> {
        let this = unsafe { self.get_unchecked_mut() };
        let inner_pinned = unsafe { std::pin::Pin::new_unchecked(&mut this.inner) };
        let res = inner_pinned.poll_frame(cx);

        match res {
            std::task::Poll::Ready(Some(Ok(frame))) => {
                if let Some(data) = frame.data_ref() {
                    this.bytes_written += data.len() as u64;
                }

                if this.is_download {
                    if let Some(expected_len) = this.content_length {
                        if this.bytes_written >= expected_len {
                            this.is_download = false;
                            println!("✓ Download completed: {}", this.filename);
                            let state = this.state.clone();
                            let session_id = this.session_id.clone();
                            tokio::spawn(async move {
                                let limit_reached = {
                                    let mut active = state.active_downloads.lock().unwrap();
                                    if active.insert(session_id) {
                                        if let Some(limit) = state.limit {
                                            let current_count = active.len();
                                            println!(
                                                "Download limit milestone: {}/{} users completed transfer.",
                                                current_count, limit
                                            );
                                            current_count >= limit
                                        } else {
                                            false
                                        }
                                    } else {
                                        false
                                    }
                                };

                                if limit_reached {
                                    println!("Limit reached. Setting expired state and initiating graceful shutdown...");
                                    state
                                        .expired
                                        .store(true, std::sync::atomic::Ordering::SeqCst);
                                    let tx = state.shutdown_tx.clone();
                                    tokio::time::sleep(Duration::from_millis(1000)).await;
                                    let _ = tx.send(true);
                                }
                            });
                        }
                    }
                }

                std::task::Poll::Ready(Some(Ok(frame)))
            }
            std::task::Poll::Ready(Some(Err(e))) => std::task::Poll::Ready(Some(Err(e))),
            std::task::Poll::Ready(None) => {
                if this.is_download {
                    this.is_download = false;
                    println!("✓ Download completed: {}", this.filename);

                    let state = this.state.clone();
                    let session_id = this.session_id.clone();
                    tokio::spawn(async move {
                        let limit_reached = {
                            let mut active = state.active_downloads.lock().unwrap();
                            if active.insert(session_id) {
                                if let Some(limit) = state.limit {
                                    let current_count = active.len();
                                    println!(
                                        "Download limit milestone: {}/{} users completed transfer.",
                                        current_count, limit
                                    );
                                    current_count >= limit
                                } else {
                                    false
                                }
                            } else {
                                false
                            }
                        };

                        if limit_reached {
                            println!("Limit reached. Setting expired state and initiating graceful shutdown...");
                            state
                                .expired
                                .store(true, std::sync::atomic::Ordering::SeqCst);
                            let tx = state.shutdown_tx.clone();
                            tokio::time::sleep(Duration::from_millis(1000)).await;
                            let _ = tx.send(true);
                        }
                    });
                }
                std::task::Poll::Ready(None)
            }
            std::task::Poll::Pending => std::task::Poll::Pending,
        }
    }
}

fn is_actual_download(req: &Request<Body>, query_download: bool) -> bool {
    let headers = req.headers();

    // 1. Browser prefetch / prerender requests must NOT count
    if let Some(purpose) = headers.get("Purpose").and_then(|v| v.to_str().ok()) {
        if purpose == "prefetch" {
            return false;
        }
    }
    if let Some(sec_purpose) = headers.get("Sec-Purpose").and_then(|v| v.to_str().ok()) {
        if sec_purpose.contains("prefetch") {
            return false;
        }
    }

    // 2. HEAD requests must NOT count
    if req.method() == http::Method::HEAD {
        return false;
    }

    // 3. Favicon requests must NOT count
    if req.uri().path().ends_with("favicon.ico") {
        return false;
    }

    // 4. If query param `download` is present, it's explicitly a download
    if query_download {
        return true;
    }

    // 5. Check Sec-Fetch-Dest header
    if let Some(dest) = headers.get("sec-fetch-dest").and_then(|v| v.to_str().ok()) {
        match dest {
            "image" | "video" | "audio" | "iframe" | "embed" | "object" | "style" | "script" => {
                // This is an inline preview asset load inside the HTML page
                return false;
            }
            "document" | "download" => {
                return true;
            }
            _ => {}
        }
    }

    // 6. Fallback based on User-Agent
    if let Some(ua) = headers
        .get(header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
    {
        let ua_lower = ua.to_lowercase();
        // If it's a command-line downloader
        if ua_lower.contains("curl")
            || ua_lower.contains("wget")
            || ua_lower.contains("httpie")
            || ua_lower.contains("aria2")
        {
            return true;
        }

        // If it looks like a standard browser but lacks `download` query param,
        // treat as preview/landing load (no download count).
        if ua_lower.contains("mozilla")
            || ua_lower.contains("safari")
            || ua_lower.contains("chrome")
            || ua_lower.contains("webkit")
        {
            return false;
        }
    }

    // Default to true for other/unknown clients
    true
}

/// Checks credentials and assigns a session cookie for tracking if not present.
fn handle_visitor_cookie(_state: &ServerState, jar: CookieJar) -> (CookieJar, String) {
    let current_token = jar.get("qrshare_session").map(|c| c.value().to_string());
    if let Some(token) = current_token {
        (jar, token)
    } else {
        // Generate an ephemeral guest token to track range request groupings and uniqueness
        let guest_token = uuid::Uuid::new_v4().to_string();
        let new_cookie: axum_extra::extract::cookie::Cookie<'static> =
            axum_extra::extract::cookie::Cookie::build(("qrshare_session", guest_token.clone()))
                .path("/")
                .http_only(true)
                .same_site(axum_extra::extract::cookie::SameSite::Lax)
                .into();
        (jar.add(new_cookie), guest_token)
    }
}

fn add_cache_control(mut res: Response) -> Response {
    res.headers_mut().insert(
        header::CACHE_CONTROL,
        header::HeaderValue::from_static("no-cache, no-store, must-revalidate"),
    );
    res.headers_mut()
        .insert(header::PRAGMA, header::HeaderValue::from_static("no-cache"));
    res.headers_mut()
        .insert(header::EXPIRES, header::HeaderValue::from_static("0"));
    res
}

#[derive(serde::Deserialize)]
struct RawQuery {
    download: Option<String>,
}

// ROUTE HANDLERS

async fn get_auth(State(state): State<Arc<ServerState>>, jar: CookieJar) -> Response {
    if state.auth.is_authenticated(&jar) {
        return Redirect::to("/").into_response();
    }
    let res = Html(templates::render_password_page(None)).into_response();
    add_cache_control(res)
}

#[derive(serde::Deserialize)]
struct AuthPayload {
    password: String,
}

async fn post_auth(
    State(state): State<Arc<ServerState>>,
    jar: CookieJar,
    Form(payload): Form<AuthPayload>,
) -> Response {
    let expected = match &state.auth.password {
        Some(p) => p,
        None => return Redirect::to("/").into_response(),
    };

    if constant_time_compare(&payload.password, expected) {
        let new_cookie = state.auth.create_session_cookie();
        (jar.add(new_cookie), Redirect::to("/")).into_response()
    } else {
        Html(templates::render_password_page(Some(
            "Incorrect Password. Please try again.",
        )))
        .into_response()
    }
}

async fn get_root(
    State(state): State<Arc<ServerState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    jar: CookieJar,
) -> Result<Response, AppError> {
    if state.expired.load(std::sync::atomic::Ordering::SeqCst) {
        return Err(AppError::Forbidden(
            "This sharing session has expired or reached its download limit.".to_string(),
        ));
    }

    if !state.auth.is_authenticated(&jar) {
        return Ok(Redirect::to("/auth").into_response());
    }

    match &state.target {
        ShareTarget::Url(_) => {
            println!("📱 {} connected", addr.ip());
        }
        ShareTarget::File(path) => {
            let filename = path.file_name().unwrap_or_default().to_string_lossy();
            println!("🔍 {} opened preview: {}", addr.ip(), filename);
        }
        ShareTarget::Folder(_) => {
            println!("📱 {} connected", addr.ip());
        }
    }

    let (jar, _) = handle_visitor_cookie(&state, jar);

    let res = match &state.target {
        ShareTarget::Url(url) => (jar, Html(templates::render_redirect_page(url))).into_response(),
        ShareTarget::File(path) => {
            let filename = path.file_name().unwrap_or_default().to_string_lossy();
            let size = tokio::fs::metadata(path).await?.len();
            let size_str = util::format_size(size);
            let mime = mime_guess::from_path(path)
                .first_or_octet_stream()
                .to_string();
            let preview = generate_preview_html(path, "/raw", &mime).await?;
            let is_code = is_code_file(path, &mime);

            (
                jar,
                Html(templates::render_file_page(
                    &filename,
                    &size_str,
                    &mime,
                    &preview,
                    is_code,
                    "/raw?download=1",
                )),
            )
                .into_response()
        }
        ShareTarget::Folder(dir) => {
            let breadcrumbs = build_breadcrumbs("");
            let items = compile_directory_items(dir, "").await?;
            let dir_name = dir.file_name().unwrap_or_default().to_string_lossy();

            (
                jar,
                Html(templates::render_folder_page(
                    &dir_name,
                    &breadcrumbs,
                    &items,
                    "/zip",
                )),
            )
                .into_response()
        }
    };
    Ok(add_cache_control(res))
}

async fn get_subpath(
    State(state): State<Arc<ServerState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    jar: CookieJar,
    Path(subpath): Path<String>,
) -> Result<Response, AppError> {
    if state.expired.load(std::sync::atomic::Ordering::SeqCst) {
        return Err(AppError::Forbidden(
            "This sharing session has expired or reached its download limit.".to_string(),
        ));
    }

    if !state.auth.is_authenticated(&jar) {
        return Ok(Redirect::to("/auth").into_response());
    }

    let (jar, _) = handle_visitor_cookie(&state, jar);

    let base_dir = match &state.target {
        ShareTarget::Folder(d) => d,
        _ => return Err(AppError::NotFound("Not directory sharing mode".to_string())),
    };

    let resolved = safe_resolve_path(base_dir, &subpath)?;
    if subpath.ends_with("favicon.ico") {
        return Err(AppError::NotFound("Favicon not found".to_string()));
    }

    if resolved.is_dir() {
        println!("📱 {} connected", addr.ip());
    } else {
        let filename = resolved.file_name().unwrap_or_default().to_string_lossy();
        println!("🔍 {} opened preview: {}", addr.ip(), filename);
    }

    let res = if resolved.is_dir() {
        let breadcrumbs = build_breadcrumbs(&subpath);
        let items = compile_directory_items(base_dir, &subpath).await?;
        let dir_name = resolved.file_name().unwrap_or_default().to_string_lossy();
        let zip_url = format!(
            "/zip/{}",
            percent_encoding::utf8_percent_encode(&subpath, percent_encoding::NON_ALPHANUMERIC)
        );

        (
            jar,
            Html(templates::render_folder_page(
                &dir_name,
                &breadcrumbs,
                &items,
                &zip_url,
            )),
        )
            .into_response()
    } else {
        // It's a file: Render preview wrap page
        let filename = resolved.file_name().unwrap_or_default().to_string_lossy();
        let size = tokio::fs::metadata(&resolved).await?.len();
        let size_str = util::format_size(size);
        let mime = mime_guess::from_path(&resolved)
            .first_or_octet_stream()
            .to_string();

        let raw_url = format!(
            "/raw/{}",
            percent_encoding::utf8_percent_encode(&subpath, percent_encoding::NON_ALPHANUMERIC)
        );
        let download_url = format!("{}?download=1", raw_url);
        let preview = generate_preview_html(&resolved, &raw_url, &mime).await?;
        let is_code = is_code_file(&resolved, &mime);

        (
            jar,
            Html(templates::render_file_page(
                &filename,
                &size_str,
                &mime,
                &preview,
                is_code,
                &download_url,
            )),
        )
            .into_response()
    };

    Ok(add_cache_control(res))
}

async fn get_raw(
    State(state): State<Arc<ServerState>>,
    jar: CookieJar,
    axum::extract::Query(query): axum::extract::Query<RawQuery>,
    req: Request<Body>,
) -> Result<Response, AppError> {
    if state.expired.load(std::sync::atomic::Ordering::SeqCst) {
        return Err(AppError::Forbidden(
            "This sharing session has expired or reached its download limit.".to_string(),
        ));
    }

    if !state.auth.is_authenticated(&jar) {
        return Err(AppError::Unauthorized("Access denied".to_string()));
    }

    let path = match &state.target {
        ShareTarget::File(p) => p.clone(),
        _ => {
            return Err(AppError::BadRequest(
                "Target is not a single file".to_string(),
            ))
        }
    };

    let filename = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();

    let is_download = is_actual_download(&req, query.download.is_some());

    if is_download {
        println!("⬇ Download started: {}", filename);
    }

    let mut service = ServeFile::new(path);
    let mut res = service
        .call(req)
        .await
        .map_err(|_| AppError::Internal("Failed to stream raw file".to_string()))?
        .into_response();

    if query.download.is_some() {
        let filename_escaped =
            percent_encoding::utf8_percent_encode(&filename, percent_encoding::NON_ALPHANUMERIC)
                .to_string();
        let header_val = format!(
            "attachment; filename=\"{}\"; filename*=UTF-8''{}",
            filename.replace('\"', "\\\""),
            filename_escaped
        );
        if let Ok(h_val) = header::HeaderValue::from_str(&header_val) {
            res.headers_mut().insert(header::CONTENT_DISPOSITION, h_val);
        }
    }

    let session_id = if let Some(cookie) = jar.get("qrshare_session") {
        cookie.value().to_string()
    } else {
        format!("anonymous-uuid-{}", uuid::Uuid::new_v4())
    };

    let content_length = res
        .headers()
        .get(header::CONTENT_LENGTH)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok());

    let (parts, body) = res.into_parts();
    let tracking_body = TrackingBody {
        inner: body,
        filename,
        state,
        session_id,
        is_download,
        content_length,
        bytes_written: 0,
    };

    Ok(Response::from_parts(parts, Body::new(tracking_body)))
}

async fn get_raw_subpath(
    State(state): State<Arc<ServerState>>,
    jar: CookieJar,
    Path(subpath): Path<String>,
    axum::extract::Query(query): axum::extract::Query<RawQuery>,
    req: Request<Body>,
) -> Result<Response, AppError> {
    if state.expired.load(std::sync::atomic::Ordering::SeqCst) {
        return Err(AppError::Forbidden(
            "This sharing session has expired or reached its download limit.".to_string(),
        ));
    }

    if !state.auth.is_authenticated(&jar) {
        return Err(AppError::Unauthorized("Access denied".to_string()));
    }

    let base_dir = match &state.target {
        ShareTarget::Folder(d) => d,
        _ => return Err(AppError::BadRequest("Target is not a folder".to_string())),
    };

    let resolved = safe_resolve_path(base_dir, &subpath)?;
    if resolved.is_dir() {
        return Err(AppError::BadRequest(
            "Cannot retrieve raw bytes of a directory".to_string(),
        ));
    }

    let filename = resolved
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();

    let is_download = is_actual_download(&req, query.download.is_some());

    if is_download {
        println!("⬇ Download started: {}", filename);
    }

    let mut service = ServeFile::new(resolved);
    let mut res = service
        .call(req)
        .await
        .map_err(|_| AppError::Internal("Failed to stream file".to_string()))?
        .into_response();

    if query.download.is_some() {
        let filename_escaped =
            percent_encoding::utf8_percent_encode(&filename, percent_encoding::NON_ALPHANUMERIC)
                .to_string();
        let header_val = format!(
            "attachment; filename=\"{}\"; filename*=UTF-8''{}",
            filename.replace('\"', "\\\""),
            filename_escaped
        );
        if let Ok(h_val) = header::HeaderValue::from_str(&header_val) {
            res.headers_mut().insert(header::CONTENT_DISPOSITION, h_val);
        }
    }

    let session_id = if let Some(cookie) = jar.get("qrshare_session") {
        cookie.value().to_string()
    } else {
        format!("anonymous-uuid-{}", uuid::Uuid::new_v4())
    };

    let content_length = res
        .headers()
        .get(header::CONTENT_LENGTH)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok());

    let (parts, body) = res.into_parts();
    let tracking_body = TrackingBody {
        inner: body,
        filename,
        state,
        session_id,
        is_download,
        content_length,
        bytes_written: 0,
    };

    Ok(Response::from_parts(parts, Body::new(tracking_body)))
}

async fn get_zip(
    State(state): State<Arc<ServerState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    jar: CookieJar,
    method: Method,
) -> Result<Response, AppError> {
    if state.expired.load(std::sync::atomic::Ordering::SeqCst) {
        return Err(AppError::Forbidden(
            "This sharing session has expired or reached its download limit.".to_string(),
        ));
    }

    if !state.auth.is_authenticated(&jar) {
        return Err(AppError::Unauthorized("Access denied".to_string()));
    }

    let root_dir = match &state.target {
        ShareTarget::Folder(d) => d.clone(),
        _ => {
            return Err(AppError::BadRequest(
                "Zip only available in directory mode".to_string(),
            ))
        }
    };

    let name = root_dir
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();
    serve_zip_stream(root_dir, &name, state, &jar, addr.ip(), method).await
}

async fn get_zip_subpath(
    State(state): State<Arc<ServerState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    jar: CookieJar,
    Path(subpath): Path<String>,
    method: Method,
) -> Result<Response, AppError> {
    if state.expired.load(std::sync::atomic::Ordering::SeqCst) {
        return Err(AppError::Forbidden(
            "This sharing session has expired or reached its download limit.".to_string(),
        ));
    }

    if !state.auth.is_authenticated(&jar) {
        return Err(AppError::Unauthorized("Access denied".to_string()));
    }

    let base_dir = match &state.target {
        ShareTarget::Folder(d) => d,
        _ => {
            return Err(AppError::BadRequest(
                "Zip only available in directory mode".to_string(),
            ))
        }
    };

    let resolved = safe_resolve_path(base_dir, &subpath)?;
    if !resolved.is_dir() {
        return Err(AppError::BadRequest("Cannot zip a single file".to_string()));
    }

    let name = resolved
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();
    serve_zip_stream(resolved, &name, state, &jar, addr.ip(), method).await
}

async fn serve_zip_stream(
    dir: PathBuf,
    name: &str,
    state: Arc<ServerState>,
    jar: &CookieJar,
    _client_ip: std::net::IpAddr,
    method: Method,
) -> Result<Response, AppError> {
    if state.expired.load(std::sync::atomic::Ordering::SeqCst) {
        return Err(AppError::Forbidden(
            "This sharing session has expired or reached its download limit.".to_string(),
        ));
    }

    if method == Method::HEAD {
        let res = Response::builder()
            .header(header::CONTENT_TYPE, "application/zip")
            .header(
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{}.zip\"", name),
            )
            .body(Body::empty())
            .map_err(|e| AppError::Internal(e.to_string()))?;
        return Ok(res);
    }

    let filename = format!("{}.zip", name);
    println!("⬇ Download started: {}", filename);

    let dir_clone = dir.clone();
    let temp_path = tokio::task::spawn_blocking(move || crate::zip::generate_zip_file(&dir_clone))
        .await
        .map_err(|e| AppError::Internal(format!("Zip thread join error: {}", e)))??;

    let file = tokio::fs::File::open(&temp_path)
        .await
        .map_err(AppError::Io)?;

    let stream = crate::zip::stream_zip_file(temp_path, file);
    let body = Body::from_stream(stream);

    let res = Response::builder()
        .header(header::CONTENT_TYPE, "application/zip")
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}.zip\"", name),
        )
        .body(body)
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let session_id = if let Some(cookie) = jar.get("qrshare_session") {
        cookie.value().to_string()
    } else {
        format!("anonymous-uuid-{}", uuid::Uuid::new_v4())
    };

    let (parts, body) = res.into_parts();
    let tracking_body = TrackingBody {
        inner: body,
        filename,
        state,
        session_id,
        is_download: true,
        content_length: None,
        bytes_written: 0,
    };

    Ok(Response::from_parts(parts, Body::new(tracking_body)))
}

async fn sse_handler(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> Sse<impl futures_util::Stream<Item = Result<Event, std::convert::Infallible>>> {
    struct DisconnectMonitor {
        _addr: SocketAddr,
    }
    impl Drop for DisconnectMonitor {
        fn drop(&mut self) {}
    }

    let monitor = Arc::new(DisconnectMonitor { _addr: addr });

    // Creates an unfolding stream yielding pings every 10s. Keep monitor alive in state closure.
    let stream = futures_util::stream::unfold(monitor, |m| async move {
        tokio::time::sleep(Duration::from_secs(10)).await;
        let event = Ok(Event::default().comment("ping"));
        Some((event, m))
    });

    Sse::new(stream)
}

// HELPERS

fn build_breadcrumbs(relative_path: &str) -> String {
    let mut html = String::new();
    html.push_str(r#"<a href="/">Root</a>"#);

    let clean = relative_path.trim_start_matches('/').trim_end_matches('/');
    if clean.is_empty() {
        return html;
    }

    let parts: Vec<&str> = clean.split('/').filter(|s| !s.is_empty()).collect();
    let mut accumulated = String::new();

    for (i, part) in parts.iter().enumerate() {
        accumulated.push('/');
        accumulated.push_str(part);
        html.push_str(r#"<span class="separator">/</span>"#);
        if i == parts.len() - 1 {
            html.push_str(&format!(
                r#"<span class="current">{}</span>"#,
                util::html_escape(part)
            ));
        } else {
            html.push_str(&format!(
                r#"<a href="{}">{}</a>"#,
                accumulated,
                util::html_escape(part)
            ));
        }
    }
    html
}

async fn compile_directory_items(base_dir: &StdPath, subpath: &str) -> Result<String, AppError> {
    let target_dir = safe_resolve_path(base_dir, subpath)?;
    let mut html = String::new();

    let mut entries_stream = tokio::fs::read_dir(target_dir).await?;
    let mut entries = Vec::new();
    while let Some(entry) = entries_stream.next_entry().await? {
        entries.push(entry);
    }

    struct EntryInfo {
        entry: tokio::fs::DirEntry,
        is_dir: bool,
        name: String,
    }
    let mut info_list = Vec::new();
    for entry in entries {
        let file_type = entry.file_type().await?;
        let is_dir = file_type.is_dir();
        let name = entry.file_name().to_string_lossy().into_owned();
        info_list.push(EntryInfo {
            entry,
            is_dir,
            name,
        });
    }

    // Sort entries: directories first, then alphabetically
    info_list.sort_by(|a, b| {
        if a.is_dir && !b.is_dir {
            std::cmp::Ordering::Less
        } else if !a.is_dir && b.is_dir {
            std::cmp::Ordering::Greater
        } else {
            a.name.cmp(&b.name)
        }
    });

    // Back path navigation if not at root
    if !subpath.is_empty() {
        let parent = StdPath::new(subpath)
            .parent()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_default();
        let parent_link = if parent.is_empty() {
            "/".to_string()
        } else {
            format!("/{}", parent)
        };

        html.push_str(&format!(
            r#"<a href="{}" class="item-row">
                <div class="item-left">
                    <svg class="item-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M19 12H5m7 7l-7-7 7-7"/></svg>
                    <span class="item-name">..</span>
                </div>
                <div class="item-right"></div>
               </a>"#,
            parent_link
        ));
    }

    for info in info_list {
        let name = info.name;

        let relative_link = if subpath.is_empty() {
            name.clone()
        } else {
            format!("{}/{}", subpath.trim_end_matches('/'), name)
        };

        let encoded_link = format!(
            "/{}",
            relative_link
                .split('/')
                .map(|segment| percent_encoding::utf8_percent_encode(
                    segment,
                    percent_encoding::NON_ALPHANUMERIC
                )
                .to_string())
                .collect::<Vec<_>>()
                .join("/")
        );

        if info.is_dir {
            html.push_str(&format!(
                r#"<a href="{}" class="item-row">
                    <div class="item-left">
                        <svg class="item-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg>
                        <span class="item-name">{}</span>
                    </div>
                    <div class="item-right">
                        <span class="item-size">-</span>
                    </div>
                   </a>"#,
                encoded_link,
                util::html_escape(&name)
            ));
        } else {
            let size = info.entry.metadata().await?.len();
            let size_str = util::format_size(size);
            html.push_str(&format!(
                r#"<a href="{}" class="item-row">
                    <div class="item-left">
                        <svg class="item-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/></svg>
                        <span class="item-name">{}</span>
                    </div>
                    <div class="item-right">
                        <span class="item-size">{}</span>
                    </div>
                   </a>"#,
                encoded_link,
                util::html_escape(&name),
                size_str
            ));
        }
    }

    Ok(html)
}

fn is_code_file(path: &StdPath, mime: &str) -> bool {
    if mime.starts_with("text/") && !mime.contains("markdown") && !mime.contains("html") {
        return true;
    }
    // Check known source code extensions
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        matches!(
            ext,
            "rs" | "js"
                | "ts"
                | "py"
                | "c"
                | "cpp"
                | "h"
                | "css"
                | "go"
                | "sh"
                | "json"
                | "yaml"
                | "toml"
        )
    } else {
        false
    }
}

async fn generate_preview_html(
    path: &StdPath,
    raw_url: &str,
    mime: &str,
) -> Result<String, AppError> {
    if mime.starts_with("image/") {
        Ok(format!(
            r#"<img src="{}" class="preview-media" alt="Image preview">"#,
            raw_url
        ))
    } else if mime.starts_with("video/") {
        Ok(format!(
            r#"<video src="{}" class="preview-media" controls autoplay muted playsinline></video>"#,
            raw_url
        ))
    } else if mime.starts_with("audio/") {
        Ok(format!(
            r#"<div class="audio-wrapper">
                <audio id="audio-element" src="{}"></audio>
                <div class="audio-player-controls">
                    <button class="play-pause-btn" id="audio-play-btn" type="button">
                        <svg id="play-svg" class="icon" viewBox="0 0 24 24" style="width:22px; height:22px; fill:currentColor;"><path d="M8 5v14l11-7z"/></svg>
                    </button>
                    <div class="audio-progress-container">
                        <div class="audio-time">
                            <span id="audio-current-time">0:00</span>
                            <span id="audio-duration">0:00</span>
                        </div>
                        <input type="range" class="slider-bar" id="audio-slider" min="0" max="100" value="0">
                    </div>
                </div>
            </div>
            <script>
                (function() {{
                    const audio = document.getElementById('audio-element');
                    const playBtn = document.getElementById('audio-play-btn');
                    const playSvg = document.getElementById('play-svg');
                    const slider = document.getElementById('audio-slider');
                    const currentTimeText = document.getElementById('audio-current-time');
                    const durationText = document.getElementById('audio-duration');

                    function formatTime(secs) {{
                        if (isNaN(secs)) return "0:00";
                        const m = Math.floor(secs / 60);
                        const s = Math.floor(secs % 60).toString().padStart(2, '0');
                        return m + ":" + s;
                    }}

                    playBtn.addEventListener('click', () => {{
                        if (audio.paused) {{
                            audio.play().catch(e => console.log("Play failed: ", e));
                            playSvg.innerHTML = '<path d="M6 19h4V5H6v14zm8-14v14h4V5h-4z"/>';
                        }} else {{
                            audio.pause();
                            playSvg.innerHTML = '<path d="M8 5v14l11-7z"/>';
                        }}
                    }});

                    audio.addEventListener('timeupdate', () => {{
                        if (!audio.duration || isNaN(audio.duration)) return;
                        const pct = (audio.currentTime / audio.duration) * 100;
                        slider.value = pct;
                        currentTimeText.textContent = formatTime(audio.currentTime);
                    }});

                    audio.addEventListener('loadedmetadata', () => {{
                        durationText.textContent = formatTime(audio.duration);
                    }});

                    // If already loaded
                    if (audio.readyState >= 1) {{
                        durationText.textContent = formatTime(audio.duration);
                    }}

                    slider.addEventListener('input', () => {{
                        if (!audio.duration || isNaN(audio.duration)) return;
                        audio.currentTime = (slider.value / 100) * audio.duration;
                    }});
                }})();
            </script>"#,
            raw_url
        ))
    } else if mime.contains("pdf") {
        Ok(format!(
            r#"<iframe src="{}" style="width: 100%; height: 60vh; border: none; border-radius: 12px;"></iframe>"#,
            raw_url
        ))
    } else if mime.contains("markdown") {
        let content = tokio::fs::read_to_string(path).await?;
        let parser = pulldown_cmark::Parser::new(&content);
        let mut html_output = String::new();
        pulldown_cmark::html::push_html(&mut html_output, parser);
        Ok(format!(
            r#"<div class="markdown-rendered">{}</div>"#,
            html_output
        ))
    } else if is_code_file(path, mime) {
        // Read file contents capped at 500KB safely without OOM
        let mut file = tokio::fs::File::open(path).await?;
        let mut buffer = vec![0u8; 500 * 1024];
        use tokio::io::AsyncReadExt;
        let n = file.read(&mut buffer).await?;
        buffer.truncate(n);

        let metadata = file.metadata().await?;
        let content = if metadata.len() > 500 * 1024 {
            let text = String::from_utf8_lossy(&buffer).into_owned();
            format!("{}\n\n... [File truncated - showing first 500KB] ...", text)
        } else {
            String::from_utf8_lossy(&buffer).into_owned()
        };

        let escaped = util::html_escape(&content);
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("plain");
        Ok(format!(
            r#"<div class="code-container"><pre><code class="language-{}">{}</code></pre></div>"#,
            ext, escaped
        ))
    } else {
        // Default download preview fallback
        Ok(r#"<div class="preview-fallback">
                <svg class="fallback-icon" viewBox="0 0 24 24">
                    <path d="M14 2H6c-1.1 0-1.99.9-1.99 2L4 20c0 1.1.89 2 1.99 2H18c1.1 0 2-.9 2-2V8l-6-6zm2 16H8v-2h8v2zm0-4H8v-2h8v2zm-3-5V3.5L18.5 9H13z"/>
                </svg>
                <span>No inline preview available</span>
            </div>"#.to_string())
    }
}

// SERVER STARTER

pub async fn start_server(state: Arc<ServerState>, addr: SocketAddr) -> Result<(), AppError> {
    let app = Router::new()
        .route("/", get(get_root))
        .route("/raw", get(get_raw))
        .route("/zip", get(get_zip))
        .route("/auth", get(get_auth).post(post_auth))
        .route("/events", get(sse_handler))
        .route("/zip/*path", get(get_zip_subpath))
        .route("/raw/*path", get(get_raw_subpath))
        .route("/*path", get(get_subpath))
        .with_state(state.clone());

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(AppError::Io)?;

    let mut shutdown_rx = state.shutdown_tx.subscribe();
    let graceful = async move {
        tokio::select! {
            _ = shutdown_rx.changed() => {
                println!("\n\x1b[1;32m✓ Session finished or expired. Shutting down...\x1b[0m");
            }
            _ = tokio::signal::ctrl_c() => {
                println!("\n\x1b[1;33m👋 Server aborted by user. Shutting down...\x1b[0m");
            }
        }
    };

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(graceful)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use std::collections::HashSet;
    use std::sync::Mutex;
    use tower::ServiceExt;

    fn test_router(state: Arc<ServerState>) -> Router {
        Router::new()
            .route("/", get(get_root))
            .route("/raw", get(get_raw))
            .route("/zip", get(get_zip))
            .route("/auth", get(get_auth).post(post_auth))
            .route("/events", get(sse_handler))
            .route("/zip/*path", get(get_zip_subpath))
            .route("/raw/*path", get(get_raw_subpath))
            .route("/*path", get(get_subpath))
            .with_state(state)
    }

    #[tokio::test]
    async fn test_auth_routing() {
        let (shutdown_tx, _) = tokio::sync::watch::channel(false);
        let state = Arc::new(ServerState {
            target: ShareTarget::Url("https://example.com".to_string()),
            auth: AuthConfig::new(Some("secret".to_string())),
            shutdown_tx,
            limit: None,
            active_downloads: Mutex::new(HashSet::new()),
            expired: std::sync::atomic::AtomicBool::new(false),
        });

        let app = test_router(state);

        // 1. Unauthenticated request -> expect redirect to /auth
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/")
                    .extension(ConnectInfo(SocketAddr::from(([127, 0, 0, 1], 12345))))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(response.headers().get(header::LOCATION).unwrap(), "/auth");

        // 2. GET /auth -> expect rendering page with caching headers
        let response = app
            .clone()
            .oneshot(Request::builder().uri("/auth").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get(header::CACHE_CONTROL).unwrap(),
            "no-cache, no-store, must-revalidate"
        );
    }

    #[tokio::test]
    async fn test_limit_by_addr_fallback() {
        let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);

        let temp_file =
            std::env::temp_dir().join(format!("qrshare-test-raw-{}", uuid::Uuid::new_v4()));
        std::fs::write(&temp_file, "hello").unwrap();

        let state = Arc::new(ServerState {
            target: ShareTarget::File(temp_file.clone()),
            auth: AuthConfig::new(None),
            shutdown_tx,
            limit: Some(1),
            active_downloads: Mutex::new(HashSet::new()),
            expired: std::sync::atomic::AtomicBool::new(false),
        });

        let app = test_router(state);

        let req = Request::builder()
            .uri("/raw")
            .extension(ConnectInfo(SocketAddr::from(([127, 0, 0, 1], 12345))))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        // Consume the body to trigger completion
        let body_bytes = axum::body::to_bytes(response.into_body(), 1024)
            .await
            .unwrap();
        assert_eq!(body_bytes, "hello");

        // Wait for the Toko delay task to trigger the channel
        tokio::time::sleep(Duration::from_millis(2200)).await;
        assert!(*shutdown_rx.borrow());

        let _ = std::fs::remove_file(temp_file);
    }

    #[tokio::test]
    async fn test_zip_head_request() {
        let (shutdown_tx, _) = tokio::sync::watch::channel(false);
        let temp_dir =
            std::env::temp_dir().join(format!("qrshare-test-zip-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&temp_dir).unwrap();

        let state = Arc::new(ServerState {
            target: ShareTarget::Folder(temp_dir.clone()),
            auth: AuthConfig::new(None),
            shutdown_tx,
            limit: None,
            active_downloads: Mutex::new(HashSet::new()),
            expired: std::sync::atomic::AtomicBool::new(false),
        });

        let app = test_router(state);

        let req = Request::builder()
            .method("HEAD")
            .uri("/zip")
            .extension(ConnectInfo(SocketAddr::from(([127, 0, 0, 1], 12345))))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get(header::CONTENT_TYPE).unwrap(),
            "application/zip"
        );
        assert!(response
            .headers()
            .get(header::CONTENT_DISPOSITION)
            .is_some());

        let body_bytes = axum::body::to_bytes(response.into_body(), 1024)
            .await
            .unwrap();
        assert!(body_bytes.is_empty());

        let _ = std::fs::remove_dir_all(temp_dir);
    }

    #[tokio::test]
    async fn test_download_limits_ignore_previews() {
        let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);

        let temp_file =
            std::env::temp_dir().join(format!("qrshare-test-preview-{}", uuid::Uuid::new_v4()));
        std::fs::write(&temp_file, "hello").unwrap();

        let state = Arc::new(ServerState {
            target: ShareTarget::File(temp_file.clone()),
            auth: AuthConfig::new(None),
            shutdown_tx,
            limit: Some(1),
            active_downloads: Mutex::new(HashSet::new()),
            expired: std::sync::atomic::AtomicBool::new(false),
        });

        let app = test_router(state);

        // 1. Request file as an inline image preview -> should NOT start or trigger limit
        let req = Request::builder()
            .uri("/raw")
            .header("sec-fetch-dest", "image")
            .header("user-agent", "Mozilla/5.0")
            .extension(ConnectInfo(SocketAddr::from(([127, 0, 0, 1], 12345))))
            .body(Body::empty())
            .unwrap();

        let response = app.clone().oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        // Ensure file content is read but limit is NOT reached
        let body_bytes = axum::body::to_bytes(response.into_body(), 1024)
            .await
            .unwrap();
        assert_eq!(body_bytes, "hello");
        tokio::time::sleep(Duration::from_millis(500)).await;
        assert!(!*shutdown_rx.borrow());

        // 2. Request file with download query param -> should trigger limit upon completion
        let req2 = Request::builder()
            .uri("/raw?download=1")
            .extension(ConnectInfo(SocketAddr::from(([127, 0, 0, 1], 12345))))
            .body(Body::empty())
            .unwrap();

        let response2 = app.oneshot(req2).await.unwrap();
        assert_eq!(response2.status(), StatusCode::OK);

        // Consume body to trigger complete event
        let body_bytes2 = axum::body::to_bytes(response2.into_body(), 1024)
            .await
            .unwrap();
        assert_eq!(body_bytes2, "hello");

        // Wait for EOF task to signal channel
        tokio::time::sleep(Duration::from_millis(1500)).await;
        assert!(*shutdown_rx.borrow());

        let _ = std::fs::remove_file(temp_file);
    }

    #[tokio::test]
    async fn test_regression_download_limits() {
        let temp_file =
            std::env::temp_dir().join(format!("qrshare-regression-{}", uuid::Uuid::new_v4()));
        std::fs::write(&temp_file, "regression-test-content").unwrap();

        // --- 1. Test --limit 1 shuts down after first completed download (using curl UA) ---
        {
            let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
            let state = Arc::new(ServerState {
                target: ShareTarget::File(temp_file.clone()),
                auth: AuthConfig::new(None),
                shutdown_tx,
                limit: Some(1),
                active_downloads: Mutex::new(HashSet::new()),
                expired: std::sync::atomic::AtomicBool::new(false),
            });
            let app = test_router(state);

            let req = Request::builder()
                .uri("/raw")
                .header("user-agent", "curl/7.68.0")
                .extension(ConnectInfo(SocketAddr::from(([127, 0, 0, 1], 12345))))
                .body(Body::empty())
                .unwrap();

            let response = app.oneshot(req).await.unwrap();
            assert_eq!(response.status(), StatusCode::OK);
            let body_bytes = axum::body::to_bytes(response.into_body(), 1024)
                .await
                .unwrap();
            assert_eq!(body_bytes, "regression-test-content");

            tokio::time::sleep(Duration::from_millis(1500)).await;
            assert!(
                *shutdown_rx.borrow(),
                "Server did not exit gracefully after limit 1"
            );
        }

        // --- 2. Test --limit 2 shuts down after second completed download (using wget UA) ---
        {
            let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
            let state = Arc::new(ServerState {
                target: ShareTarget::File(temp_file.clone()),
                auth: AuthConfig::new(None),
                shutdown_tx,
                limit: Some(2),
                active_downloads: Mutex::new(HashSet::new()),
                expired: std::sync::atomic::AtomicBool::new(false),
            });
            let app = test_router(state.clone());

            // First download (wget)
            let req1 = Request::builder()
                .uri("/raw")
                .header("user-agent", "Wget/1.20.3")
                .extension(ConnectInfo(SocketAddr::from(([127, 0, 0, 1], 12345))))
                .body(Body::empty())
                .unwrap();
            let response1 = app.clone().oneshot(req1).await.unwrap();
            assert_eq!(response1.status(), StatusCode::OK);
            let _ = axum::body::to_bytes(response1.into_body(), 1024)
                .await
                .unwrap();

            tokio::time::sleep(Duration::from_millis(500)).await;
            assert!(
                !*shutdown_rx.borrow(),
                "Server exited prematurely after 1 download under limit 2"
            );

            // Second download (wget)
            let req2 = Request::builder()
                .uri("/raw")
                .header("user-agent", "Wget/1.20.3")
                .extension(ConnectInfo(SocketAddr::from(([127, 0, 0, 2], 12345))))
                .body(Body::empty())
                .unwrap();
            let response2 = app.oneshot(req2).await.unwrap();
            assert_eq!(response2.status(), StatusCode::OK);
            let _ = axum::body::to_bytes(response2.into_body(), 1024)
                .await
                .unwrap();

            tokio::time::sleep(Duration::from_millis(1500)).await;
            assert!(
                *shutdown_rx.borrow(),
                "Server did not exit gracefully after limit 2"
            );
        }

        // --- 3. Test HEAD request and Range preview do NOT count ---
        {
            let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
            let state = Arc::new(ServerState {
                target: ShareTarget::File(temp_file.clone()),
                auth: AuthConfig::new(None),
                shutdown_tx,
                limit: Some(1),
                active_downloads: Mutex::new(HashSet::new()),
                expired: std::sync::atomic::AtomicBool::new(false),
            });
            let app = test_router(state);

            // HEAD Request
            let req_head = Request::builder()
                .method("HEAD")
                .uri("/raw")
                .extension(ConnectInfo(SocketAddr::from(([127, 0, 0, 1], 12345))))
                .body(Body::empty())
                .unwrap();
            let response_head = app.clone().oneshot(req_head).await.unwrap();
            assert_eq!(response_head.status(), StatusCode::OK);
            let _ = axum::body::to_bytes(response_head.into_body(), 1024)
                .await
                .unwrap();

            tokio::time::sleep(Duration::from_millis(500)).await;
            assert!(!*shutdown_rx.borrow(), "HEAD request counted as a download");

            // Range Preview (Mozilla UA, no download query param)
            let req_range = Request::builder()
                .uri("/raw")
                .header("user-agent", "Mozilla/5.0")
                .header("range", "bytes=0-5")
                .extension(ConnectInfo(SocketAddr::from(([127, 0, 0, 1], 12345))))
                .body(Body::empty())
                .unwrap();
            let response_range = app.clone().oneshot(req_range).await.unwrap();
            assert_eq!(response_range.status(), StatusCode::PARTIAL_CONTENT);
            let _ = axum::body::to_bytes(response_range.into_body(), 1024)
                .await
                .unwrap();

            tokio::time::sleep(Duration::from_millis(500)).await;
            assert!(
                !*shutdown_rx.borrow(),
                "Range preview counted as a download"
            );
        }

        // --- 4. Test Password-protected download counts ---
        {
            let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
            let auth = AuthConfig::new(Some("secret-pass".to_string()));
            let state = Arc::new(ServerState {
                target: ShareTarget::File(temp_file.clone()),
                auth: auth.clone(),
                shutdown_tx,
                limit: Some(1),
                active_downloads: Mutex::new(HashSet::new()),
                expired: std::sync::atomic::AtomicBool::new(false),
            });
            let app = test_router(state);

            // Authenticated download
            let cookie = auth.create_session_cookie();
            let cookie_str = format!("{}={}", cookie.name(), cookie.value());

            let req_auth = Request::builder()
                .uri("/raw")
                .header("cookie", cookie_str)
                .header("user-agent", "curl/7.68.0")
                .extension(ConnectInfo(SocketAddr::from(([127, 0, 0, 1], 12345))))
                .body(Body::empty())
                .unwrap();
            let response_auth = app.oneshot(req_auth).await.unwrap();
            assert_eq!(response_auth.status(), StatusCode::OK);
            let _ = axum::body::to_bytes(response_auth.into_body(), 1024)
                .await
                .unwrap();

            tokio::time::sleep(Duration::from_millis(1500)).await;
            assert!(
                *shutdown_rx.borrow(),
                "Authenticated download did not count"
            );
        }

        let _ = std::fs::remove_file(temp_file);
    }
}
