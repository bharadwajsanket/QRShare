//! Embedded HTML templates for QRShare v3.5.4.
//! Vanilla CSS + minimal JS · Light + Dark · Mobile-first · Zero external dependencies.

pub const COMMON_STYLE: &str = include_str!("templates/common.css");
pub const PRISM_CSS: &str = include_str!("templates/prism.css");
pub const PRISM_JS: &str = include_str!("templates/prism.js");

// ─── Common JavaScript Functions ─────────────────────────────────────────────
pub const COMMON_JS: &str = r#"
async function copyToClipboard(text, buttonEl) {
    var labelEl = buttonEl.querySelector('.copy-label') || buttonEl;
    var iconEl = buttonEl.querySelector('.copy-icon');
    var originalText = labelEl.textContent;
    
    var ICON_OK = '<polyline points="20 6 9 17 4 12"/>';
    var originalIcon = iconEl ? iconEl.innerHTML : '';
    
    function showSuccess() {
        labelEl.textContent = '✓ Copied!';
        if (iconEl) iconEl.innerHTML = ICON_OK;
        buttonEl.classList.add('copied');
        setTimeout(function() {
            labelEl.textContent = originalText;
            if (iconEl) iconEl.innerHTML = originalIcon;
            buttonEl.classList.remove('copied');
        }, 2000);
    }
    
    if (navigator.clipboard && navigator.clipboard.writeText) {
        try {
            await navigator.clipboard.writeText(text);
            showSuccess();
            return;
        } catch (_) {}
    }
    
    // Fallback
    try {
        var textArea = document.createElement("textarea");
        textArea.value = text;
        textArea.style.top = "0";
        textArea.style.left = "0";
        textArea.style.position = "fixed";
        document.body.appendChild(textArea);
        textArea.focus();
        textArea.select();
        var successful = document.execCommand('copy');
        document.body.removeChild(textArea);
        if (successful) {
            showSuccess();
        }
    } catch (_) {}
}

async function sharePage(title, url, buttonEl) {
    if (navigator.share) {
        try {
            await navigator.share({ title: title, url: url });
            return;
        } catch (_) {}
    }
    // Fallback: Copy URL and show feedback
    var labelEl = buttonEl.querySelector('.share-label') || buttonEl;
    var originalText = labelEl.textContent;
    await copyToClipboard(url, buttonEl);
    labelEl.textContent = '✓ Link Copied!';
    setTimeout(function() {
        labelEl.textContent = originalText;
    }, 2000);
}
"#;

// ─── File type icon ──────────────────────────────────────────────────────────

fn file_icon_svg(mime: &str) -> &'static str {
    if mime.starts_with("image/") {
        r#"<svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
              <rect x="3" y="3" width="18" height="18" rx="2.5"/>
              <circle cx="8.5" cy="8.5" r="1.5"/>
              <path d="m21 15-5-5L5 21"/>
           </svg>"#
    } else if mime.starts_with("video/") {
        r#"<svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
              <path d="m22 8-6 4 6 4V8z"/>
              <rect x="2" y="5" width="14" height="14" rx="2.5"/>
           </svg>"#
    } else if mime.starts_with("audio/") {
        r#"<svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
              <path d="M9 18V5l12-2v13"/>
              <circle cx="6" cy="18" r="3"/>
              <circle cx="18" cy="16" r="3"/>
           </svg>"#
    } else if mime == "application/pdf" {
        r#"<svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
              <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
              <path d="M14 2v6h6"/>
              <path d="M16 13H8M16 17H8M10 9H8"/>
           </svg>"#
    } else if mime == "application/zip"
        || mime == "application/x-zip-compressed"
        || mime == "application/gzip"
        || mime == "application/x-tar"
        || mime == "application/x-7z-compressed"
    {
        r#"<svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
              <path d="M21 8v13H3V8"/>
              <path d="M1 3h22v5H1z"/>
              <path d="M10 12h4"/>
           </svg>"#
    } else if mime.starts_with("text/") {
        r#"<svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
              <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
              <path d="M14 2v6h6"/>
              <path d="M16 13H8M16 17H8"/>
           </svg>"#
    } else {
        r#"<svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
              <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
              <path d="M14 2v6h6"/>
           </svg>"#
    }
}

