//! Embedded HTML templates for QRShare v3.5.4.
//! Vanilla CSS + minimal JS · Light + Dark · Mobile-first · Zero external dependencies.

pub const COMMON_STYLE: &str = include_str!("templates/common.css");
pub const PRISM_CSS: &str = include_str!("templates/prism.css");
pub const PRISM_JS: &str = include_str!("templates/prism.js");

// ─── Common JavaScript ────────────────────────────────────────────────────────
pub const COMMON_JS: &str = r#"
// ── Copy to clipboard with visual feedback ──────────────────
async function copyToClipboard(text, buttonEl) {
    var labelEl = buttonEl.querySelector('.copy-label') || buttonEl;
    var iconEl = buttonEl.querySelector('.copy-icon');
    var originalText = labelEl.textContent;
    var ICON_OK = '<polyline points="20 6 9 17 4 12"/>';
    var originalIcon = iconEl ? iconEl.innerHTML : '';

    function showSuccess() {
        labelEl.textContent = '\u2713 Copied!';
        if (iconEl) iconEl.innerHTML = ICON_OK;
        buttonEl.classList.add('copied');
        setTimeout(function() {
            labelEl.textContent = originalText;
            if (iconEl) iconEl.innerHTML = originalIcon;
            buttonEl.classList.remove('copied');
        }, 2000);
    }

    if (navigator.clipboard && navigator.clipboard.writeText) {
        try { await navigator.clipboard.writeText(text); showSuccess(); return; } catch (_) {}
    }
    // Fallback for older browsers / HTTP
    try {
        var ta = document.createElement('textarea');
        ta.value = text;
        ta.style.cssText = 'position:fixed;top:0;left:0;opacity:0';
        document.body.appendChild(ta);
        ta.focus(); ta.select();
        var ok = document.execCommand('copy');
        document.body.removeChild(ta);
        if (ok) showSuccess();
    } catch (_) {}
}

// ── Share via Web Share API, fall back to copy URL ──────────
async function sharePage(title, url, buttonEl) {
    if (navigator.share) {
        try { await navigator.share({ title: title, url: url }); return; } catch (_) {}
    }
    var labelEl = buttonEl.querySelector('.share-label') || buttonEl;
    var originalText = labelEl.textContent;
    await copyToClipboard(url, buttonEl);
    labelEl.textContent = '\u2713 Link Copied!';
    setTimeout(function() { labelEl.textContent = originalText; }, 2000);
}

// ── Tap-to-zoom for preview images ──────────────────────────
(function() {
    document.addEventListener('DOMContentLoaded', function() {
        var imgs = document.querySelectorAll('[data-zoomable]');
        imgs.forEach(function(img) {
            img.addEventListener('click', function() {
                img.classList.toggle('zoomed');
            });
            img.addEventListener('keydown', function(e) {
                if (e.key === 'Enter' || e.key === ' ') {
                    e.preventDefault();
                    img.classList.toggle('zoomed');
                }
            });
            img.setAttribute('tabindex', '0');
            img.setAttribute('role', 'button');
            img.setAttribute('aria-label', 'Tap to zoom');
        });
    });
})();
"#;

// ─── File type icon ──────────────────────────────────────────────────────────

fn file_icon_svg(mime: &str) -> &'static str {
    if mime.starts_with("image/") {
        r#"<svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><rect x="3" y="3" width="18" height="18" rx="2.5"/><circle cx="8.5" cy="8.5" r="1.5"/><path d="m21 15-5-5L5 21"/></svg>"#
    } else if mime.starts_with("video/") {
        r#"<svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="m22 8-6 4 6 4V8z"/><rect x="2" y="5" width="14" height="14" rx="2.5"/></svg>"#
    } else if mime.starts_with("audio/") {
        r#"<svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M9 18V5l12-2v13"/><circle cx="6" cy="18" r="3"/><circle cx="18" cy="16" r="3"/></svg>"#
    } else if mime == "application/pdf" {
        r#"<svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><path d="M14 2v6h6"/><path d="M16 13H8M16 17H8M10 9H8"/></svg>"#
    } else if mime == "application/zip"
        || mime == "application/x-zip-compressed"
        || mime == "application/gzip"
        || mime == "application/x-tar"
        || mime == "application/x-7z-compressed"
    {
        r#"<svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M21 8v13H3V8"/><path d="M1 3h22v5H1z"/><path d="M10 12h4"/></svg>"#
    } else if mime.starts_with("text/") {
        r#"<svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><path d="M14 2v6h6"/><path d="M16 13H8M16 17H8"/></svg>"#
    } else {
        r#"<svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><path d="M14 2v6h6"/></svg>"#
    }
}

