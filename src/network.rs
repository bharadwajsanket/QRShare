use crate::error::AppError;
use std::net::{IpAddr, Ipv4Addr, TcpListener, UdpSocket};

/// Detects the machine's primary local IP address.
/// Attempts to establish a UDP connection to a public IP (does not send data)
/// to determine the appropriate network interface and its IP address.
/// Falls back to loopback if the local network is unreachable.
pub fn get_local_ip() -> IpAddr {
    // Connect to Cloudflare DNS (1.1.1.1) to resolve the routing path and local IP.
    // This is entirely offline-safe for interface selection and sends no actual network packets.
    if let Ok(socket) = UdpSocket::bind("0.0.0.0:0") {
        if socket.connect("1.1.1.1:80").is_ok() {
            if let Ok(local_addr) = socket.local_addr() {
                return local_addr.ip();
            }
        }
    }
    // Fallback: Loopback address
    IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))
}

/// Finds a free TCP port available for binding.
/// If a preferred port is provided, verifies if it's available. If not, binds to port 0
/// to let the OS assign an available one automatically.
pub fn find_available_port(preferred_port: Option<u16>) -> Result<u16, AppError> {
    if let Some(port) = preferred_port {
        // Verify preferred port is bindable
        match TcpListener::bind(format!("0.0.0.0:{}", port)) {
            Ok(_) => Ok(port),
            Err(e) => Err(AppError::Io(std::io::Error::new(
                std::io::ErrorKind::AddrInUse,
                format!("Port {} is already in use: {}", port, e),
            ))),
        }
    } else {
        // Let the OS allocate a random free port
        let listener = TcpListener::bind("0.0.0.0:0")
            .map_err(|e| AppError::Internal(format!("Failed to bind to any free port: {}", e)))?;
        let port = listener
            .local_addr()
            .map_err(|e| AppError::Internal(format!("Failed to read local port: {}", e)))?
            .port();
        Ok(port)
    }
}