// ─── Shared head fragment ────────────────────────────────────────────────────

fn head(title: &str, extra_style: &str, extra_head: &str) -> String {
    let dark_bg = "#09090B";
    let light_bg = "#FAFAFA";
    format!(
        "<head>\n    <meta charset=\"UTF-8\">\n    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0, viewport-fit=cover\">\n    <meta name=\"theme-color\" media=\"(prefers-color-scheme: dark)\"  content=\"{dark_bg}\">\n    <meta name=\"theme-color\" media=\"(prefers-color-scheme: light)\" content=\"{light_bg}\">\n    <title>{title}</title>\n    <style>{common}{extra}</style>\n    {extra_head}\n</head>",
        dark_bg = dark_bg,
        light_bg = light_bg,
        title = title,
        common = COMMON_STYLE,
        extra = extra_style,
        extra_head = extra_head,
    )
}

// ─── Consistent Brand Footer ──────────────────────────────────────────────────

fn brand_footer() -> &'static str {
    r#"<footer class="footer">
        <p>Made with ❤️ by <a href="https://github.com/bharadwajsanket" target="_blank" rel="noopener">Sanket Bharadwaj</a></p>
        <p><a href="https://github.com/bharadwajsanket/QRShare" target="_blank" rel="noopener">GitHub Repository</a></p>
    </footer>"#
}

// ─── Master Page Layout ───────────────────────────────────────────────────────

pub fn render_page_layout(
    browser_title: &str,
    icon_svg: &str,
    display_title: &str,
    preview_html: &str,
    metadata_html: &str,
    actions_html: &str,
    extra_js: &str,
    is_code: bool,
) -> String {
    let prism_style = if is_code {
        format!("<style>{}</style>", PRISM_CSS)
    } else {
        String::new()
    };

    let prism_script = if is_code {
        format!(
            "<script>{}</script><script>Prism.highlightAll();</script>",
            PRISM_JS
        )
    } else {
        String::new()
    };

    let preview_area_html = if preview_html.is_empty() {
        String::new()
    } else {
        format!(r#"<div class="preview-area">{}</div>"#, preview_html)
    };

    let page_head = head(
        browser_title,
        "",
        &prism_style,
    );

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
{head}
<body>
    <div class="page">
        <header class="page-header">
            <span class="wordmark">QRShare</span>
        </header>

        <main class="card">
            {preview_area_html}

            <div class="card-body">
                <div class="card-meta">
                    <div class="title-row">
                        <div class="title-icon">{icon_svg}</div>
                        <h1 class="card-title">{display_title}</h1>
                    </div>
                </div>

                {metadata_html}

                <div class="card-actions">
                    {actions_html}
                </div>
            </div>
        </main>
    </div>

    {footer}

    <script>
        {common_js}
        {extra_js}
    </script>
    {prism_script}
</body>
</html>"#,
        head = page_head,
        preview_area_html = preview_area_html,
        icon_svg = icon_svg,
        display_title = display_title,
        metadata_html = metadata_html,
        actions_html = actions_html,
        common_js = COMMON_JS,
        extra_js = extra_js,
        footer = brand_footer(),
        prism_script = prism_script,
    )
}

// ─── Metadata grid composition ───────────────────────────────────────────────