// ─── HTML <head> fragment ─────────────────────────────────────────────────────

fn head(title: &str, extra_style: &str, extra_head: &str) -> String {
    format!(
        "<head>
    <meta charset=\"UTF-8\">
    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0, viewport-fit=cover\">
    <meta name=\"theme-color\" media=\"(prefers-color-scheme: dark)\"  content=\"#0A0A0B\">
    <meta name=\"theme-color\" media=\"(prefers-color-scheme: light)\" content=\"#FAFAFA\">
    <title>{title}</title>
    <style>{common}{extra}</style>
    {extra_head}
</head>",
        title = title,
        common = COMMON_STYLE,
        extra = extra_style,
        extra_head = extra_head,
    )
}

// ─── Footer ───────────────────────────────────────────────────────────────────

fn brand_footer() -> &'static str {
    r#"<footer class="footer">
        Made with ❤️ by <a href="https://github.com/bharadwajsanket" target="_blank" rel="noopener">Sanket Bharadwaj</a>
        <span style="margin: 0 6px; opacity: 0.35;">·</span>
        <a href="https://github.com/bharadwajsanket/QRShare" target="_blank" rel="noopener">GitHub</a>
    </footer>"#
}

// ─── Master Page Layout ───────────────────────────────────────────────────────

pub fn render_page_layout(
    browser_title: &str,
    icon_svg: &str,
    display_title: &str,
    content_type_label: &str,
    subtitle_html: &str,
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

    let wordmark_type = if content_type_label.is_empty() {
        String::new()
    } else {
        format!(
            r#"<span class="wordmark-sep">·</span><span class="wordmark-type">{}</span>"#,
            content_type_label
        )
    };

    let subtitle_section = if subtitle_html.is_empty() {
        String::new()
    } else {
        format!(r#"<p class="card-subtitle">{}</p>"#, subtitle_html)
    };

    let page_head = head(browser_title, "", &prism_style);

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
{head}
<body>
    <div class="page">
        <header class="page-header">
            <span class="wordmark">QRShare{wordmark_type}</span>
        </header>

        <main class="card">
            {preview_area_html}

            <div class="card-body">
                <div class="title-row">
                    <div class="title-icon" aria-hidden="true">{icon_svg}</div>
                    <div>
                        <h1 class="card-title">{display_title}</h1>
                        {subtitle_section}
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
        wordmark_type = wordmark_type,
        preview_area_html = preview_area_html,
        icon_svg = icon_svg,
        display_title = display_title,
        subtitle_section = subtitle_section,
        metadata_html = metadata_html,
        actions_html = actions_html,
        common_js = COMMON_JS,
        extra_js = extra_js,
        footer = brand_footer(),
        prism_script = prism_script,
    )
}

// ─── Metadata grid ────────────────────────────────────────────────────────────

pub fn build_metadata(
    target_type: &str,
    session_time: &chrono::DateTime<chrono::Local>,
    expire_str: Option<&str>,
    limit: Option<usize>,
    downloads: usize,
) -> String {
    let time_str = session_time.format("%I:%M %p").to_string();
    let time_str = if time_str.starts_with('0') { &time_str[1..] } else { &time_str };
    let shared_val = format!("{} \u{00b7} {}", session_time.format("%b %d, %Y"), time_str);

    let expiration_val = if let Some(exp) = expire_str {
        exp.to_string()
    } else if let Some(lim) = limit {
        if lim == 1 { "1 download".to_string() } else { format!("{} downloads", lim) }
    } else {
        "Never".to_string()
    };

    let downloads_val = if let Some(lim) = limit {
        format!("{} / {}", downloads, lim)
    } else {
        downloads.to_string()
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
                <span class="metadata-label">Expires</span>
                <span class="metadata-value">{}</span>
            </div>
            <div class="metadata-item">
                <span class="metadata-label">Downloads</span>
                <span class="metadata-value">{}</span>
            </div>
        </div>"#,
        target_type, shared_val, expiration_val, downloads_val
    )
}

// ─── Password page ────────────────────────────────────────────────────────────

