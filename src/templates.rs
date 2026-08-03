//! Embedded HTML templates for QRShare v2.5.4.
//! Vanilla CSS + minimal JS · Light + Dark · Mobile-first · Zero external dependencies.

pub const COMMON_STYLE: &str = include_str!("templates/common.css");
pub const PRISM_CSS: &str = include_str!("templates/prism.css");
pub const PRISM_JS: &str = include_str!("templates/prism.js");

// ─── File type icon ──────────────────────────────────────────────────────────

fn file_icon_svg(mime: &str) -> &'static str {
    if mime.starts_with("image/") {
        // Picture frame with mountain
        r#"<svg width="52" height="52" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
              <rect x="3" y="3" width="18" height="18" rx="2.5"/>
              <circle cx="8.5" cy="8.5" r="1.5"/>
              <path d="m21 15-5-5L5 21"/>
           </svg>"#
    } else if mime.starts_with("video/") {
        // Film / video camera
        r#"<svg width="52" height="52" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
              <path d="m22 8-6 4 6 4V8z"/>
              <rect x="2" y="5" width="14" height="14" rx="2.5"/>
           </svg>"#
    } else if mime.starts_with("audio/") {
        // Music note
        r#"<svg width="52" height="52" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
              <path d="M9 18V5l12-2v13"/>
              <circle cx="6" cy="18" r="3"/>
              <circle cx="18" cy="16" r="3"/>
           </svg>"#
    } else if mime == "application/pdf" {
        // Document with lines
        r#"<svg width="52" height="52" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
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
        // Archive / box
        r#"<svg width="52" height="52" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
              <path d="M21 8v13H3V8"/>
              <path d="M1 3h22v5H1z"/>
              <path d="M10 12h4"/>
           </svg>"#
    } else if mime.starts_with("text/") {
        // Text document
        r#"<svg width="52" height="52" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
              <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
              <path d="M14 2v6h6"/>
              <path d="M16 13H8M16 17H8"/>
           </svg>"#
    } else {
        // Generic file
        r#"<svg width="52" height="52" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
              <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
              <path d="M14 2v6h6"/>
           </svg>"#
    }
}

// ─── Shared head fragment ────────────────────────────────────────────────────

