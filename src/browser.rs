use anyhow::{Result, Context};
use std::time::Duration;
use reqwest::Client;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct BrowserClient {
    client: Client,
    base_url: String,
    tab_id: Arc<Mutex<Option<String>>>,
}

impl BrowserClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .unwrap(),
            base_url: base_url.to_string(),
            tab_id: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn ensure_session(&self) -> Result<()> {
        {
            let tid = self.tab_id.lock().unwrap();
            if tid.is_some() {
                return Ok(());
            }
        }

        let res = self.client.post(format!("{}/tabs", self.base_url))
            .json(&serde_json::json!({
                "url": "https://speechnotes.co/dictate/",
                "userId": "speech-tui-user",
                "sessionKey": "default"
            }))
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        let tab_id = res["tabId"].as_str().context("Failed to get tabId")?.to_string();
        {
            let mut tid = self.tab_id.lock().unwrap();
            *tid = Some(tab_id);
        }

        // Wait for page load
        tokio::time::sleep(Duration::from_secs(5)).await;

        Ok(())
    }

    pub async fn start_dictation(&self) -> Result<()> {
        let tab_id = {
            let tab_id_guard = self.tab_id.lock().unwrap();
            tab_id_guard.as_ref().context("No active tab")?.clone()
        };

        // Find the mic button. In speechnotes.co it's usually the big orange button.
        // We'll use the snapshot to find it or try a direct selector if possible.
        // Camofox /act endpoint can click by text or ref.

        self.client.post(format!("{}/tabs/{}/act", self.base_url, tab_id))
            .json(&serde_json::json!({
                "action": "click",
                "text": "Start"
            }))
            .send()
            .await?;

        Ok(())
    }

    pub async fn stop_dictation(&self) -> Result<()> {
        let tab_id = {
            let tab_id_guard = self.tab_id.lock().unwrap();
            tab_id_guard.as_ref().context("No active tab")?.clone()
        };

        self.client.post(format!("{}/tabs/{}/act", self.base_url, tab_id))
            .json(&serde_json::json!({
                "action": "click",
                "text": "Stop"
            }))
            .send()
            .await?;

        Ok(())
    }

    pub async fn get_text(&self) -> Result<String> {
        let tab_id = {
            let tab_id_guard = self.tab_id.lock().unwrap();
            tab_id_guard.as_ref().context("No active tab")?.clone()
        };

        let res = self.client.get(format!("{}/tabs/{}/snapshot", self.base_url, tab_id))
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        // Handle popups before extracting text
        self.dismiss_popups(&tab_id, &res).await?;

        // Extract text from the snapshot.
        // speechnotes.co editor is typically identified in the accessibility tree.
        // We look for a generic container with "editor" or a large amount of text.
        // Based on camofox snapshots, we look for elements with role 'generic' or 'textbox'

        if let Some(snapshot) = res["snapshot"].as_str() {
             // In a real scenario, we'd parse the snapshot string (which is often a tree)
             // For this implementation, we'll use a heuristic to find the main text area.
             // Usually, it's the largest text block.
             return Ok(self.parse_dictated_text(snapshot));
        }

        Ok("".to_string())
    }

    async fn dismiss_popups(&self, tab_id: &str, snapshot_res: &serde_json::Value) -> Result<()> {
        let snapshot = snapshot_res["snapshot"].as_str().unwrap_or("");

        // Common popup close button patterns on speechnotes.co
        let popup_markers = ["Close", "Dismiss", "Maybe later", "Don't show again", "×"];

        for marker in popup_markers {
            if snapshot.contains(marker) {
                let _ = self.client.post(format!("{}/tabs/{}/act", self.base_url, tab_id))
                    .json(&serde_json::json!({
                        "action": "click",
                        "text": marker
                    }))
                    .send()
                    .await;
            }
        }
        Ok(())
    }

    fn parse_dictated_text(&self, snapshot: &str) -> String {
        // Simple heuristic: extract text between markers or find the longest paragraph
        // In a real Camofox snapshot, text is often enclosed in brackets or quotes
        // For this expert implementation, we assume the snapshot has a structured format
        // like "[e1] Some text" or similar.

        let mut lines: Vec<&str> = snapshot.lines().collect();
        lines.sort_by_key(|l| l.len());

        // Return the longest line that doesn't look like a menu item
        lines.iter()
            .rev()
            .find(|l| l.len() > 20 && !l.contains("Menu") && !l.contains("Home"))
            .map(|l| l.trim().to_string())
            .unwrap_or_else(|| "".to_string())
    }

    pub async fn check_connection(&self) -> bool {
        self.client.get(format!("{}/health", self.base_url))
            .send()
            .await
            .is_ok()
    }
}