pub fn render_password_page(error_msg: Option<&str>) -> String {
    let error_html = match error_msg {
        Some(msg) => format!(
            r#"<p class="pw-error" role="alert" id="pw-error">{}</p>"#,
            crate::util::html_escape(msg)
        ),
        None => String::new(),
    };

    // aria-describedby connects input to error when present
    let input_aria = if error_msg.is_some() {
        r#" aria-describedby="pw-error""#
    } else {
        ""
    };

    let page_head = head("Protected Share \u{2014} QRShare", "body { justify-content: center; }", "");

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
{head}
<body>
    <div class="pw-wrap">
        <div class="card">
            <div class="pw-body">
                <div class="pw-heading">
                    <div class="pw-icon" aria-hidden="true">
                        <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                            <rect x="3" y="11" width="18" height="11" rx="2"/>
                            <path d="M7 11V7a5 5 0 0 1 10 0v4"/>
                        </svg>
                    </div>
                    <h1 class="pw-title">Protected Share</h1>
                    <p class="pw-sub">Enter the password to access this content.</p>
                </div>

                <form action="/auth" method="POST" class="pw-form" novalidate>
                    {error_html}
                    <input
                        type="password"
                        name="password"
                        class="input"
                        placeholder="Password"
                        aria-label="Password"{input_aria}
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
        input_aria = input_aria,
        footer = brand_footer(),
    )
}

// ─── File page ────────────────────────────────────────────────────────────────

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
    let browser_title = format!("{} \u{2022} QRShare", name);
    let icon_svg = file_icon_svg(mime).to_string();
    let display_title = crate::util::html_escape(name);
    let subtitle = crate::util::html_escape(size_str);

    let metadata_html = build_metadata("File", session_time, expire_str, limit, downloads);

    let actions_html = format!(
        r#"<a href="{download_url}" class="btn btn-primary" download aria-label="Download {name}">
               <svg class="icon" viewBox="0 0 24 24" aria-hidden="true"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/></svg>
               Download
           </a>
           <button class="btn btn-secondary" id="share-btn" type="button" aria-label="Share {name}">
               <svg class="icon" viewBox="0 0 24 24" aria-hidden="true"><circle cx="18" cy="5" r="3"/><circle cx="6" cy="12" r="3"/><circle cx="18" cy="19" r="3"/><line x1="8.59" y1="13.51" x2="15.42" y2="17.49"/><line x1="15.41" y1="6.51" x2="8.59" y2="10.49"/></svg>
               <span class="share-label">Share</span>
           </button>"#,
        download_url = download_url,
        name = crate::util::html_escape(name)
    );

    let extra_js = format!(
        "var shareBtn = document.getElementById('share-btn');
         shareBtn.addEventListener('click', function() {{
             sharePage('{}', window.location.href, shareBtn);
         }});",
        name.replace('\'', "\\'")
    );

    render_page_layout(
        &browser_title,
        &icon_svg,
        &display_title,
        "File",
        &subtitle,
        preview_html,
        &metadata_html,
        &actions_html,
        &extra_js,
        is_code,
    )
}

// ─── Folder page ──────────────────────────────────────────────────────────────

pub fn render_folder_page(
    dir_name: &str,
    breadcrumbs_html: &str,
    items_html: &str,
    item_count: usize,
    zip_url: &str,
    session_time: &chrono::DateTime<chrono::Local>,
    expire_str: Option<&str>,
    limit: Option<usize>,
    downloads: usize,
) -> String {
    let browser_title = format!("{} \u{2022} QRShare", dir_name);
    let icon_svg = r#"<svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg>"#.to_string();
    let display_title = crate::util::html_escape(dir_name);

    let subtitle = if item_count == 1 {
        "1 item".to_string()
    } else {
        format!("{} items", item_count)
    };

    let preview_html = format!(
        r#"<div class="folder-breadcrumbs">
            <nav class="breadcrumbs" aria-label="Directory path">{}</nav>
        </div>
        <div class="item-list" role="list">{}</div>"#,
        breadcrumbs_html, items_html
    );

    let metadata_html = build_metadata("Folder", session_time, expire_str, limit, downloads);

    let actions_html = format!(
        r#"<a href="{zip_url}" class="btn btn-primary" aria-label="Download folder as ZIP">
               <svg class="icon" viewBox="0 0 24 24" aria-hidden="true"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/></svg>
               Download ZIP
           </a>
           <button class="btn btn-secondary" id="share-btn" type="button" aria-label="Share folder">
               <svg class="icon" viewBox="0 0 24 24" aria-hidden="true"><circle cx="18" cy="5" r="3"/><circle cx="6" cy="12" r="3"/><circle cx="18" cy="19" r="3"/><line x1="8.59" y1="13.51" x2="15.42" y2="17.49"/><line x1="15.41" y1="6.51" x2="8.59" y2="10.49"/></svg>
               <span class="share-label">Share</span>
           </button>"#,
        zip_url = zip_url
    );

    let extra_js = format!(
        "var shareBtn = document.getElementById('share-btn');
         shareBtn.addEventListener('click', function() {{
             sharePage('{}', window.location.href, shareBtn);
         }});",
        dir_name.replace('\'', "\\'")
    );

    render_page_layout(
        &browser_title,
        &icon_svg,
        &display_title,
        "Folder",
        &subtitle,
        &preview_html,
        &metadata_html,
        &actions_html,
        &extra_js,
        false,
    )
}

