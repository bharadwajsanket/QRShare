//! Embedded HTML templates for QRShare.
//! Contains all CSS and JS inline for single-roundtrip instant page loads.
//! Supports both system light and dark themes.

pub const COMMON_STYLE: &str = include_str!("templates/common.css");
pub const PRISM_CSS: &str = include_str!("templates/prism.css");
pub const PRISM_JS: &str = include_str!("templates/prism.js");

pub fn render_password_page(error_msg: Option<&str>) -> String {
    let error_html = match error_msg {
        Some(msg) => format!(
            r#"<div class="error-msg">{}</div>"#,
            crate::util::html_escape(msg)
        ),
        None => String::new(),
    };

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Authentication - QRShare</title>
    <style>
        {style}
        body {{
            justify-content: center;
        }}
        .password-container {{
            max-width: 400px;
            width: 100%;
        }}
        .logo-section {{
            text-align: center;
            margin-bottom: 8px;
        }}
        .logo-title {{
            font-size: 28px;
            font-weight: 800;
            letter-spacing: -1px;
            margin: 0;
        }}
        .logo-subtitle {{
            font-size: 13px;
            color: var(--muted);
            margin: 8px 0 0 0;
        }}
        .input-group {{
            display: flex;
            flex-direction: column;
            gap: 12px;
        }}
        .input-field {{
            background: rgba(0, 0, 0, 0.15);
            border: 1px solid var(--border);
            color: var(--fg);
            height: 52px;
            padding: 0 16px;
            border-radius: 14px;
            font-size: 15px;
            box-sizing: border-box;
            outline: none;
            transition: var(--transition);
        }}
        .input-field:focus {{
            border-color: var(--fg);
            box-shadow: 0 0 0 1px var(--fg);
        }}
        .error-msg {{
            color: #ef4444;
            font-size: 13px;
            font-weight: 500;
            text-align: center;
        }}
    </style>
</head>
<body>
    <div class="container password-container">
        <div class="card">
            <div class="logo-section">
                <h1 class="logo-title">QRShare</h1>
                <p class="logo-subtitle">Secure, local encrypted connection. Enter password to view files.</p>
            </div>
            
            {error_html}

            <form action="/auth" method="POST">
                <div class="input-group">
                    <input type="password" name="password" class="input-field" placeholder="Enter password" aria-label="Password" required autofocus>
                    <button type="submit" class="btn btn-primary" aria-label="Unlock files">Unlock Files</button>
                </div>
            </form>
        </div>
    </div>
    
    <footer class="footer">
        Made with ❤️ by <a href="https://github.com/bharadwajsanket" target="_blank" rel="noopener">Sanket Bharadwaj</a>
    </footer>
</body>
</html>"#,
        style = COMMON_STYLE,
        error_html = error_html
    )
}

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

    // Shared files direct downloads and copy URL Javascript
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0, viewport-fit=cover">
    <title>{name_escaped} - QRShare</title>
    <style>
        {style}
        .header {{
            display: flex;
            align-items: center;
            justify-content: space-between;
            margin-bottom: 8px;
        }}
        .brand {{
            font-size: 18px;
            font-weight: 700;
            letter-spacing: -0.5px;
            margin: 0;
        }}
        .preview-container {{
            display: flex;
            justify-content: center;
            align-items: center;
            width: 100%;
            min-height: 180px;
            background: rgba(0, 0, 0, 0.08);
            border-radius: 16px;
            border: 1px dashed var(--border);
            overflow: hidden;
            box-sizing: border-box;
            padding: 8px;
        }}
        .preview-media {{
            max-width: 100%;
            max-height: 60vh;
            border-radius: 10px;
            object-fit: contain;
            box-shadow: 0 4px 12px rgba(0,0,0,0.15);
        }}
        .preview-fallback {{
            display: flex;
            flex-direction: column;
            align-items: center;
            gap: 12px;
            color: var(--muted);
            text-align: center;
            padding: 40px;
        }}
        .fallback-icon {{
            width: 64px;
            height: 64px;
            stroke: var(--muted);
            stroke-width: 1.5;
            fill: none;
        }}
        .meta-section {{
            display: flex;
            flex-direction: column;
            gap: 4px;
        }}
        .file-name {{
            font-size: 20px;
            font-weight: 700;
            word-break: break-all;
            margin: 0;
            line-height: 1.3;
        }}
        .file-details {{
            font-size: 13px;
            color: var(--muted);
        }}
        .action-grid {{
            display: grid;
            grid-template-columns: 1fr 1fr;
            gap: 12px;
        }}
        @media (max-width: 480px) {{
            .action-grid {{
                grid-template-columns: 1fr;
            }}
        }}
        /* Markdown rendering container overrides */
        .markdown-rendered {{
            width: 100%;
            text-align: left;
            padding: 16px;
            max-height: 60vh;
            overflow-y: auto;
            color: var(--fg);
            line-height: 1.6;
        }}
        .markdown-rendered h1, .markdown-rendered h2, .markdown-rendered h3 {{
            margin-top: 24px;
            margin-bottom: 12px;
            font-weight: 600;
        }}
        .markdown-rendered p {{
            margin-bottom: 16px;
        }}
        .markdown-rendered code {{
            background: rgba(0, 0, 0, 0.1);
            padding: 2px 6px;
            border-radius: 6px;
            font-family: monospace;
            font-size: 0.9em;
        }}
        .code-container {{
            width: 100%;
            text-align: left;
            max-height: 60vh;
            overflow-y: auto;
        }}
        
        /* Premium custom audio player */
        .audio-wrapper {{
            width: 100%;
            padding: 24px;
            box-sizing: border-box;
            background: rgba(0, 0, 0, 0.2);
            border-radius: 16px;
            display: flex;
            flex-direction: column;
            gap: 16px;
        }}
        .audio-player-controls {{
            display: flex;
            align-items: center;
            gap: 16px;
        }}
        .play-pause-btn {{
            width: 44px;
            height: 44px;
            border-radius: 50%;
            background: var(--accent);
            color: var(--accent-fg);
            display: flex;
            align-items: center;
            justify-content: center;
            cursor: pointer;
            border: none;
            transition: var(--transition);
        }}
        .play-pause-btn:hover {{
            transform: scale(1.05);
        }}
        .audio-progress-container {{
            flex-grow: 1;
            display: flex;
            flex-direction: column;
            gap: 6px;
        }}
        .audio-time {{
            display: flex;
            justify-content: space-between;
            font-size: 11px;
            color: var(--muted);
        }}
        .slider-bar {{
            -webkit-appearance: none;
            width: 100%;
            height: 6px;
            border-radius: 3px;
            background: var(--border);
            outline: none;
            cursor: pointer;
        }}
        .slider-bar::-webkit-slider-thumb {{
            -webkit-appearance: none;
            width: 12px;
            height: 12px;
            border-radius: 50%;
            background: var(--accent);
            cursor: pointer;
            transition: var(--transition);
        }}
        .slider-bar::-webkit-slider-thumb:hover {{
            transform: scale(1.2);
        }}
    </style>
    {prism_style}