fn head(title: &str, extra_style: &str, extra_head: &str) -> String {
    // Hex literals cannot appear raw inside format! strings (Rust 2021 prefix rules)
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

    <footer class="footer">
        Shared with <a href="https://github.com/bharadwajsanket/QRShare" target="_blank" rel="noopener">QRShare</a>
    </footer>
</body>
</html>"#,
        head = page_head,
        error_html = error_html,
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

    let name_esc = crate::util::html_escape(name);

    // Determine whether to show a standalone icon header (no rich preview)
    let has_rich_preview = mime.starts_with("image/")
        || mime.starts_with("video/")
        || mime.starts_with("audio/")
        || mime.starts_with("text/")
        || mime == "application/pdf";

    // File type label
    let type_label = if let Some(sub) = mime.split('/').nth(1) {
        sub.to_uppercase()
    } else {
        "FILE".to_string()
    };

    // Icon shown above filename when there is no rich inline preview
    let icon_header = if !has_rich_preview {
        format!(
            r#"<div class="file-icon-wrap" aria-hidden="true">{icon}</div>"#,
            icon = file_icon_svg(mime)
        )
    } else {
        String::new()
    };

    let page_head = head(
        &format!("{} — QRShare", name_esc),
        r#"
        /* ── File page layout ── */
        .file-header {
            display: flex;
            align-items: center;
            justify-content: space-between;
            padding: 0 2px;
        }
        .file-mime-badge {
            font-size: 12px;
            font-weight: 500;
            color: var(--muted);
            letter-spacing: 0.04em;
        }
        /* Preview area — zero horizontal padding so images fill edge-to-edge */
        .preview-area {
            background: rgba(0, 0, 0, 0.06);
            border-bottom: 1px solid var(--border);
            overflow: hidden;
            min-height: 160px;
            display: flex;
            align-items: center;
            justify-content: center;
        }
        @media (prefers-color-scheme: light) {
            .preview-area { background: rgba(0, 0, 0, 0.03); }
        }
        /* File icon (for non-rich previews) */
        .file-icon-wrap {
            display: flex;
            align-items: center;
            justify-content: center;
            padding: 36px 0;
            color: var(--subtle);
        }
        /* Content section */
        .file-body {
            padding: 24px;
            display: flex;
            flex-direction: column;
            gap: 20px;
        }
        .file-meta {
            display: flex;
            flex-direction: column;
            gap: 10px;
        }
        .file-name {
            font-size: 20px;
            font-weight: 700;
            letter-spacing: -0.03em;
            line-height: 1.25;
            word-break: break-all;
            color: var(--fg);
        }
        .file-actions {
            display: flex;
            flex-direction: column;
            gap: 10px;
        }
        @media (min-width: 380px) {
            .file-actions { flex-direction: row; }
            .file-actions .btn { flex: 1; }
        }
        "#,
        &prism_style,
    );

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
{head}
<body>
    <div class="page">
        <header class="file-header">
            <span class="wordmark">QRShare</span>
            <span class="file-mime-badge">{type_label}</span>
        </header>

        <main class="card">
            <div class="preview-area">
                {icon_header}
                {preview_html}
            </div>

            <div class="file-body">
                <div class="file-meta">
                    <h1 class="file-name">{name_esc}</h1>
                    <div class="chips">
                        <span class="chip">{size_str}</span>
                        <span class="chip">{type_label}</span>
                    </div>
                </div>

                <div class="file-actions">
                    <a href="{download_url}"
                       class="btn btn-primary"
                       download="{name_esc}"
                       aria-label="Download {name_esc}">
                        <svg class="icon" viewBox="0 0 24 24" aria-hidden="true">
                            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
                            <polyline points="7 10 12 15 17 10"/>
                            <line x1="12" y1="15" x2="12" y2="3"/>
                        </svg>
                        Download
                    </a>
                    <button class="btn btn-secondary"
                            id="copy-btn"
                            type="button"
                            aria-label="Copy sharing link">
                        <svg class="icon" id="copy-icon" viewBox="0 0 24 24" aria-hidden="true">
                            <rect x="9" y="9" width="13" height="13" rx="2"/>
                            <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/>
                        </svg>
                        <span id="copy-label">Copy Link</span>
                    </button>
                </div>
            </div>
        </main>
    </div>

    <footer class="footer">
        Shared with <a href="https://github.com/bharadwajsanket/QRShare" target="_blank" rel="noopener">QRShare</a>
    </footer>

    <script>
        (function() {{
            var btn  = document.getElementById('copy-btn');
            var lbl  = document.getElementById('copy-label');
            var ico  = document.getElementById('copy-icon');

            var ICON_COPY = '<rect x="9" y="9" width="13" height="13" rx="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/>';
            var ICON_OK   = '<polyline points="20 6 9 17 4 12"/>';

            function reset() {{
                lbl.textContent = 'Copy Link';
                ico.innerHTML   = ICON_COPY;
            }}

            btn.addEventListener('click', async function() {{
                var url = window.location.href;

                if (navigator.share) {{
                    try {{
                        await navigator.share({{ title: '{name_esc}', url: url }});
                        return;
                    }} catch (_) {{}}
                }}

                try {{
                    await navigator.clipboard.writeText(url);
                    lbl.textContent = 'Copied!';
                    ico.innerHTML   = ICON_OK;
                    setTimeout(reset, 2000);
                }} catch (_) {{
                    reset();
                }}
            }});
        }})();
    </script>
    {prism_script}
</body>
</html>"#,
        head = page_head,
        name_esc = name_esc,
        size_str = size_str,
        type_label = type_label,
        preview_html = preview_html,
        icon_header = icon_header,
        download_url = download_url,
        prism_script = prism_script,
    )
}

// ─── Folder page ─────────────────────────────────────────────────────────────