// ─── URL Page ─────────────────────────────────────────────────────────────────

pub fn render_url_page(
    url: &str,
    session_time: &chrono::DateTime<chrono::Local>,
    expire_str: Option<&str>,
    limit: Option<usize>,
    downloads: usize,
) -> String {
    let url_esc = crate::util::html_escape(url);

    // Extract domain for display and browser title
    let domain = url.split("://").nth(1).unwrap_or(url).split('/').next().unwrap_or("URL");
    let domain_esc = crate::util::html_escape(domain);
    let browser_title = format!("{} \u{2022} QRShare", domain);

    let icon_svg = r#"<svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"/><path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"/></svg>"#.to_string();

    // Domain shown as clickable link — immediately actionable
    let preview_html = format!(
        r#"<div class="url-display">
            <a href="{url_esc}" target="_blank" rel="noopener" class="url-domain" aria-label="Open {domain_esc}">{domain_esc}</a>
            <span class="url-full">{url_esc}</span>
        </div>"#,
        url_esc = url_esc,
        domain_esc = domain_esc
    );

    let metadata_html = build_metadata("URL", session_time, expire_str, limit, downloads);

    let actions_html = format!(
        r#"<a href="{url_esc}" target="_blank" rel="noopener" class="btn btn-primary" aria-label="Open {domain_esc} in new tab">
               <svg class="icon" viewBox="0 0 24 24" aria-hidden="true"><path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/><polyline points="15 3 21 3 21 9"/><line x1="10" y1="14" x2="21" y2="3"/></svg>
               Open
           </a>
           <button class="btn btn-secondary" id="copy-btn" type="button" aria-label="Copy link to clipboard">
               <svg class="icon copy-icon" viewBox="0 0 24 24" aria-hidden="true"><rect x="9" y="9" width="13" height="13" rx="2" ry="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/></svg>
               <span class="copy-label">Copy Link</span>
           </button>
           <button class="btn btn-secondary" id="share-btn" type="button" aria-label="Share this link">
               <svg class="icon" viewBox="0 0 24 24" aria-hidden="true"><circle cx="18" cy="5" r="3"/><circle cx="6" cy="12" r="3"/><circle cx="18" cy="19" r="3"/><line x1="8.59" y1="13.51" x2="15.42" y2="17.49"/><line x1="15.41" y1="6.51" x2="8.59" y2="10.49"/></svg>
               <span class="share-label">Share</span>
           </button>"#,
        url_esc = url_esc,
        domain_esc = domain_esc
    );

    let extra_js = format!(
        "var copyBtn = document.getElementById('copy-btn');
         copyBtn.addEventListener('click', function() {{
             copyToClipboard('{}', copyBtn);
         }});
         var shareBtn = document.getElementById('share-btn');
         shareBtn.addEventListener('click', function() {{
             sharePage('Shared Link', window.location.href, shareBtn);
         }});",
        url_esc.replace('\'', "\\'")
    );

    render_page_layout(
        &browser_title,
        &icon_svg,
        "Shared Link",
        "URL",
        "",
        &preview_html,
        &metadata_html,
        &actions_html,
        &extra_js,
        false,
    )
}

// ─── Text Page ────────────────────────────────────────────────────────────────

