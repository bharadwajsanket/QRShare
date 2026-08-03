use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(
    name = "qrshare",
    version = env!("CARGO_PKG_VERSION"),
    author = "Sanket Bharadwaj <https://github.com/bharadwajsanket>",
    about = "Fast. Private. Local. Share files, folders, or URLs over local network instantly."
)]
pub struct Cli {
    /// File, directory, or URL to share
    #[arg(required = true)]
    pub target: String,

    /// Password to protect the shared resource. If passed without a value, you will be prompted securely.
    #[arg(
        long,
        short = 'P',
        num_args = 0..=1,
        default_missing_value = "__prompt__",
        help_heading = "Security Options"
    )]
    pub password: Option<String>,

    /// Expiration time (e.g. 5m, 30m, 1h)
    #[arg(long, short = 'e', help_heading = "Sharing Options")]
    pub expire: Option<String>,

    /// Shut down after N successful downloads
    #[arg(
        long,
        short = 'l',
        conflicts_with_all = &["once", "twice"],
        help_heading = "Sharing Options"
    )]
    pub limit: Option<usize>,

    /// Shut down after 1 successful download
    #[arg(long, conflicts_with = "twice", help_heading = "Sharing Options")]
    pub once: bool,

    /// Shut down after 2 successful downloads
    #[arg(long, help_heading = "Sharing Options")]
    pub twice: bool,

    /// Port to listen on (default: auto-select an available port)
    #[arg(long, short = 'p', help_heading = "Network Options")]
    pub port: Option<u16>,

    /// Host IP address to bind to (default: auto-detected local IP)
    #[arg(long, short = 'H', help_heading = "Network Options")]
    pub host: Option<String>,

    /// Open the sharing URL in the local default browser
    #[arg(long, short = 'o', help_heading = "General Options")]
    pub open: bool,
}

/// Parses a duration string (e.g., "5m", "1h", "30s") into a std::time::Duration.
/// Supports suffix units: 's' for seconds, 'm' for minutes, and 'h' for hours.
/// Defaults to seconds if no unit is provided.
pub fn parse_duration(s: &str) -> Result<std::time::Duration, String> {
    let s = s.trim().to_lowercase();
    if s.is_empty() {
        return Err("Duration string is empty".to_string());
    }

    let (val_str, unit) = s.split_at(s.len() - 1);

    // Check if the last character is a digit or unit
    let last_char = s
        .chars()
        .last()
        .ok_or_else(|| "Duration string is empty".to_string())?;
    if last_char.is_ascii_digit() {
        // If no unit, assume seconds
        let val: u64 = s
            .parse()
            .map_err(|_| format!("Invalid numeric value in duration: '{}'", s))?;
        return Ok(std::time::Duration::from_secs(val));
    }

    let val: u64 = val_str
        .parse()
        .map_err(|_| format!("Invalid numeric value in duration: '{}'", val_str))?;

    match unit {
        "s" => Ok(std::time::Duration::from_secs(val)),
        "m" => Ok(std::time::Duration::from_secs(val * 60)),
        "h" => Ok(std::time::Duration::from_secs(val * 3600)),
        _ => Err(format!(
            "Unknown duration unit '{}'. Use 's', 'm', or 'h'",
            unit
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_duration() {
        assert_eq!(
            parse_duration("5s").unwrap(),
            std::time::Duration::from_secs(5)
        );
        assert_eq!(
            parse_duration("10m").unwrap(),
            std::time::Duration::from_secs(600)
        );
        assert_eq!(
            parse_duration("2h").unwrap(),
            std::time::Duration::from_secs(7200)
        );
        assert_eq!(
            parse_duration("60").unwrap(),
            std::time::Duration::from_secs(60)
        );
        assert!(parse_duration("abc").is_err());
        assert!(parse_duration("").is_err());
        assert!(parse_duration("5x").is_err());
    }
}