</head>
<body>
    <div class="container">
        <header class="header">
            <h2 class="brand">QRShare</h2>
            <div class="file-details">{mime}</div>
        </header>

        <main class="card">
            <div class="preview-container">
                {preview_html}
            </div>

            <div class="meta-section">
                <h1 class="file-name">{name_escaped}</h1>
                <div class="file-details">{size_str}</div>
            </div>

            <div class="action-grid">
                <a href="{download_url}" class="btn btn-primary" download="{name_escaped}" aria-label="Download file">
                    <svg class="icon" viewBox="0 0 24 24"><path d="M19.35 10.04C18.67 6.59 15.64 4 12 4 9.11 4 6.6 5.64 5.35 8.04 2.34 8.36 0 10.91 0 14c0 3.31 2.69 6 6 6h13c2.76 0 5-2.24 5-5 0-2.64-2.05-4.78-4.65-4.96zM17 13l-5 5-5-5h3V9h4v4h3z"/></svg>
                    Download File
                </a>
                <button class="btn btn-secondary" id="share-btn" aria-label="Copy sharing link">
                    <svg class="icon" id="share-icon" viewBox="0 0 24 24"><path d="M16 1H4c-1.1 0-2 .9-2 2v14h2V3h12V1zm3 4H8c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h11c1.1 0 2-.9 2-2V7c0-1.1-.9-2-2-2zm0 16H8V7h11v14z"/></svg>
                    <span id="share-text">Copy Link</span>
                </button>
            </div>
        </main>
    </div>

    <footer class="footer">
        Made with ❤️ by <a href="https://github.com/bharadwajsanket" target="_blank" rel="noopener">Sanket Bharadwaj</a>
    </footer>

    <script>
        // Copy link or share
        const shareBtn = document.getElementById('share-btn');
        const shareText = document.getElementById('share-text');
        const shareIcon = document.getElementById('share-icon');

        shareBtn.addEventListener('click', async () => {{
            const url = window.location.origin + '/raw';
            
            // Web Share API support check
            if (navigator.share) {{
                try {{
                    await navigator.share({{
                        title: '{name_escaped}',
                        url: url
                    }});
                    return;
                }} catch (e) {{
                    // Fall back to copy if shared cancelled or fails
                }}
            }}

            // Fallback to Clipboard Copy
            navigator.clipboard.writeText(window.location.href).then(() => {{
                shareText.textContent = 'Copied ✓';
                // SVG checkmark icon replacement
                shareIcon.innerHTML = '<path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z"/>';
                
                setTimeout(() => {{
                    shareText.textContent = 'Copy Link';
                    shareIcon.innerHTML = '<path d="M16 1H4c-1.1 0-2 .9-2 2v14h2V3h12V1zm3 4H8c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h11c1.1 0 2-.9 2-2V7c0-1.1-.9-2-2-2zm0 16H8V7h11v14z"/>';
                }}, 2000);
            }});
        }});
    </script>
    {prism_script}
