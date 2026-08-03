use clap::Parser;
use std::collections::HashSet;
use std::io::Write;
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

    // 2. Resolve share target type
    let target = if cli.target.starts_with("http://") || cli.target.starts_with("https://") {
        ShareTarget::Url(cli.target.clone())
    } else {
        let path = PathBuf::from(&cli.target);
        if !path.exists() {
            eprintln!("Error: Target path '{}' does not exist.", cli.target);
            std::process::exit(1);
        }
        if path.is_file() {
            ShareTarget::File(path)
        } else if path.is_dir() {
            ShareTarget::Folder(path)
        } else {
            eprintln!(
                "Error: Target path '{}' is neither a file nor a directory.",
                cli.target
            );
            std::process::exit(1);
        }
    };

    // 3. Handle password protection
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

    // 4. Set up graceful shutdown channel
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

    // Instantiate state early so expiration can access it
    let state = Arc::new(ServerState {
        target: target.clone(),
        auth: auth.clone(),
        shutdown_tx: shutdown_tx.clone(),
        limit,
        active_downloads: Mutex::new(HashSet::new()),
        expired: std::sync::atomic::AtomicBool::new(false),
    });

    // 5. Expiration timer
    if let Some(ref duration_str) = cli.expire {
        let duration = match cli::parse_duration(duration_str) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("Error parsing expiration: {}.", e);
                eprintln!("Please specify a duration using format like '10m', '1h', or '30s'.");
                eprintln!("Example:\n    --expire 15m");
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

    // 7. Resolve network host and port
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
                eprintln!("Try another port using:\n    --port {}", p + 1);
            } else {
                eprintln!("Error allocating port: {}", e);
            }
            std::process::exit(1);
        }
    };

    // 8. Print startup information
    let server_url = format!("http://{}:{}", host, port);

    // get_dir_size may walk a large directory tree; run it off the async executor
    // so it cannot block Tokio's runtime thread during startup.
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

    // 9. Optionally open local browser
    if cli.open {
        open_browser(&server_url);
    }

    // 10. Start listening

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
        ShareTarget::Url(_) => "N/A (Redirect Link)".to_string(),
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

    let lines = vec![
        format!("{:<10} {}", target_label, target_value),
        format!("{:<10} {}", "Size", size_val),
        format!("{:<10} {}", "Address", server_url),
        format!("{:<10} {}", "Security", security_desc),
        format!("{:<10} {}", "Limit", limit_desc),
    ];

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
