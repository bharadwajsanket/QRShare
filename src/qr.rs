use crate::error::AppError;
use qrcode::QrCode;

/// Renders a URL to a compact terminal QR code and prints it.
/// Uses double-row density unicode characters to keep the height small
/// and scannable on dark background terminals.
pub fn print_qr_code(url: &str) -> Result<(), AppError> {
    let code = QrCode::new(url.as_bytes())
        .map_err(|e| AppError::Internal(format!("Failed to generate QR code: {}", e)))?;

    let width = code.width();

    // Print a top spacer
    println!();

    // We add a quiet zone of 2 cells around the QR code.
    // In our model:
    // - true  = Dark module (black)
    // - false = Light module (white)
    // For a dark background terminal:
    // - Light module (white) is represented by drawing foreground pixels ('█')
    // - Dark module (black) is represented by background empty space (' ')
    let get_module = |x: i32, y: i32| -> bool {
        if x >= 0 && x < width as i32 && y >= 0 && y < width as i32 {
            code[(x as usize, y as usize)] == qrcode::types::Color::Dark
        } else {
            false // Quiet zone is light (white)
        }
    };

    // Step by 2 vertically since each character draws 2 rows
    for y in (-2..width as i32 + 2).step_by(2) {
        print!("    "); // Indentation margin
        for x in -2..width as i32 + 2 {
            let top = get_module(x, y);
            let bottom = get_module(x, y + 1);
            let ch = match (top, bottom) {
                (false, false) => '█', // light/light -> full block
                (false, true) => '▀',  // light/dark -> top block
                (true, false) => '▄',  // dark/light -> bottom block
                (true, true) => ' ',   // dark/dark -> space
            };
            print!("{}", ch);
        }
        println!();
    }

    println!();
    Ok(())
}