</body>
</html>"#,
        style = COMMON_STYLE,
        name_escaped = crate::util::html_escape(name),
        size_str = size_str,
        mime = mime,
        preview_html = preview_html,
        download_url = download_url,
        prism_style = prism_style,
        prism_script = prism_script
    )
}

pub fn render_folder_page(
    dir_name: &str,
    breadcrumbs_html: &str,
    items_html: &str,
    zip_url: &str,
) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0, viewport-fit=cover">
    <title>{dir_name_escaped} - QRShare</title>
    <style>
        {style}
        .header {{
            display: flex;
            align-items: center;
            justify-content: space-between;
            margin-bottom: 8px;
        }}
        .brand {{
            font-size: 18px;
            font-weight: 700;
            letter-spacing: -0.5px;
            margin: 0;
        }}
        .breadcrumbs {{
            display: flex;
            align-items: center;
            gap: 6px;
            font-size: 14px;
            color: var(--muted);
            flex-wrap: wrap;
        }}
        .breadcrumbs a {{
            color: var(--muted);
            text-decoration: none;
            transition: var(--transition);
        }}
        .breadcrumbs a:hover {{
            color: var(--fg);
        }}
        .breadcrumbs .separator {{
            color: var(--border);
        }}
        .breadcrumbs .current {{
            color: var(--fg);
            font-weight: 600;
        }}
        .folder-actions {{
            display: flex;
            align-items: center;
            justify-content: space-between;
            border-bottom: 1px solid var(--border);
            padding-bottom: 16px;
            margin-bottom: 8px;
        }}
        .folder-title {{
            font-size: 20px;
            font-weight: 700;
            margin: 0;
        }}
        .item-list {{
            display: flex;
            flex-direction: column;
            gap: 2px;
        }}
        .item-row {{
            display: flex;
            align-items: center;
            justify-content: space-between;
            min-height: 44px;
            padding: 10px 16px;
            border-radius: 12px;
            text-decoration: none;
            color: var(--fg);
            transition: var(--transition);
            box-sizing: border-box;
            outline: none;
        }}
        .item-row:hover, .item-row:focus-visible {{
            background: var(--border);
        }}
        .item-row:focus-visible {{
            outline: 2px solid var(--accent);
            outline-offset: -2px;
        }}
        .item-left {{
            display: flex;
            align-items: center;
            gap: 12px;
            min-width: 0;
        }}
        .item-icon {{
            flex-shrink: 0;
            width: 20px;
            height: 20px;
            color: var(--muted);
        }}
        .item-row:hover .item-icon {{
            color: var(--fg);
        }}
        .item-name {{
            font-size: 14px;
            font-weight: 500;
            white-space: nowrap;
            overflow: hidden;
            text-overflow: ellipsis;
        }}
        .item-right {{
            display: flex;
            align-items: center;
            gap: 16px;
            flex-shrink: 0;
        }}
        .item-size {{
            font-size: 12px;
            color: var(--muted);
        }}
        .btn-small {{
            height: 36px;
            padding: 0 16px;
            border-radius: 10px;
            font-size: 12px;
        }}
    </style>
