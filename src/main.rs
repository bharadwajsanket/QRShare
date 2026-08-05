use clap::Parser;
use std::collections::HashSet;
use std::io::{IsTerminal, Write};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::sync::watch;

mod cli;
mod error;
mod network;
mod qr;
mod security;
mod server;
mod session;
mod templates;
mod util;
mod zip;

use error::AppError;
use server::{ServerState, ShareTarget};
use session::AuthConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Parse command line arguments
    let cli = cli::Cli::parse();

    // 3. Resolve share target type
    let target = if let Some(ref text) = cli.text {
        ShareTarget::Text(text.clone())
    } else if cli.clipboard {
        match arboard::Clipboard::new() {
            Ok(mut cb) => {
                if let Ok(img) = cb.get_image() {
                    match encode_image_to_png(&img) {
                        Ok(png_bytes) => ShareTarget::Image(png_bytes),
                        Err(e) => {
                            eprintln!("Error: Failed to encode clipboard image: {}", e);
                            std::process::exit(1);
                        }
                    }
                } else if let Ok(text) = cb.get_text() {
                    ShareTarget::Text(text)
                } else {
                    eprintln!(
                        "Error: Clipboard does not contain supported content (text or image)."
                    );
                    std::process::exit(1);
                }
            }
            Err(e) => {
                eprintln!("Error: Failed to open clipboard: {}", e);
                #[cfg(target_os = "linux")]
                eprintln!("Note: Sharing from clipboard on Linux requires a running X11 or Wayland session.");
                std::process::exit(1);
            }
        }
    } else if !std::io::stdin().is_terminal() {
        let mut buffer = String::new();
        if let Err(e) = std::io::Read::read_to_string(&mut std::io::stdin(), &mut buffer) {
            eprintln!("Error: Failed to read stdin: {}", e);
            std::process::exit(1);
        }
        ShareTarget::Text(buffer)
    } else if let Some(ref target_str) = cli.target {
        if target_str.starts_with("http://") || target_str.starts_with("https://") {
            ShareTarget::Url(target_str.clone())
        } else {
            let path = PathBuf::from(target_str);
            if !path.exists() {
                eprintln!("Error: Target path '{}' does not exist.", target_str);
                std::process::exit(1);
            }
            if path.is_file() {
                ShareTarget::File(path)
            } else if path.is_dir() {
                ShareTarget::Folder(path)
            } else {
                eprintln!(
                    "Error: Target path '{}' is neither a file nor a directory.",
                    target_str
                );
                std::process::exit(1);
            }
        }
    } else {
        eprintln!("Error: No share target specified.");
        eprintln!("Please specify a file/folder/URL to share, or use --text, --clipboard, or pipe from stdin.");
        eprintln!("See 'qrshare --help' for details.");
        std::process::exit(1);
    };

    // 4. Handle password protection
    let mut password = cli.password.clone();
    if let Some(ref p) = password {
        if p == "__prompt__" {
            // Prompt securely on terminal
            print!("Enter password: ");
            std::io::stdout().flush()?;
            let prompted = rpassword::read_password()
                .map_err(|e| AppError::Internal(format!("Failed to read password: {}", e)))?;
            let trimmed = prompted.trim();
            if trimmed.is_empty() {
                eprintln!("Error: Password cannot be empty.");
                std::process::exit(1);
            }
            password = Some(trimmed.to_string());
        }
    }
    let auth = AuthConfig::new(password);

    // 5. Set up graceful shutdown channel
    let (shutdown_tx, _shutdown_rx) = watch::channel(false);

    // 6. Download limit parameters
    let limit = if let Some(l) = cli.limit {
        Some(l)
    } else if cli.once {
        Some(1)
    } else if cli.twice {
        Some(2)
    } else {
        None
    };

    // Get current local time for session sharing timestamp
    let session_time = chrono::Local::now();

    // Instantiate state early so expiration can access it
    let state = Arc::new(ServerState {
        target: target.clone(),
        auth: auth.clone(),
        shutdown_tx: shutdown_tx.clone(),
        limit,
        active_downloads: Mutex::new(HashSet::new()),
        expired: std::sync::atomic::AtomicBool::new(false),
        session_time,
        expire_str: cli.expire.clone(),
    });

    // 7. Expiration timer
    if let Some(ref duration_str) = cli.expire {
        let duration = match cli::parse_duration(duration_str) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("Error parsing expiration: {}.", e);
                std::process::exit(1);
            }
        };

        let tx = shutdown_tx.clone();
        let dur_str = duration_str.clone();
        let state_clone = state.clone();
        tokio::spawn(async move {
            tokio::time::sleep(duration).await;
            println!(
                "\n⏳ Expiration timer (duration: {}) reached. Shutting down...",
                dur_str
            );
            state_clone
                .expired
                .store(true, std::sync::atomic::Ordering::SeqCst);
            let _ = tx.send(true);
        });
    }

    // 8. Resolve network host and port
    let host = if let Some(ref h) = cli.host {
        h.clone()
    } else {
        network::get_local_ip().to_string()
    };

    let port = match network::find_available_port(cli.port) {
        Ok(p) => p,
        Err(e) => {
            if let Some(p) = cli.port {
                eprintln!("Error: Could not bind to port {}.", p);
            } else {
                eprintln!("Error allocating port: {}", e);
            }
            std::process::exit(1);
        }
    };

    // 9. Attempt mDNS registration for qrshare.local
    let _mdns_daemon = if host != "127.0.0.1" {
        match mdns_sd::ServiceDaemon::new() {
            Ok(daemon) => {
                let service_type = "_http._tcp.local.";
                let instance_name = "qrshare";
                let host_name = "qrshare.local.";
                let properties = std::collections::HashMap::new();
                match mdns_sd::ServiceInfo::new(
                    service_type,
                    instance_name,
                    host_name,
                    &host,
                    port,
                    properties,
                ) {
                    Ok(info) => {
                        let info = info.enable_addr_auto();
                        let _ = daemon.register(info);
                        Some(daemon)
                    }
                    Err(_) => None,
                }
            }
            Err(_) => None,
        }
    } else {
        None
    };

    let server_url = format!("http://{}:{}", host, port);

    // 10. Print startup information
    let target_for_banner = target.clone();
    let (banner_host, banner_port) = (host.clone(), port);
    let has_pass = cli.password.is_some();
    tokio::task::spawn_blocking(move || {
        print_startup_banner(
            &banner_host,
            banner_port,
            &target_for_banner,
            limit,
            has_pass,
        );
    })
    .await
    .unwrap_or(());

    // Render the QR code in the terminal
    if let Err(e) = qr::print_qr_code(&server_url) {
        eprintln!("Warning: Failed to print QR code: {}", e);
    }

    println!("\n────────────────────────────\n");
    println!("📡 Waiting for connections...");

    // 11. Optionally open local browser
    if cli.open {
        open_browser(&server_url);
    }

    // 12. Start listening
    let addr: SocketAddr = format!("0.0.0.0:{}", port)
        .parse()
        .map_err(|e| AppError::Internal(format!("Invalid socket address: {}", e)))?;

    server::start_server(state, addr).await?;

    Ok(())
}

