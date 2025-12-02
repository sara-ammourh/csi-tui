// ═══════════════════════════════════════════════════════════════════════════════
// 📦 menu.rs - Main Menu (Simple)
// ═══════════════════════════════════════════════════════════════════════════════
// قائمة بسيطة: Set ESP أو View CSI Output
// ═══════════════════════════════════════════════════════════════════════════════

use std::io::{self, Write};
use std::time::Duration;
use crossterm::{
    cursor::MoveTo,
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
};

/// Menu choice
#[derive(Debug, Clone, PartialEq)]
pub enum MenuChoice {
    SetEsp { port: String, baud: u32 },
    ViewCsiOutput,
    Quit,
}

/// Show main menu and get choice
pub fn show_menu() -> Result<MenuChoice, String> {
    // Make sure terminal is in normal mode first
    let _ = disable_raw_mode();
    
    let mut stdout = io::stdout();
    
    // Clear screen
    execute!(stdout, Clear(ClearType::All), MoveTo(0, 0)).map_err(|e| e.to_string())?;
    
    // Print menu
    println!();
    println!("  ╔═══════════════════════════════════════════════════╗");
    println!("  ║                                                   ║");
    println!("  ║         📡 CSI-TUI - ESP32 Tool                   ║");
    println!("  ║                                                   ║");
    println!("  ╠═══════════════════════════════════════════════════╣");
    println!("  ║                                                   ║");
    println!("  ║   [1] 🔧 Set ESP    - Configure & Terminal        ║");
    println!("  ║                                                   ║");
    println!("  ║   [2] 📊 View CSI   - View CSI Output             ║");
    println!("  ║                                                   ║");
    println!("  ║   [Q] 🚪 Quit                                     ║");
    println!("  ║                                                   ║");
    println!("  ╚═══════════════════════════════════════════════════╝");
    println!();
    
    // Show available ports
    print_available_ports();
    
    println!();
    println!("  Press 1, 2, or Q:");
    stdout.flush().map_err(|e| e.to_string())?;
    
    // Enable raw mode for key detection
    enable_raw_mode().map_err(|e| e.to_string())?;
    
    // Clear any pending events
    while event::poll(Duration::from_millis(100)).unwrap_or(false) {
        let _ = event::read();
    }
    
    // Wait for valid key
    let choice = loop {
        if event::poll(Duration::from_millis(100)).map_err(|e| e.to_string())? {
            if let Ok(Event::Key(key)) = event::read() {
                // Only handle Press events (not Release)
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                match key.code {
                    KeyCode::Char('1') => break 1,
                    KeyCode::Char('2') => break 2,
                    KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => break 0,
                    _ => continue,
                }
            }
        }
    };
    
    // Back to normal mode
    disable_raw_mode().map_err(|e| e.to_string())?;
    
    match choice {
        1 => {
            let (port, baud) = get_port_settings()?;
            Ok(MenuChoice::SetEsp { port, baud })
        }
        2 => Ok(MenuChoice::ViewCsiOutput),
        _ => Ok(MenuChoice::Quit),
    }
}

/// Get port settings from user
fn get_port_settings() -> Result<(String, u32), String> {
    let mut stdout = io::stdout();
    
    println!();
    println!("  ─────────────────────────────────────────────────────");
    println!("  🔌 Serial Port Configuration");
    println!("  ─────────────────────────────────────────────────────");
    
    // Show available ports
    print_available_ports();
    
    // Get port name
    println!();
    print!("  Enter port name (e.g., COM3): ");
    stdout.flush().map_err(|e| e.to_string())?;
    
    let mut port = String::new();
    io::stdin().read_line(&mut port).map_err(|e| e.to_string())?;
    let port = port.trim().to_string();
    
    if port.is_empty() {
        return Err("Port name cannot be empty".to_string());
    }
    
    // Get baud rate
    println!();
    println!("  Common baud rates: 9600, 115200, 460800, 921600");
    print!("  Enter baud rate [115200]: ");
    stdout.flush().map_err(|e| e.to_string())?;
    
    let mut baud_str = String::new();
    io::stdin().read_line(&mut baud_str).map_err(|e| e.to_string())?;
    let baud_str = baud_str.trim();
    
    let baud: u32 = if baud_str.is_empty() {
        115200
    } else {
        baud_str.parse().map_err(|_| "Invalid baud rate")?
    };
    
    println!();
    println!("  ✅ Connecting to {} @ {} baud...", port, baud);
    println!();
    
    Ok((port, baud))
}

/// Print available serial ports
fn print_available_ports() {
    print!("  📋 Available ports: ");
    
    match serialport::available_ports() {
        Ok(ports) if !ports.is_empty() => {
            let port_names: Vec<String> = ports.iter().map(|p| p.port_name.clone()).collect();
            println!("{}", port_names.join(", "));
        }
        _ => {
            println!("(none detected)");
        }
    }
}
