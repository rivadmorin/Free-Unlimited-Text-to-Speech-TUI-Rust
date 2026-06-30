#!/bin/bash

# --- Color Definitions ---
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}==============================================${NC}"
echo -e "${BLUE}      Speech-TUI Quick Launcher               ${NC}"
echo -e "${BLUE}==============================================${NC}"

# Ensure we are in the project root
cd "$(dirname "$0")"

# 1. Check if Camofox Browser is running
if ! systemctl is-active --quiet camofox.service; then
    echo -e "${YELLOW}Warning: camofox.service is not running.${NC}"
    echo -e "Headless browser automation might fail."
    echo -e "Try: sudo systemctl start camofox.service"
    echo ""
fi

# 2. Find and execute binary
if command -v speech-tui &> /dev/null; then
    echo -e "${GREEN}Launching system-wide speech-tui...${NC}"
    speech-tui
elif [ -f "./target/release/speech-tui" ]; then
    echo -e "${GREEN}Launching local release build...${NC}"
    ./target/release/speech-tui
elif [ -f "./target/debug/speech-tui" ]; then
    echo -e "${YELLOW}Release build not found. Launching local debug build...${NC}"
    ./target/debug/speech-tui
else
    echo -e "${YELLOW}Binary not found. Attempting to build and run...${NC}"
    if command -v cargo &> /dev/null; then
        cargo run --release
    else
        echo -e "${RED}Error: cargo not found. Please run install.sh first.${NC}"
        exit 1
    fi
fi