pub fn render_text_page(
    text: &str,
    preview_html: &str,
    is_code: bool,
    download_filename: &str,
    session_time: &chrono::DateTime<chrono::Local>,
    expire_str: Option<&str>,
    limit: Option<usize>,
    downloads: usize,
) -> String {
    let icon_svg = r#"<svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="16" y1="13" x2="8" y2="13"/><line x1="16" y1="17" x2="8" y2="17"/></svg>"#.to_string();

    let metadata_html = build_metadata("Text", session_time, expire_str, limit, downloads);

    let actions_html = format!(
        r#"<button class="btn btn-primary" id="copy-btn" type="button" aria-label="Copy text to clipboard">
               <svg class="icon copy-icon" viewBox="0 0 24 24" aria-hidden="true"><rect x="9" y="9" width="13" height="13" rx="2" ry="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/></svg>
               <span class="copy-label">Copy</span>
           </button>
           <a href="/raw?download=1" class="btn btn-secondary" download="{filename}" aria-label="Download as text file">
               <svg class="icon" viewBox="0 0 24 24" aria-hidden="true"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/></svg>
               Download
           </a>
           <button class="btn btn-secondary" id="share-btn" type="button" aria-label="Share this text">
               <svg class="icon" viewBox="0 0 24 24" aria-hidden="true"><circle cx="18" cy="5" r="3"/><circle cx="6" cy="12" r="3"/><circle cx="18" cy="19" r="3"/><line x1="8.59" y1="13.51" x2="15.42" y2="17.49"/><line x1="15.41" y1="6.51" x2="8.59" y2="10.49"/></svg>
               <span class="share-label">Share</span>
           </button>"#,
        filename = download_filename
    );

    let text_escaped_js = text
        .replace('\\', "\\\\")
        .replace('\'', "\\'")
        .replace('\n', "\\n")
        .replace('\r', "\\r");

    let extra_js = format!(
        "var copyBtn = document.getElementById('copy-btn');
         copyBtn.addEventListener('click', function() {{
             copyToClipboard('{}', copyBtn);
         }});
         var shareBtn = document.getElementById('share-btn');
         shareBtn.addEventListener('click', function() {{
             sharePage('Shared Text', window.location.href, shareBtn);
         }});",
        text_escaped_js
    );

    render_page_layout(
        "Shared Text \u{2022} QRShare",
        &icon_svg,
        "Shared Text",
        "Text",
        "",
        preview_html,
        &metadata_html,
        &actions_html,
        &extra_js,
        is_code,
    )
}

// ─── Clipboard Image Page ─────────────────────────────────────────────────────

pub fn render_clipboard_image_page(
    session_time: &chrono::DateTime<chrono::Local>,
    expire_str: Option<&str>,
    limit: Option<usize>,
    downloads: usize,
) -> String {
    let icon_svg = r#"<svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><rect x="3" y="3" width="18" height="18" rx="2" ry="2"/><circle cx="8.5" cy="8.5" r="1.5"/><polyline points="21 15 16 10 5 21"/></svg>"#.to_string();

    let preview_html = r#"<img src="/raw" class="preview-media" alt="Clipboard image" loading="lazy" data-zoomable>"#.to_string();

    let metadata_html = build_metadata("Image (Clipboard)", session_time, expire_str, limit, downloads);

    let actions_html = r#"<a href="/raw?download=1" class="btn btn-primary" download="qrshare-clip.png" aria-label="Download clipboard image">
               <svg class="icon" viewBox="0 0 24 24" aria-hidden="true"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/></svg>
               Download
           </a>
           <button class="btn btn-secondary" id="share-btn" type="button" aria-label="Share this image">
               <svg class="icon" viewBox="0 0 24 24" aria-hidden="true"><circle cx="18" cy="5" r="3"/><circle cx="6" cy="12" r="3"/><circle cx="18" cy="19" r="3"/><line x1="8.59" y1="13.51" x2="15.42" y2="17.49"/><line x1="15.41" y1="6.51" x2="8.59" y2="10.49"/></svg>
               <span class="share-label">Share</span>
           </button>"#.to_string();

    let extra_js = "var shareBtn = document.getElementById('share-btn'); shareBtn.addEventListener('click', function() { sharePage('Shared Image', window.location.href, shareBtn); });".to_string();

    render_page_layout(
        "Clipboard \u{2022} QRShare",
        &icon_svg,
        "Shared Image",
        "Clipboard",
        "",
        &preview_html,
        &metadata_html,
        &actions_html,
        &extra_js,
        false,
    )
}
