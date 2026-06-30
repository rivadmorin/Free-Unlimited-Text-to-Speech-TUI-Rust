# Speech-TUI 🎤

**Speech-TUI** is a lightweight, highly efficient Terminal User Interface (TUI) application designed for unlimited free Speech-to-Text dictation on Linux Debian. It leverages the power of `camofox-browser` to automate `speechnotes.co` without the overhead of local AI models.

## 🚀 Features

- **Free & Unlimited:** Uses `speechnotes.co` via headless automation.
- **Privacy Focused:** No local AI models; data is processed via the browser.
- **TUI Powered:** Built with `ratatui` and `tokio` for a snappy, non-blocking experience.
- **VU Meter:** Real-time visual feedback of your audio input.
- **Sidebar Navigation:** Easily browse and manage your local transcripts.
- **Clipboard Integration:** Copy transcripts directly to your system clipboard.
- **Debian Optimized:** Native `.deb` packaging support.

## ⌨️ Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `Space` | Start / Pause Dictation |
| `S` | Save Transcript |
| `Y` | Copy to Clipboard |
| `j`/`k` or `↑`/`↓` | Navigate Sidebar |
| `h`/`l` or `←`/`→` | Switch Focus (Sidebar/Main) |
| `Q` | Quit Safely |

## 🛠 Prerequisites

Ensure your Debian system has the following installed:

### System Dependencies
```bash
sudo apt update
sudo apt install -y libasound2-dev libpulse-dev libxcb1-dev wl-clipboard
```

### Camofox Browser
The application requires `camofox-browser` to be running as a background service.
1. Install `camofox-browser` (refer to its official documentation).
2. Ensure the REST API is accessible at `http://localhost:8080`.
3. Configure the browser profile to automatically grant microphone permissions (`media.navigator.permission.disabled = true`).

## 📦 Installation

### Building from Source
```bash
cargo build --release
```

### Packaging as .deb
```bash
cargo install cargo-deb
cargo deb
sudo dpkg -i target/debian/speech-tui_*.deb
```

## 📂 Data Storage
Transcripts are saved by default in `~/Documents/Transcripts/` in `.md` or `.txt` format.

## 🧹 Uninstallation

To completely remove the application and its configuration:

1. **Remove the package:**
   ```bash
   sudo apt purge speech-tui
   ```

2. **Clean up configuration and data (optional):**
   ```bash
   rm -rf ~/Documents/Transcripts/
   ```

## 🏗 Architecture
- **`main.rs`**: Entry point and TUI event loop.
- **`app.rs`**: Application state management.
- **`ui.rs`**: UI rendering logic.
- **`browser.rs`**: Camofox-browser REST API automation.
- **`audio.rs`**: Audio capture and VU meter computation.

## ⚖️ License
MIT License. See `LICENSE` for details.