pub fn build_metadata(
    target_type: &str,
    session_time: &chrono::DateTime<chrono::Local>,
    expire_str: Option<&str>,
    limit: Option<usize>,
    downloads: usize,
) -> String {
    let type_val = target_type;

    let time_str = session_time.format("%I:%M %p").to_string();
    let time_str = if time_str.starts_with('0') { &time_str[1..] } else { &time_str };
    let shared_val = format!("{} • {}", session_time.format("%b %d, %Y"), time_str);

    let expiration_val = if let Some(exp) = expire_str {
        exp.to_string()
    } else if let Some(lim) = limit {
        if lim == 1 {
            "after 1 download".to_string()
        } else {
            format!("after {} downloads", lim)
        }
    } else {
        "Never".to_string()
    };

    let downloads_val = if let Some(lim) = limit {
        format!("{} / {}", downloads, lim)
    } else {
        format!("{} / Unlimited", downloads)
    };

    format!(
        r#"<div class="metadata-grid">
            <div class="metadata-item">
                <span class="metadata-label">Type</span>
                <span class="metadata-value">{}</span>
            </div>
            <div class="metadata-item">
                <span class="metadata-label">Shared</span>
                <span class="metadata-value">{}</span>
            </div>
            <div class="metadata-item">
                <span class="metadata-label">Expiration</span>
                <span class="metadata-value">{}</span>
            </div>
            <div class="metadata-item">
                <span class="metadata-label">Downloads</span>
                <span class="metadata-value">{}</span>
            </div>
        </div>"#,
        type_val, shared_val, expiration_val, downloads_val
    )
}

// ─── Password page ───────────────────────────────────────────────────────────

pub fn render_password_page(error_msg: Option<&str>) -> String {
    let error_html = match error_msg {
        Some(msg) => format!(
            r#"<p class="pw-error" role="alert">{}</p>"#,
            crate::util::html_escape(msg)
        ),
        None => String::new(),
    };

    let page_head = head(
        "Protected Share — QRShare",
        r#"
        body { justify-content: center; }
        .pw-wrap {
            width: 100%;
            max-width: 420px;
            padding: 24px 20px;
            display: flex;
            flex-direction: column;
            gap: 20px;
        }
        .pw-icon {
            display: flex;
            align-items: center;
            justify-content: center;
            width: 56px;
            height: 56px;
            border-radius: 16px;
            background: rgba(124, 58, 237, 0.12);
            border: 1px solid rgba(124, 58, 237, 0.2);
            color: var(--accent);
            flex-shrink: 0;
            margin: 0 auto;
        }
        .pw-body {
            display: flex;
            flex-direction: column;
            gap: 24px;
            padding: 32px 28px;
        }
        .pw-heading {
            text-align: center;
        }
        .pw-title {
            font-size: 22px;
            font-weight: 700;
            letter-spacing: -0.03em;
            color: var(--fg);
            margin: 0 0 6px 0;
        }
        .pw-sub {
            font-size: 14px;
            color: var(--muted);
            line-height: 1.5;
            margin: 0;
        }
        .pw-form {
            display: flex;
            flex-direction: column;
            gap: 12px;
        }
        .pw-error {
            font-size: 13px;
            font-weight: 500;
            color: var(--danger);
            text-align: center;
            padding: 10px 14px;
            background: rgba(248, 113, 113, 0.08);
            border: 1px solid rgba(248, 113, 113, 0.2);
            border-radius: 10px;
        }
        "#,
        "",
    );

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
{head}
<body>
    <div class="pw-wrap">
        <div class="card">
            <div class="pw-body">
                <div class="pw-heading">
                    <div class="pw-icon">
                        <svg width="26" height="26" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                            <rect x="3" y="11" width="18" height="11" rx="2"/>
                            <path d="M7 11V7a5 5 0 0 1 10 0v4"/>
                        </svg>
                    </div>
                    <h1 class="pw-title" style="margin-top:16px">Protected Share</h1>
                    <p class="pw-sub">Enter the password to access this file.</p>
                </div>

                <form action="/auth" method="POST" class="pw-form">
                    {error_html}
                    <input
                        type="password"
                        name="password"
                        class="input"
                        placeholder="Password"
                        aria-label="Password"
                        required
                        autofocus>
                    <button type="submit" class="btn btn-primary btn-full">
                        Unlock
                    </button>
                </form>
            </div>
        </div>
    </div>

    {footer}
</body>
</html>"#,
        head = page_head,
        error_html = error_html,
        footer = brand_footer(),
    )
}

// ─── File page ───────────────────────────────────────────────────────────────

