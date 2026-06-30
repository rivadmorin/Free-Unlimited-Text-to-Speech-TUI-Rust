use std::path::PathBuf;
use std::fs;
use anyhow::{Result, Context};
use directories::UserDirs;
use chrono::Local;

pub enum Focus {
    Sidebar,
    Main,
}

pub struct App {
    pub transcript: String,
    pub transcripts_list: Vec<PathBuf>,
    pub selected_index: usize,
    pub is_recording: bool,
    pub is_connected: bool,
    pub amplitude: f32,
    pub status_message: String,
    pub focus: Focus,
}

impl App {
    pub fn new() -> Self {
        let mut app = Self {
            transcript: String::new(),
            transcripts_list: Vec::new(),
            selected_index: 0,
            is_recording: false,
            is_connected: false,
            amplitude: 0.0,
            status_message: "Press Space to start".to_string(),
            focus: Focus::Main,
        };
        app.refresh_transcripts();
        app
    }

    pub fn refresh_transcripts(&mut self) {
        if let Some(user_dirs) = UserDirs::new() {
            if let Some(doc_dir) = user_dirs.document_dir() {
                let path = doc_dir.join("Transcripts");
                if !path.exists() {
                    let _ = fs::create_dir_all(&path);
                }
                if let Ok(entries) = fs::read_dir(path) {
                    self.transcripts_list = entries
                        .filter_map(|e| e.ok())
                        .map(|e| e.path())
                        .filter(|p| p.extension().map_or(false, |ext| ext == "md" || ext == "txt"))
                        .collect();
                    self.transcripts_list.sort_by(|a, b| b.cmp(a)); // Newest first
                }
            }
        }
    }

    pub fn save_transcript(&mut self) -> Result<()> {
        if self.transcript.is_empty() {
            return Ok(());
        }
        let user_dirs = UserDirs::new().context("Could not find user directories")?;
        let doc_dir = user_dirs.document_dir().context("Could not find Documents directory")?;
        let path = doc_dir.join("Transcripts");

        let filename = format!("transcript_{}.md", Local::now().format("%Y%m%d_%H%M%S"));
        let full_path = path.join(filename);
        fs::write(&full_path, &self.transcript)?;
        self.status_message = format!("Saved to {}", full_path.display());
        self.refresh_transcripts();
        Ok(())
    }

    pub fn next_transcript(&mut self) {
        if !self.transcripts_list.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.transcripts_list.len();
        }
    }

    pub fn previous_transcript(&mut self) {
        if !self.transcripts_list.is_empty() {
            if self.selected_index > 0 {
                self.selected_index -= 1;
            } else {
                self.selected_index = self.transcripts_list.len() - 1;
            }
        }
    }

    pub fn copy_to_clipboard(&mut self) {
        match arboard::Clipboard::new() {
            Ok(mut clipboard) => {
                if let Err(e) = clipboard.set_text(self.transcript.clone()) {
                    self.status_message = format!("Clipboard error: {}", e);
                } else {
                    self.status_message = "Copied to clipboard".to_string();
                }
            }
            Err(e) => {
                self.status_message = format!("Failed to access clipboard: {}", e);
            }
        }
    }
}
