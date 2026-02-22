#!/bin/bash
#
# MASTerm Uninstallation Script
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/theWayCrafter/MASTerm/main/uninstall.sh | bash
#

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color
BOLD='\033[1m'

# Paths
INSTALL_DIR="$HOME/.masterm"
CONFIG_FILE="$HOME/.masterm.toml"
BINARY="$HOME/.cargo/bin/masterm"

# Functions
info() {
    echo -e "${BLUE}ℹ${NC} $1"
}

success() {
    echo -e "${GREEN}✓${NC} $1"
}

warn() {
    echo -e "${YELLOW}⚠${NC} $1"
}

cleanup_shell_config() {
    local rc_file="$1"
    local shell_name="$2"

    if [ -f "$rc_file" ]; then
        info "Cleaning up $shell_name config ($rc_file)..."
        
        # Create backup
        cp "$rc_file" "$rc_file.bak"
        
        # Check if file needs cleaning
        if grep -q "masterm init" "$rc_file" || grep -q "# MASTerm" "$rc_file"; then
            # Remove lines containing 'masterm init' or '# MASTerm'
            # We use a temporary file to store the filtered content
            grep -v "masterm init" "$rc_file" | grep -v "# MASTerm" > "$rc_file.tmp"
            mv "$rc_file.tmp" "$rc_file"
            success "Removed MASTerm from $rc_file (Backup saved to $rc_file.bak)"
        else
            info "No MASTerm configuration found in $rc_file"
        fi
    fi
}

# Main uninstallation
echo ""
echo -e "${RED}${BOLD}MASTerm Uninstallation${NC}"
echo -e "${RED}════════════════════════════════════════════════════════════${NC}"
echo ""

echo -e "This will remove:"
echo -e "  - Binary: $BINARY"
echo -e "  - Config: $CONFIG_FILE"
echo -e "  - Data:   $INSTALL_DIR"
echo ""
echo -n "Are you sure you want to continue? [y/N] "
read -r response
if [[ ! "$response" =~ ^([yY][eE][sS]|[yY])$ ]]; then
    echo "Aborted."
    exit 0
fi
echo ""

# 1. Remove Binary
if [ -f "$BINARY" ]; then
    rm "$BINARY"
    success "Removed binary"
else
    warn "Binary not found at $BINARY"
fi

# 2. Remove Config
if [ -f "$CONFIG_FILE" ]; then
    rm "$CONFIG_FILE"
    success "Removed configuration file"
fi

# 3. Remove Data Directory
if [ -d "$INSTALL_DIR" ]; then
    rm -rf "$INSTALL_DIR"
    success "Removed data directory"
fi

# 4. Clean Shell Configs
cleanup_shell_config "$HOME/.zshrc" "Zsh"
cleanup_shell_config "$HOME/.bashrc" "Bash"
cleanup_shell_config "$HOME/.config/fish/config.fish" "Fish"

echo ""
echo -e "${GREEN}${BOLD}Uninstallation complete.${NC}"
echo -e "Please restart your terminal to ensure all changes take effect."
echo ""