pub fn render_file_page(
    name: &str,
    size_str: &str,
    mime: &str,
    preview_html: &str,
    is_code: bool,
    download_url: &str,
    session_time: &chrono::DateTime<chrono::Local>,
    expire_str: Option<&str>,
    limit: Option<usize>,
    downloads: usize,
) -> String {
    let browser_title = format!("{} • QRShare", name);
    let icon_svg = file_icon_svg(mime).to_string();
    let display_title = format!("{} ({})", name, size_str);

    let metadata_html = build_metadata("File", session_time, expire_str, limit, downloads);

    let actions_html = format!(
        r#"<a href="{download_url}"
              class="btn btn-primary"
              download="{name}"
              aria-label="Download {name}">
               <svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                   <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
                   <polyline points="7 10 12 15 17 10"/>
                   <line x1="12" y1="15" x2="12" y2="3"/>
               </svg>
               Download
           </a>
           <button class="btn btn-secondary"
                   id="share-btn"
                   type="button"
                   aria-label="Share">
               <svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                   <circle cx="18" cy="5" r="3"/><circle cx="6" cy="12" r="3"/><circle cx="18" cy="19" r="3"/>
                   <line x1="8.59" y1="13.51" x2="15.42" y2="17.49"/>
                   <line x1="15.41" y1="6.51" x2="8.59" y2="10.49"/>
               </svg>
               <span class="share-label">Share</span>
           </button>"#,
        download_url = download_url,
        name = crate::util::html_escape(name)
    );

    let extra_js = format!(
        r#"var shareBtn = document.getElementById('share-btn');
           shareBtn.addEventListener('click', function() {{
               sharePage('{}', window.location.href, shareBtn);
           }});"#,
        name.replace('\'', "\\'")
    );

    render_page_layout(
        &browser_title,
        &icon_svg,
        &display_title,
        preview_html,
        &metadata_html,
        &actions_html,
        &extra_js,
        is_code,
    )
}

// ─── Folder page ─────────────────────────────────────────────────────────────

pub fn render_folder_page(
    dir_name: &str,
    breadcrumbs_html: &str,
    items_html: &str,
    zip_url: &str,
    session_time: &chrono::DateTime<chrono::Local>,
    expire_str: Option<&str>,
    limit: Option<usize>,
    downloads: usize,
) -> String {
    let browser_title = format!("{} • QRShare", dir_name);
    let icon_svg = r#"<svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
        <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
    </svg>"#.to_string();
    let display_title = dir_name.to_string();

    let preview_html = format!(
        r#"<div class="folder-breadcrumbs" style="padding: 16px 24px 8px; border-bottom: 1px solid var(--border);">
            <nav class="breadcrumbs" aria-label="Directory path">
                {}
            </nav>
        </div>
        <div class="item-list" role="list">
            {}
        </div>"#,
        breadcrumbs_html, items_html
    );

    let metadata_html = build_metadata("Folder", session_time, expire_str, limit, downloads);

    let actions_html = format!(
        r#"<a href="{zip_url}"
              class="btn btn-primary"
              aria-label="Download ZIP">
               <svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                   <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
                   <polyline points="7 10 12 15 17 10"/>
                   <line x1="12" y1="15" x2="12" y2="3"/>
               </svg>
               Download ZIP
           </a>
           <button class="btn btn-secondary"
                   id="share-btn"
                   type="button"
                   aria-label="Share">
               <svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                   <circle cx="18" cy="5" r="3"/><circle cx="6" cy="12" r="3"/><circle cx="18" cy="19" r="3"/>
                   <line x1="8.59" y1="13.51" x2="15.42" y2="17.49"/>
                   <line x1="15.41" y1="6.51" x2="8.59" y2="10.49"/>
               </svg>
               <span class="share-label">Share</span>
           </button>"#,
        zip_url = zip_url
    );

    let extra_js = format!(
        r#"var shareBtn = document.getElementById('share-btn');
           shareBtn.addEventListener('click', function() {{
               sharePage('{}', window.location.href, shareBtn);
           }});"#,
        dir_name.replace('\'', "\\'")
    );

    render_page_layout(
        &browser_title,
        &icon_svg,
        &display_title,
        &preview_html,
        &metadata_html,
        &actions_html,
        &extra_js,
        false,
    )
}