</head>
<body>
    <div class="container">
        <header class="header">
            <h2 class="brand">QRShare</h2>
            <div class="breadcrumbs">
                {breadcrumbs_html}
            </div>
        </header>

        <main class="card">
            <div class="folder-actions">
                <h1 class="folder-title">Files</h1>
                <a href="{zip_url}" class="btn btn-primary btn-small" aria-label="Download folder ZIP archive">
                    <svg class="icon" viewBox="0 0 24 24"><path d="M19.35 10.04C18.67 6.59 15.64 4 12 4 9.11 4 6.6 5.64 5.35 8.04 2.34 8.36 0 10.91 0 14c0 3.31 2.69 6 6 6h13c2.76 0 5-2.24 5-5 0-2.64-2.05-4.78-4.65-4.96zM12 17l-4-4h3V9h2v4h3l-4 4z"/></svg>
                    Download ZIP
                </a>
            </div>

            <div class="item-list">
                {items_html}
            </div>
        </main>
    </div>

    <footer class="footer">
        Made with ❤️ by <a href="https://github.com/bharadwajsanket" target="_blank" rel="noopener">Sanket Bharadwaj</a>
    </footer>
</body>
</html>"#,
        style = COMMON_STYLE,
        dir_name_escaped = crate::util::html_escape(dir_name),
        breadcrumbs_html = breadcrumbs_html,
        items_html = items_html,
        zip_url = zip_url
    )
}

pub fn render_redirect_page(target_url: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Redirecting - QRShare</title>
    <meta http-equiv="refresh" content="1;url={target_url_escaped}">
    <style>
        {style}
        body {{
            justify-content: center;
        }}
        .redirect-card {{
            text-align: center;
            max-width: 420px;
            width: 100%;
        }}
        .spinner {{
            width: 40px;
            height: 40px;
            border: 3px solid var(--border);
            border-top: 3px solid var(--fg);
            border-radius: 50%;
            margin: 0 auto 24px auto;
            animation: spin 0.8s linear infinite;
        }}
        @keyframes spin {{
            0% {{ transform: rotate(0deg); }}
            100% {{ transform: rotate(360deg); }}
        }}
        .redirect-title {{
            font-size: 20px;
            font-weight: 700;
            margin: 0 0 8px 0;
        }}
        .redirect-text {{
            font-size: 14px;
            color: var(--muted);
            margin: 0 0 24px 0;
            word-break: break-all;
        }}
    </style>
</head>
<body>
    <div class="container redirect-card">
        <div class="card">
            <div class="spinner"></div>
            <h1 class="redirect-title">Redirecting...</h1>
            <p class="redirect-text">Transferring you to {target_url_escaped}</p>
            <a href="{target_url_escaped}" class="btn btn-primary">Open Link Directly</a>
        </div>
    </div>
</body>
</html>"#,
        style = COMMON_STYLE,
        target_url_escaped = crate::util::html_escape(target_url)
    )
}
