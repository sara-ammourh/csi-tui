// ═══════════════════════════════════════════════════════════════════════════════
// 📦 esp_terminal.rs - ESP32 Raw Serial Terminal (Like PuTTY)
// ═══════════════════════════════════════════════════════════════════════════════
// طرفية ESP خام - تعرض كل شيء من ESP مباشرة مثل PuTTY
// Raw ESP terminal - displays everything from ESP directly like PuTTY
// ═══════════════════════════════════════════════════════════════════════════════

use std::io::{self, Read, Write};
use std::time::Duration;

use crossterm::{
    cursor::MoveTo,
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
};

/// Run ESP terminal - raw serial connection like PuTTY
/// تشغيل طرفية ESP - اتصال تسلسلي خام مثل PuTTY
pub fn run_esp_terminal(port_name: &str, baud_rate: u32) -> Result<(), String> {
    // Open serial port
    let mut port = serialport::new(port_name, baud_rate)
        .timeout(Duration::from_millis(10))
        .open()
        .map_err(|e| format!("Failed to open {}: {}", port_name, e))?;
    
    // Clear screen and show connection message
    let mut stdout = io::stdout();
    execute!(stdout, Clear(ClearType::All), MoveTo(0, 0)).map_err(|e| e.to_string())?;
    
    println!("═══════════════════════════════════════════════════════════════");
    println!("  🔌 Connected to {} @ {} baud", port_name, baud_rate);
    println!("  Press Ctrl+] to exit  ");
    println!("═══════════════════════════════════════════════════════════════");
    println!();
    stdout.flush().map_err(|e| e.to_string())?;
    
    // Enable raw mode for character-by-character input
    enable_raw_mode().map_err(|e| e.to_string())?;
    
    // Clear any pending keyboard events (important!)
    // تنظيف أي أحداث لوحة مفاتيح معلقة
    while event::poll(Duration::from_millis(50)).unwrap_or(false) {
        let _ = event::read();
    }
    
    let mut buf = [0u8; 1024];
    
    loop {
        // Read from serial port and print to screen
        match port.read(&mut buf) {
            Ok(n) if n > 0 => {
                // Convert to UTF-8 string (replace invalid bytes)
                // تحويل إلى UTF-8 (استبدال البايتات غير الصالحة)
                let text = String::from_utf8_lossy(&buf[..n]);
                print!("{}", text);
                stdout.flush().map_err(|e| e.to_string())?;
            }
            Ok(_) => {}
            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => {}
            Err(e) => {
                disable_raw_mode().ok();
                return Err(format!("Read error: {}", e));
            }
        }
        
        // Check for keyboard input
        if event::poll(Duration::from_millis(1)).unwrap_or(false) {
            if let Ok(Event::Key(key)) = event::read() {
                // Only handle key press, not release (fixes double character issue on Windows)
                // معالجة الضغط فقط، وليس الإفلات (يصلح مشكلة الحرف المزدوج على Windows)
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                
                match key.code {
                    // Ctrl+] to exit (like PuTTY)
                    KeyCode::Char(']') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        break;
                    }
                    // Ctrl+C also exits
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        // Send Ctrl+C to ESP
                        let _ = port.write_all(&[0x03]);
                    }
                    // Enter key
                    KeyCode::Enter => {
                        let _ = port.write_all(b"\r\n");
                    }
                    // Backspace
                    KeyCode::Backspace => {
                        let _ = port.write_all(&[0x08]);
                    }
                    // Tab
                    KeyCode::Tab => {
                        let _ = port.write_all(&[0x09]);
                    }
                    // Escape
                    KeyCode::Esc => {
                        let _ = port.write_all(&[0x1B]);
                    }
                    // Regular character - send to ESP
                    KeyCode::Char(c) => {
                        let mut buf = [0u8; 4];
                        let s = c.encode_utf8(&mut buf);
                        let _ = port.write_all(s.as_bytes());
                    }
                    // Arrow keys
                    KeyCode::Up => { let _ = port.write_all(b"\x1B[A"); }
                    KeyCode::Down => { let _ = port.write_all(b"\x1B[B"); }
                    KeyCode::Right => { let _ = port.write_all(b"\x1B[C"); }
                    KeyCode::Left => { let _ = port.write_all(b"\x1B[D"); }
                    _ => {}
                }
            }
        }
    }
    
    // Cleanup
    disable_raw_mode().map_err(|e| e.to_string())?;
    
    println!();
    println!();
    println!("  🔌 Disconnected from {}", port_name);
    println!("  Press Enter to continue...");
    stdout.flush().map_err(|e| e.to_string())?;
    
    // Wait for Enter
    let mut input = String::new();
    let _ = io::stdin().read_line(&mut input);
    
    Ok(())
}