// ─── URL Page ────────────────────────────────────────────────────────────────

pub fn render_url_page(
    url: &str,
    session_time: &chrono::DateTime<chrono::Local>,
    expire_str: Option<&str>,
    limit: Option<usize>,
    downloads: usize,
) -> String {
    let url_esc = crate::util::html_escape(url);
    
    // Extract domain for browser title
    let domain = url.split("://").nth(1).unwrap_or(url).split('/').next().unwrap_or("URL");
    let browser_title = format!("{} • QRShare", domain);
    
    let icon_svg = r#"<svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
        <path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"/>
        <path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"/>
    </svg>"#.to_string();

    let display_title = "Shared Link".to_string();

    let preview_html = format!(
        r#"<div style="padding: 24px; width: 100%; display: flex; justify-content: center;">
            <div class="url-link-card" style="width: 100%; background: rgba(0, 0, 0, 0.2); border: 1px solid var(--border); border-radius: 12px; padding: 14px 18px; font-family: 'SF Mono', 'Fira Code', monospace; font-size: 13px; color: var(--muted); word-break: break-all; text-align: left; line-height: 1.4;">
                {}
            </div>
        </div>"#,
        url_esc
    );

    let metadata_html = build_metadata("URL", session_time, expire_str, limit, downloads);

    let actions_html = format!(
        r#"<a href="{url_esc}"
              target="_blank"
              rel="noopener"
              class="btn btn-primary"
              aria-label="Open Website">
               <svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                   <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/>
                   <polyline points="15 3 21 3 21 9"/>
                   <line x1="10" y1="14" x2="21" y2="3"/>
               </svg>
               Open Website
           </a>
           <button class="btn btn-secondary"
                   id="copy-btn"
                   type="button"
                   aria-label="Copy Link">
               <svg class="icon copy-icon" id="copy-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                   <rect x="9" y="9" width="13" height="13" rx="2" ry="2"/>
                   <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/>
               </svg>
               <span class="copy-label">Copy Link</span>
           </button>
           <button class="btn btn-secondary"
                   id="share-btn"
                   type="button"
                   aria-label="Share">
               <svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                   <circle cx="18" cy="5" r="3"/><circle cx="6" cy="12" r="3"/><circle cx="18" cy="19" r="3"/>
                   <line x1="8.59" y1="13.51" x2="15.42" y2="17.49"/>
                   <line x1="15.41" y1="6.51" x2="8.59" y2="10.49"/>
               </svg>
               <span class="share-label">Share</span>
           </button>"#,
        url_esc = url_esc
    );

    let extra_js = format!(
        r#"var copyBtn = document.getElementById('copy-btn');
           copyBtn.addEventListener('click', function() {{
               copyToClipboard('{}', copyBtn);
           }});
           
           var shareBtn = document.getElementById('share-btn');
           shareBtn.addEventListener('click', function() {{
               sharePage('Shared Link', window.location.href, shareBtn);
           }});"#,
        url_esc.replace('\'', "\\'")
    );

    render_page_layout(
        &browser_title,
        &icon_svg,
        &display_title,
        &preview_html,
        &metadata_html,
        &actions_html,
        &extra_js,
        false,
    )
}

// ─── Text Page ───────────────────────────────────────────────────────────────

