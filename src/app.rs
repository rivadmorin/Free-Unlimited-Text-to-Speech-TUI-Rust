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
    pub selected_content: String,
    pub transcripts_list: Vec<PathBuf>,
    pub selected_index: usize,
    pub is_recording: bool,
    pub is_connected: bool,
    pub audio_active: bool,
    pub last_error: Option<String>,
    pub amplitude: f32,
    pub status_message: String,
    pub focus: Focus,
}

impl App {
    pub fn new() -> Self {
        let mut app = Self {
            transcript: String::new(),
            selected_content: String::new(),
            transcripts_list: Vec::new(),
            selected_index: 0,
            is_recording: false,
            is_connected: false,
            audio_active: false,
            last_error: None,
            amplitude: 0.0,
            status_message: "Press Space to start".to_string(),
            focus: Focus::Main,
        };
        let _ = app.refresh_transcripts();
        app
    }

    pub fn refresh_transcripts(&mut self) -> Result<()> {
        let user_dirs = UserDirs::new().context("Could not find user directories")?;
        let doc_dir = user_dirs.document_dir().context("Could not find Documents directory")?;
        let path = doc_dir.join("Transcripts");

        if !path.exists() {
            fs::create_dir_all(&path).context("Failed to create transcripts directory")?;
        }

        let entries = fs::read_dir(path).context("Failed to read transcripts directory")?;
        self.transcripts_list = entries
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|ext| ext == "md" || ext == "txt"))
            .collect();
        self.transcripts_list.sort_by(|a, b| b.cmp(a)); // Newest first
        self.load_selected_content();

        Ok(())
    }

    pub fn save_transcript(&mut self) -> Result<()> {
        if self.transcript.is_empty() {
            return Ok(());
        }
        let res = (|| -> Result<String> {
            let user_dirs = UserDirs::new().context("Could not find user directories")?;
            let doc_dir = user_dirs.document_dir().context("Could not find Documents directory")?;
            let path = doc_dir.join("Transcripts");

            let filename = format!("transcript_{}.md", Local::now().format("%Y%m%d_%H%M%S"));
            let full_path = path.join(filename);
            fs::write(&full_path, &self.transcript).context("Failed to write transcript file")?;
            Ok(full_path.display().to_string())
        })();

        match res {
            Ok(path_str) => {
                self.status_message = format!("Saved to {}", path_str);
                let _ = self.refresh_transcripts();
                self.last_error = None;
                Ok(())
            }
            Err(e) => {
                self.last_error = Some(format!("Save error: {}", e));
                Err(e)
            }
        }
    }

    pub fn load_selected_content(&mut self) {
        if let Some(path) = self.transcripts_list.get(self.selected_index) {
            if let Ok(content) = fs::read_to_string(path) {
                self.selected_content = content;
            }
        }
    }

    pub fn next_transcript(&mut self) {
        if !self.transcripts_list.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.transcripts_list.len();
            self.load_selected_content();
        }
    }

    pub fn previous_transcript(&mut self) {
        if !self.transcripts_list.is_empty() {
            if self.selected_index > 0 {
                self.selected_index -= 1;
            } else {
                self.selected_index = self.transcripts_list.len() - 1;
            }
            self.load_selected_content();
        }
    }

    pub fn copy_to_clipboard(&mut self) {
        let text_to_copy = match self.focus {
            Focus::Main => self.transcript.clone(),
            Focus::Sidebar => self.selected_content.clone(),
        };

        if text_to_copy.is_empty() {
            self.status_message = "Nothing to copy".to_string();
            return;
        }

        match arboard::Clipboard::new() {
            Ok(mut clipboard) => {
                if let Err(e) = clipboard.set_text(text_to_copy) {
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
