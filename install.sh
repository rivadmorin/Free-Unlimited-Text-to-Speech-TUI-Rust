#!/bin/bash

# --- Color Definitions ---
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# --- Headers ---
echo -e "${BLUE}==============================================${NC}"
echo -e "${BLUE}      Speech-TUI Interactive Installer        ${NC}"
echo -e "${BLUE}==============================================${NC}"

# Ensure we are in the project root
cd "$(dirname "$0")"

# --- Helper Functions ---
confirm() {
    read -r -p "${1:-Are you sure? [y/N]} " response
    case "$response" in
        [yY][eE][sS]|[yY])
            true
            ;;
        *)
            false
            ;;
    esac
}

# --- 1. Privilege & User Handling ---
if [ "$EUID" -eq 0 ]; then
    echo -e "${RED}Error: Please do not run this script as root/sudo directly.${NC}"
    echo -e "The script will prompt for sudo when needed."
    exit 1
fi

REAL_USER=${SUDO_USER:-$USER}
HOME_DIR=$(getent passwd "$REAL_USER" | cut -d: -f6)

# --- 2. System Dependency Check & Installation ---
echo -e "\n${BLUE}[1/6] Installing System Dependencies...${NC}"
DEPENDENCIES=(
    "build-essential"
    "libxcb1-dev"
    "wl-clipboard"
    "alsa-utils"
    "libasound2-dev"
    "libpulse-dev"
    "git"
    "nodejs"
    "npm"
    "curl"
)

echo -e "Required packages: ${DEPENDENCIES[*]}"
if confirm "Do you want to install these dependencies via apt? [y/N]"; then
    sudo apt update || { echo -e "${RED}apt update failed.${NC}"; exit 1; }
    sudo apt install -y "${DEPENDENCIES[@]}" || { echo -e "${RED}Failed to install dependencies.${NC}"; exit 1; }
    echo -e "${GREEN}Dependencies installed successfully.${NC}"
else
    echo -e "${YELLOW}Skipping dependency installation. Ensure they are present manually.${NC}"
fi

# --- 3. Rust Environment Setup ---
echo -e "\n${BLUE}[2/6] Checking Rust Environment...${NC}"
if ! command -v cargo &> /dev/null; then
    echo -e "${YELLOW}Rust/Cargo is missing.${NC}"
    if confirm "Do you want to install Rust via rustup now? [y/N]"; then
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "$HOME_DIR/.cargo/env"
        echo -e "${GREEN}Rust installed successfully.${NC}"
    else
        echo -e "${RED}Rust is required to build Speech-TUI. Exiting.${NC}"
        exit 1
    fi
else
    echo -e "${GREEN}Rust is already installed.$(cargo --version)${NC}"
fi

# cargo-deb check
if ! cargo deb --version &> /dev/null; then
    echo -e "${BLUE}Installing cargo-deb...${NC}"
    cargo install cargo-deb || { echo -e "${RED}Failed to install cargo-deb.${NC}"; exit 1; }
    echo -e "${GREEN}cargo-deb installed.${NC}"
fi

# --- 4. Camofox Browser Setup ---
echo -e "\n${BLUE}[3/6] Setting up Camofox Browser...${NC}"
CAMOFOX_DIR="$HOME_DIR/.local/share/camofox"
if [ ! -d "$CAMOFOX_DIR" ]; then
    echo -e "Cloning Camofox Browser to $CAMOFOX_DIR..."
    mkdir -p "$(dirname "$CAMOFOX_DIR")"
    git clone https://github.com/jo-inc/camofox-browser "$CAMOFOX_DIR"
    cd "$CAMOFOX_DIR" || exit
    npm install
    cd - || exit
else
    echo -e "${GREEN}Camofox Browser already exists at $CAMOFOX_DIR.${NC}"
fi

# Configure Firefox profile for microphone
PROFILE_DIR="$CAMOFOX_DIR/profile"
PREFS_FILE="$PROFILE_DIR/prefs.js"
echo -e "Configuring microphone permissions in $PREFS_FILE..."
mkdir -p "$PROFILE_DIR"
touch "$PREFS_FILE"
if ! grep -q "media.navigator.permission.disabled" "$PREFS_FILE"; then
    echo 'user_pref("media.navigator.permission.disabled", true);' >> "$PREFS_FILE"
    echo -e "${GREEN}Microphone permissions auto-granted.${NC}"
else
    echo -e "${YELLOW}Microphone permissions already configured.${NC}"
fi

# --- 5. Rust TUI Build & Packaging ---
echo -e "\n${BLUE}[4/6] Building and Installing Speech-TUI...${NC}"
cargo deb || { echo -e "${RED}cargo deb failed.${NC}"; exit 1; }
if [ $? -eq 0 ]; then
    # Pick the most recently created .deb package
    DEB_FILE=$(ls -t target/debian/speech-tui_*.deb | head -n 1)
    if [ -z "$DEB_FILE" ]; then
        echo -e "${RED}Error: .deb file not found in target/debian/.${NC}"
        exit 1
    fi
    echo -e "${GREEN}Build successful. Installing $DEB_FILE...${NC}"
    sudo apt install -y "./$DEB_FILE"
else
    echo -e "${RED}Failed to build .deb package.${NC}"
    exit 1
fi

# --- 6. Systemd Automation ---
echo -e "\n${BLUE}[5/6] Systemd Service Setup...${NC}"
if confirm "Do you want to register camofox-browser as a background systemd service? [y/N]"; then
    SERVICE_FILE="/etc/systemd/system/camofox.service"
    echo "Creating $SERVICE_FILE..."
    sudo bash -c "cat > $SERVICE_FILE" <<EOF
[Unit]
Description=Camofox Browser Service
After=network.target

[Service]
Type=simple
User=$REAL_USER
WorkingDirectory=$CAMOFOX_DIR
ExecStart=/usr/bin/npm start -- --port 8080
Restart=always
Environment=NODE_ENV=production

[Install]
WantedBy=multi-user.target
EOF

    sudo systemctl daemon-reload
    sudo systemctl enable camofox.service
    sudo systemctl start camofox.service
    echo -e "${GREEN}camofox.service is now active and enabled on boot.${NC}"
else
    echo -e "${YELLOW}Skipping systemd service setup.${NC}"
fi

# --- 7. Environment Setup ---
echo -e "\n${BLUE}[6/6] Finalizing Environment...${NC}"
TRANSCRIPTS_DIR="$HOME_DIR/Documents/Transcripts"
if [ ! -d "$TRANSCRIPTS_DIR" ]; then
    mkdir -p "$TRANSCRIPTS_DIR"
    chown "$REAL_USER:$REAL_USER" "$TRANSCRIPTS_DIR"
    echo -e "Created transcript directory at $TRANSCRIPTS_DIR"
fi

# --- Success Banner ---
echo -e "\n${GREEN}==============================================${NC}"
echo -e "${GREEN}      Installation Complete Successfully!     ${NC}"
echo -e "${GREEN}==============================================${NC}"
echo -e "You can now launch the application by typing:"
echo -e "${BLUE}  speech-tui${NC}"
echo -e "\nMake sure Camofox is running at http://localhost:8080"
echo -e "Transcripts will be saved in: $TRANSCRIPTS_DIR"
echo -e "${GREEN}==============================================${NC}"