pub fn render_folder_page(
    dir_name: &str,
    breadcrumbs_html: &str,
    items_html: &str,
    zip_url: &str,
) -> String {
    let dir_name_esc = crate::util::html_escape(dir_name);

    let page_head = head(
        &format!("{} — QRShare", dir_name_esc),
        r#"
        .folder-header {
            display: flex;
            align-items: center;
            justify-content: space-between;
            gap: 12px;
            flex-wrap: wrap;
            padding: 0 2px;
        }
        .folder-breadcrumbs {
            display: flex;
            flex-direction: column;
            gap: 4px;
            min-width: 0;
        }
        .folder-title {
            font-size: 15px;
            font-weight: 700;
            letter-spacing: -0.02em;
            color: var(--fg);
            white-space: nowrap;
            overflow: hidden;
            text-overflow: ellipsis;
        }
        .folder-actions {
            display: flex;
            align-items: center;
            justify-content: space-between;
            gap: 12px;
            padding: 20px 24px 16px;
            border-bottom: 1px solid var(--border);
        }
        .folder-label {
            font-size: 13px;
            font-weight: 600;
            color: var(--muted);
            letter-spacing: 0.02em;
            text-transform: uppercase;
        }
        .btn-zip {
            height: 38px;
            padding: 0 16px;
            font-size: 13px;
            border-radius: 10px;
            flex-shrink: 0;
        }
        "#,
        "",
    );

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
{head}
<body>
    <div class="page">
        <header class="folder-header">
            <div class="folder-breadcrumbs">
                <span class="wordmark">QRShare</span>
                <nav class="breadcrumbs" aria-label="Directory path">
                    {breadcrumbs_html}
                </nav>
            </div>
        </header>

        <main class="card">
            <div class="folder-actions">
                <span class="folder-label">Files</span>
                <a href="{zip_url}"
                   class="btn btn-primary btn-zip"
                   aria-label="Download all files as ZIP">
                    <svg class="icon" viewBox="0 0 24 24" aria-hidden="true">
                        <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
                        <polyline points="7 10 12 15 17 10"/>
                        <line x1="12" y1="15" x2="12" y2="3"/>
                    </svg>
                    Download ZIP
                </a>
            </div>

            <div class="item-list" role="list">
                {items_html}
            </div>
        </main>
    </div>

    <footer class="footer">
        Shared with <a href="https://github.com/bharadwajsanket/QRShare" target="_blank" rel="noopener">QRShare</a>
    </footer>
</body>
</html>"#,
        head = page_head,
        breadcrumbs_html = breadcrumbs_html,
        items_html = items_html,
        zip_url = zip_url,
    )
}

// ─── Redirect page ───────────────────────────────────────────────────────────

pub fn render_redirect_page(target_url: &str) -> String {
    let target_esc = crate::util::html_escape(target_url);

    let page_head = head(
        "Redirecting — QRShare",
        r#"
        body { justify-content: center; }
        .redirect-wrap {
            width: 100%;
            max-width: 420px;
            padding: 24px 20px;
            animation: fadeUp 180ms cubic-bezier(0.16, 1, 0.3, 1) both;
        }
        .redirect-body {
            display: flex;
            flex-direction: column;
            align-items: center;
            gap: 20px;
            padding: 40px 28px;
            text-align: center;
        }
        .spinner {
            width: 36px;
            height: 36px;
            border: 2.5px solid var(--border-med);
            border-top-color: var(--accent);
            border-radius: 50%;
            animation: spin 0.75s linear infinite;
            flex-shrink: 0;
        }
        @keyframes spin {
            to { transform: rotate(360deg); }
        }
        .redirect-title {
            font-size: 20px;
            font-weight: 700;
            letter-spacing: -0.03em;
            color: var(--fg);
            margin: 0;
        }
        .redirect-url {
            font-size: 12px;
            color: var(--muted);
            word-break: break-all;
            padding: 8px 14px;
            background: var(--border);
            border-radius: 8px;
            font-family: 'SF Mono', 'Fira Code', monospace;
            max-width: 100%;
            overflow: hidden;
            text-overflow: ellipsis;
            white-space: nowrap;
        }
        "#,
        &format!(
            r#"<meta http-equiv="refresh" content="1;url={}">"#,
            target_esc
        ),
    );

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
{head}
<body>
    <div class="redirect-wrap">
        <div class="card">
            <div class="redirect-body">
                <div class="spinner" aria-hidden="true"></div>
                <h1 class="redirect-title">Taking you there…</h1>
                <span class="redirect-url">{target_esc}</span>
                <a href="{target_esc}" class="btn btn-primary">Open Directly</a>
            </div>
        </div>
    </div>
</body>
</html>"#,
        head = page_head,
        target_esc = target_esc,
    )
}
