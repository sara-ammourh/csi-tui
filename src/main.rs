// main.rs - Application Entry Point
mod app;
mod csv_loader;
mod csv_logger;
mod detectors;
mod esp_terminal;
mod menu;
mod parser;
mod serial_reader;
mod state;
mod ui;

use std::io;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use crate::app::App;
use crate::esp_terminal::run_esp_terminal;
use crate::menu::{show_menu, MenuChoice};
use crate::state::create_shared_state;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    loop {
        // Small delay to ensure terminal is ready
        std::thread::sleep(std::time::Duration::from_millis(100));
        
        let choice = match show_menu() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Error: {}", e);
                continue;
            }
        };
        
        match choice {
            MenuChoice::SetEsp { port, baud } => {
                if let Err(e) = run_esp_terminal(&port, baud) {
                    eprintln!("Error: {}", e);
                    println!("Press Enter to continue...");
                    let mut input = String::new();
                    let _ = io::stdin().read_line(&mut input);
                }
            }
            MenuChoice::ViewCsiOutput => {
                if let Err(e) = run_csi_viewer() {
                    eprintln!("Error: {}", e);
                }
            }
            MenuChoice::Quit => {
                println!("Goodbye!");
                break;
            }
        }
    }
    Ok(())
}

fn run_csi_viewer() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let state = create_shared_state();
    let mut app = App::new(state.clone());
    let result = run_app_loop(&mut terminal, &mut app, &state);

    // Cleanup - important to do in correct order!
    // تنظيف - مهم بالترتيب الصحيح!
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;
    
    // Clear any pending events
    // تنظيف الأحداث المعلقة
    while crossterm::event::poll(std::time::Duration::from_millis(10))? {
        let _ = crossterm::event::read();
    }
    
    result.map_err(|e| e.into())
}

fn run_app_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    state: &state::SharedState,
) -> Result<(), String> {
    loop {
        {
            let mut state_guard = state.lock().map_err(|e| e.to_string())?;
            if state_guard.playback_mode && state_guard.playback_playing {
                if let Some(frame) = state_guard.advance_playback() {
                    if frame.subcarrier_count() > state_guard.max_sc {
                        state_guard.max_sc = frame.subcarrier_count();
                    }
                    state_guard.frames.push(frame);
                    if state_guard.frames.len() > 100 {
                        state_guard.frames.remove(0);
                    }
                    state_guard.status_message = format!("Playing: {:.1}s / {:.1}s",
                        state_guard.get_current_playback_second(),
                        state_guard.playback_duration_secs
                    );
                }
            }
        }
        app.run_detectors()?;
        terminal.draw(|frame| { ui::render(frame, state); }).map_err(|e| format!("Draw error: {}", e))?;
        if app.handle_events()? { break; }
        {
            let state_guard = state.lock().map_err(|e| e.to_string())?;
            if state_guard.should_quit { break; }
        }
    }
    Ok(())
}
