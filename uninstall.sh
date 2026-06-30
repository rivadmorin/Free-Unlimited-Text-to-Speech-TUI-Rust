#!/bin/bash

# --- Color Definitions ---
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# --- Headers ---
echo -e "${RED}==============================================${NC}"
echo -e "${RED}      Speech-TUI Uninstaller                ${NC}"
echo -e "${RED}==============================================${NC}"

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

# --- 1. Destruction Warning ---
echo -e "${YELLOW}WARNING: This will remove Speech-TUI and its components from your system.${NC}"
if ! confirm "Do you want to proceed with uninstallation? [y/N]"; then
    echo "Uninstallation cancelled."
    exit 0
fi

# --- 2. Service Deactivation ---
echo -e "\n${BLUE}[1/4] Deactivating Systemd Service...${NC}"
SERVICE_NAME="camofox.service"
SERVICE_FILE="/etc/systemd/system/$SERVICE_NAME"
if [ -f "$SERVICE_FILE" ] || systemctl list-unit-files | grep -q "$SERVICE_NAME"; then
    echo -e "Stopping and disabling $SERVICE_NAME..."
    sudo systemctl stop "$SERVICE_NAME" 2>/dev/null || true
    sudo systemctl disable "$SERVICE_NAME" 2>/dev/null || true
    if [ -f "$SERVICE_FILE" ]; then
        sudo rm "$SERVICE_FILE"
    fi
    sudo systemctl daemon-reload
    echo -e "${GREEN}Service removed.${NC}"
else
    echo -e "${YELLOW}Systemd service $SERVICE_NAME not found, skipping.${NC}"
fi

# --- 3. Application Removal ---
echo -e "\n${BLUE}[2/4] Removing Speech-TUI Application...${NC}"
PACKAGE_FOUND=false
if dpkg -l | grep -q "speech-tui"; then
    echo -e "Uninstalling speech-tui package..."
    sudo apt remove -y speech-tui
    PACKAGE_FOUND=true
fi

if [ -f "/usr/bin/speech-tui" ]; then
    echo -e "Removing stray binary at /usr/bin/speech-tui..."
    sudo rm "/usr/bin/speech-tui"
    PACKAGE_FOUND=true
fi

if [ "$PACKAGE_FOUND" = true ]; then
    echo -e "${GREEN}Application uninstalled.${NC}"
else
    echo -e "${YELLOW}Speech-TUI installation not found, skipping.${NC}"
fi

# --- 4. Camofox Browser Data Removal ---
echo -e "\n${BLUE}[3/4] Cleaning up Camofox Browser...${NC}"
REAL_USER=${SUDO_USER:-$USER}
HOME_DIR=$(getent passwd "$REAL_USER" | cut -d: -f6)
CAMOFOX_DIR="$HOME_DIR/.local/share/camofox"

if [ -d "$CAMOFOX_DIR" ]; then
    if confirm "Do you want to delete the Camofox Browser directory ($CAMOFOX_DIR)? [y/N]"; then
        rm -rf "$CAMOFOX_DIR"
        echo -e "${GREEN}Camofox Browser directory removed.${NC}"
    else
        echo -e "${YELLOW}Preserving Camofox Browser directory.${NC}"
    fi
fi

# --- 5. Interactive Data Purging ---
echo -e "\n${BLUE}[4/4] Purging Application Data...${NC}"
TRANSCRIPTS_DIR="$HOME_DIR/Documents/Transcripts"
if [ -d "$TRANSCRIPTS_DIR" ]; then
    echo -e "${RED}CRITICAL: Do you want to delete all saved transcripts in $TRANSCRIPTS_DIR?${NC}"
    if confirm "THIS ACTION CANNOT BE UNDONE. Delete all transcripts? [y/N]"; then
        rm -rf "$TRANSCRIPTS_DIR"
        echo -e "${GREEN}Transcripts directory wiped.${NC}"
    else
        echo -e "${YELLOW}Preserving transcripts at $TRANSCRIPTS_DIR.${NC}"
    fi
fi

# --- Cleanup Confirmation ---
echo -e "\n${GREEN}==============================================${NC}"
echo -e "${GREEN}      Uninstallation Finished Successfully!   ${NC}"
echo -e "${GREEN}==============================================${NC}"
echo -e "The system is now clean."
echo -e "${GREEN}==============================================${NC}"