pub fn render_text_page(
    text: &str,
    preview_html: &str,
    is_code: bool,
    session_time: &chrono::DateTime<chrono::Local>,
    expire_str: Option<&str>,
    limit: Option<usize>,
    downloads: usize,
) -> String {
    let browser_title = "Shared Text • QRShare".to_string();
    
    let icon_svg = r#"<svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
        <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
        <polyline points="14 2 14 8 20 8"/>
        <line x1="16" y1="13" x2="8" y2="13"/>
        <line x1="16" y1="17" x2="8" y2="17"/>
    </svg>"#.to_string();

    let display_title = "Shared Text".to_string();
    let metadata_html = build_metadata("Text", session_time, expire_str, limit, downloads);

    let actions_html = r#"<button class="btn btn-primary"
                                  id="copy-btn"
                                  type="button"
                                  aria-label="Copy Text">
                               <svg class="icon copy-icon" id="copy-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                   <rect x="9" y="9" width="13" height="13" rx="2" ry="2"/>
                                   <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/>
                               </svg>
                               <span class="copy-label">Copy</span>
                           </button>
                           <a href="/raw?download=1"
                              class="btn btn-secondary"
                              download="qrshare.txt"
                              aria-label="Download as text file">
                               <svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                   <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
                                   <polyline points="7 10 12 15 17 10"/>
                                   <line x1="12" y1="15" x2="12" y2="3"/>
                               </svg>
                               Download .txt
                           </a>
                           <button class="btn btn-secondary"
                                   id="share-btn"
                                   type="button"
                                   aria-label="Share">
                               <svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                   <circle cx="18" cy="5" r="3"/><circle cx="6" cy="12" r="3"/><circle cx="18" cy="19" r="3"/>
                                   <line x1="8.59" y1="13.51" x2="15.42" y2="17.49"/>
                                   <line x1="15.41" y1="6.51" x2="8.59" y2="10.49"/>
                               </svg>
                               <span class="share-label">Share</span>
                           </button>"#.to_string();

    let text_escaped_js = text.replace('\\', "\\\\").replace('\'', "\\'").replace('\n', "\\n").replace('\r', "\\r");

    let extra_js = format!(
        r#"var copyBtn = document.getElementById('copy-btn');
           copyBtn.addEventListener('click', function() {{
               copyToClipboard('{}', copyBtn);
           }});
           
           var shareBtn = document.getElementById('share-btn');
           shareBtn.addEventListener('click', function() {{
               sharePage('Shared Text', window.location.href, shareBtn);
           }});"#,
        text_escaped_js
    );

    render_page_layout(
        &browser_title,
        &icon_svg,
        &display_title,
        preview_html,
        &metadata_html,
        &actions_html,
        &extra_js,
        is_code,
    )
}

// ─── Clipboard Image Page ────────────────────────────────────────────────────

pub fn render_clipboard_image_page(
    session_time: &chrono::DateTime<chrono::Local>,
    expire_str: Option<&str>,
    limit: Option<usize>,
    downloads: usize,
) -> String {
    let browser_title = "Clipboard • QRShare".to_string();
    
    let icon_svg = r#"<svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
        <rect x="3" y="3" width="18" height="18" rx="2" ry="2"/>
        <circle cx="8.5" cy="8.5" r="1.5"/>
        <polyline points="21 15 16 10 5 21"/>
    </svg>"#.to_string();

    let display_title = "Shared Image".to_string();

    let preview_html = r#"<img src="/raw" class="preview-media" alt="Clipboard Image preview">"#.to_string();

    let metadata_html = build_metadata("Clipboard Image", session_time, expire_str, limit, downloads);

    let actions_html = r#"<a href="/raw?download=1"
                              class="btn btn-primary"
                              download="qrshare-clip.png"
                              aria-label="Download Image">
                               <svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                   <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
                                   <polyline points="7 10 12 15 17 10"/>
                                   <line x1="12" y1="15" x2="12" y2="3"/>
                               </svg>
                               Download
                           </a>
                           <button class="btn btn-secondary"
                                   id="share-btn"
                                   type="button"
                                   aria-label="Share">
                               <svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                   <circle cx="18" cy="5" r="3"/><circle cx="6" cy="12" r="3"/><circle cx="18" cy="19" r="3"/>
                                   <line x1="8.59" y1="13.51" x2="15.42" y2="17.49"/>
                                   <line x1="15.41" y1="6.51" x2="8.59" y2="10.49"/>
                               </svg>
                               <span class="share-label">Share</span>
                           </button>"#.to_string();

    let extra_js = r#"var shareBtn = document.getElementById('share-btn');
                      shareBtn.addEventListener('click', function() {
                          sharePage('Shared Image', window.location.href, shareBtn);
                      });"#.to_string();

    render_page_layout(
        &browser_title,
        &icon_svg,
        &display_title,
        &preview_html,
        &metadata_html,
        &actions_html,
        &extra_js,
        false,
    )
}