/// Helper to trigger platform-specific default browser opening commands
fn open_browser(url: &str) {
    #[cfg(target_os = "macos")]
    let _ = std::process::Command::new("open").arg(url).spawn();
    #[cfg(target_os = "linux")]
    let _ = std::process::Command::new("xdg-open").arg(url).spawn();
    #[cfg(target_os = "windows")]
    let _ = std::process::Command::new("cmd")
        .args(["/C", "start", url])
        .spawn();
}

fn print_startup_banner(
    host: &str,
    port: u16,
    target: &ShareTarget,
    limit: Option<usize>,
    has_password: bool,
) {
    let server_url = format!("http://{}:{}", host, port);
    let (target_label, target_value) = match target {
        ShareTarget::Url(url) => ("URL", url.clone()),
        ShareTarget::File(path) => {
            let filename = path
                .file_name()
                .and_then(|f| f.to_str())
                .unwrap_or("unknown");
            ("File", filename.to_string())
        }
        ShareTarget::Folder(path) => {
            let dirname = path
                .file_name()
                .and_then(|f| f.to_str())
                .unwrap_or("unknown");
            ("Folder", dirname.to_string())
        }
        ShareTarget::Text(text) => {
            let preview = if text.len() > 30 {
                format!("{}...", text.chars().take(27).collect::<String>())
            } else {
                text.clone()
            };
            ("Text", preview.replace('\n', " "))
        }
        ShareTarget::Image(_) => ("Clipboard Image", "PNG Image".to_string()),
    };

    let size_val = match target {
        ShareTarget::File(path) => {
            let bytes = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
            util::format_size(bytes)
        }
        ShareTarget::Folder(path) => {
            let bytes = get_dir_size(path);
            util::format_size(bytes)
        }
        ShareTarget::Url(_) => "N/A (Preview Link)".to_string(),
        ShareTarget::Text(text) => util::format_size(text.len() as u64),
        ShareTarget::Image(bytes) => util::format_size(bytes.len() as u64),
    };

    let limit_desc = match limit {
        Some(1) => "Once".to_string(),
        Some(2) => "Twice".to_string(),
        Some(n) => format!("{} downloads", n),
        None => "Unlimited".to_string(),
    };

    let security_desc = if has_password {
        "Password Protected"
    } else {
        "Open Access (LAN Only)"
    };

    let mut lines = vec![
        format!("{:<10} {}", target_label, target_value),
        format!("{:<10} {}", "Size", size_val),
        format!("{:<10} {}", "Address", server_url),
    ];

    if host != "127.0.0.1" {
        let dns_url = if port == 80 {
            "http://qrshare.local".to_string()
        } else {
            format!("http://qrshare.local:{}", port)
        };
        lines.push(format!("{:<10} {}", "Local DNS", dns_url));
    }

    lines.push(format!("{:<10} {}", "Security", security_desc));
    lines.push(format!("{:<10} {}", "Limit", limit_desc));

    let content_width = 54;

    println!("\x1b[1;36m");
    println!("  ██████╗  ██████╗  ███████╗██╗  ██╗  █████╗  ██████╗  ███████╗");
    println!(" ██╔═══██╗ ██╔══██╗ ██╔════╝██║  ██║ ██╔══██╗ ██╔══██╗ ██╔════╝");
    println!(" ██║   ██║ ██████╔╝ ███████╗███████║ ███████║ ██████╔╝ █████╗  ");
    println!(" ██║ ▄ ██║ ██╔══██╗ ╚════██║██╔══██║ ██╔══██║ ██╔══██╗ ██╔══╝  ");
    println!(" ╚██████╔╝ ██║  ██║ ███████║██║  ██║ ██║  ██║ ██║  ██║ ███████╗");
    println!("  ╚═══██╔╝  ╚═╝  ╚═╝ ╚══════╝╚═╝  ╚═╝ ╚═╝  ╚═╝ ╚═╝  ╚═╝ ╚══════╝");
    println!("\x1b[0m");
    println!("                 \x1b[1;36mFast\x1b[0m \x1b[2m•\x1b[0m \x1b[1;32mPrivate\x1b[0m \x1b[2m•\x1b[0m \x1b[1;35mLocal\x1b[0m\n");

    println!("┌────────────────────────────────────────────────────────┐");
    for line in lines {
        let display_line = if line.chars().count() > content_width {
            let mut truncated: String = line.chars().take(content_width - 3).collect();
            truncated.push_str("...");
            truncated
        } else {
            let padding = content_width - line.chars().count();
            format!("{}{}", line, " ".repeat(padding))
        };
        println!("│ {} │", display_line);
    }
    println!("└────────────────────────────────────────────────────────┘");
}

fn get_dir_size(path: &std::path::Path) -> u64 {
    let mut size = 0;
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            if let Ok(meta) = entry.metadata() {
                if meta.is_file() {
                    size += meta.len();
                } else if meta.is_dir() {
                    size += get_dir_size(&entry.path());
                }
            }
        }
    }
    size
}

fn encode_image_to_png(img: &arboard::ImageData) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut png_bytes = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut png_bytes, img.width as u32, img.height as u32);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header()?;
        writer.write_image_data(&img.bytes)?;
    }
    Ok(png_bytes)
}
