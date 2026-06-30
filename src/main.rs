mod app;
mod ui;
mod browser;
mod audio;

use std::{error::Error, io, time::Duration};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use tokio::time::interval;
use crate::app::{App, Focus};
use crate::browser::BrowserClient;
use crate::audio::AudioMonitor;
use std::sync::{Arc, Mutex};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Setup logging
    env_logger::init();

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let app = Arc::new(Mutex::new(App::new()));
    let browser = BrowserClient::new("http://localhost:8080");
    let audio_data = AudioMonitor::new().ok();
    let (audio, _stream) = if let Some((monitor, stream)) = audio_data {
        (Some(monitor), Some(stream))
    } else {
        (None, None)
    };

    // Main event loop
    let mut poll_interval = interval(Duration::from_millis(500));

    // Background task for browser polling and status
    let app_task = Arc::clone(&app);
    let browser_bg = browser.clone();
    tokio::spawn(async move {
        loop {
            poll_interval.tick().await;

            let is_connected = browser_bg.check_connection().await;
            let is_recording = {
                let a = app_task.lock().unwrap();
                a.is_recording
            };

            // Only poll more frequently if recording
            if !is_recording {
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
            {
                let mut a = app_task.lock().unwrap();
                a.is_connected = is_connected;
            }

            if is_connected {
                if let Err(_) = browser_bg.ensure_session().await {
                    let mut a = app_task.lock().unwrap();
                    a.status_message = "Failed to start browser session".to_string();
                    continue;
                }

                let is_recording = {
                    let a = app_task.lock().unwrap();
                    a.is_recording
                };

                if is_recording {
                    if let Ok(text) = browser_bg.get_text().await {
                        let mut a = app_task.lock().unwrap();
                        if !text.is_empty() && text != a.transcript {
                             a.transcript = text;
                        }
                    }
                }
            }
        }
    });

    // Background task for VU meter
    let app_vu = Arc::clone(&app);
    if let Some(monitor) = audio {
        let monitor = Arc::new(monitor);
        tokio::spawn(async move {
            let mut vu_interval = interval(Duration::from_millis(100));
            loop {
                vu_interval.tick().await;
                let mut a = app_vu.lock().unwrap();
                a.amplitude = monitor.get_amplitude();
            }
        });
    }

    loop {
        let app_ui = app.lock().unwrap();
        terminal.draw(|f| ui::render(f, &app_ui))?;
        drop(app_ui);

        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                let mut a = app.lock().unwrap();
                match key.code {
                    KeyCode::Char('q') | KeyCode::Char('Q') => break,
                    KeyCode::Char(' ') => {
                        a.is_recording = !a.is_recording;
                        let is_recording = a.is_recording;
                        if is_recording {
                            a.status_message = "Dictation started".to_string();
                        } else {
                            a.status_message = "Dictation paused".to_string();
                        }

                        let browser_task = browser.clone();
                        tokio::spawn(async move {
                            if is_recording {
                                let _ = browser_task.start_dictation().await;
                            } else {
                                let _ = browser_task.stop_dictation().await;
                            }
                        });
                    }
                    KeyCode::Char('s') | KeyCode::Char('S') => {
                        let _ = a.save_transcript();
                    }
                    KeyCode::Char('y') | KeyCode::Char('Y') => {
                        a.copy_to_clipboard();
                    }
                    KeyCode::Char('j') | KeyCode::Down => {
                        if matches!(a.focus, Focus::Sidebar) {
                            a.next_transcript();
                        }
                    }
                    KeyCode::Char('k') | KeyCode::Up => {
                        if matches!(a.focus, Focus::Sidebar) {
                            a.previous_transcript();
                        }
                    }
                    KeyCode::Char('h') | KeyCode::Left => {
                        a.focus = Focus::Sidebar;
                    }
                    KeyCode::Char('l') | KeyCode::Right => {
                        a.focus = Focus::Main;
                    }
                    _ => {}
                }
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
