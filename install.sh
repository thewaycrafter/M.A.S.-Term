#!/bin/bash
#
# MASTerm Installation Script
# https://github.com/singhalmridul/MASTerm
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/singhalmridul/MASTerm/main/install.sh | bash
#
# Or with options:
#   curl -fsSL https://raw.githubusercontent.com/singhalmridul/MASTerm/main/install.sh | bash -s -- --shell zsh
#

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color
BOLD='\033[1m'

# Configuration
REPO="singhalmridul/MASTerm"
INSTALL_DIR="$HOME/.masterm"
CONFIG_FILE="$HOME/.masterm.toml"
CARGO_BIN="$HOME/.cargo/bin"

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

error() {
    echo -e "${RED}✗${NC} $1"
}

header() {
    echo ""
    echo -e "${MAGENTA}${BOLD}$1${NC}"
    echo -e "${MAGENTA}════════════════════════════════════════════════════════════${NC}"
    echo ""
}

# Detect OS
detect_os() {
    case "$(uname -s)" in
        Linux*)     OS="linux";;
        Darwin*)    OS="macos";;
        CYGWIN*|MINGW*|MSYS*) OS="windows";;
        *)          OS="unknown";;
    esac
    echo "$OS"
}

# Detect architecture
detect_arch() {
    case "$(uname -m)" in
        x86_64|amd64)   ARCH="x86_64";;
        arm64|aarch64)  ARCH="aarch64";;
        *)              ARCH="unknown";;
    esac
    echo "$ARCH"
}

# Detect shell
detect_shell() {
    if [ -n "$SHELL" ]; then
        case "$SHELL" in
            */zsh)  echo "zsh";;
            */bash) echo "bash";;
            */fish) echo "fish";;
            *)      echo "unknown";;
        esac
    else
        echo "unknown"
    fi
}

# Check if command exists
has_command() {
    command -v "$1" >/dev/null 2>&1
}

# Install Rust if not present
install_rust() {
    if has_command rustc; then
        success "Rust is already installed"
        return 0
    fi

    info "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
    success "Rust installed successfully"
}

# Build and install MASTerm from source
install_from_source() {
    info "Installing MASTerm from source..."
    
    # Create temp directory
    TEMP_DIR=$(mktemp -d)
    cd "$TEMP_DIR"
    
    # Clone repo
    info "Cloning repository..."
    git clone --depth 1 "https://github.com/$REPO.git" masterm
    cd masterm
    
    # Build
    info "Building MASTerm (this may take a few minutes)..."
    source "$HOME/.cargo/env" 2>/dev/null || true
    cargo build --release
    
    # Install
    cargo install --path crates/masterm-cli
    
    # Cleanup
    cd /
    rm -rf "$TEMP_DIR"
    
    success "MASTerm installed to $CARGO_BIN/masterm"
}

# Create directories
create_directories() {
    info "Creating directories..."
    mkdir -p "$INSTALL_DIR"
    mkdir -p "$INSTALL_DIR/plugins"
    mkdir -p "$INSTALL_DIR/cache"
    mkdir -p "$INSTALL_DIR/shell"
    success "Created $INSTALL_DIR"
}

# Create default config
create_config() {
    if [ -f "$CONFIG_FILE" ]; then
        warn "Config already exists at $CONFIG_FILE"
        return 0
    fi

    local ENABLED_PLUGINS=""
    
    # Default recommended plugins
    local PLUGINS=(
        "git:Git integration"
        "env:Environment detection"
        "prod-guard:Production safety details"
        "node:Node.js context"
        "python:Python context"
        "rust:Rust context"
        "go:Go context"
        "docker:Docker context"
        "kubernetes:Kubernetes context"
    )

    local SECURITY_PLUGINS=(
        "secret-detection:Prevent secret leaks"
        "audit-log:Command usage auditing"
        "priv-escalation:Sudo usage alerts"
        "suspicious-pattern:Malicious command detection"
        "package-audit:Package installation safety"
        "ssh-gpg-monitor:Key usage monitoring"
    )

    info "Configuration Setup"
    echo "Do you want to customize enabled plugins? [y/N] (Default: All recommended)"
    read -r customize
    
    if [[ "$customize" =~ ^([yY][eE][sS]|[yY])$ ]]; then
        info "Context Plugins:"
        for entry in "${PLUGINS[@]}"; do
            KEY="${entry%%:*}"
            DESC="${entry#*:}"
            echo -n "  Enable $KEY ($DESC)? [Y/n] "
            read -r ans
            if [[ -z "$ans" || "$ans" =~ ^([yY][eE][sS]|[yY])$ ]]; then
                ENABLED_PLUGINS="$ENABLED_PLUGINS\"$KEY\", "
            fi
        done

        info "Security Plugins:"
        for entry in "${SECURITY_PLUGINS[@]}"; do
            KEY="${entry%%:*}"
            DESC="${entry#*:}"
            echo -n "  Enable $KEY ($DESC)? [Y/n] "
            read -r ans
            if [[ -z "$ans" || "$ans" =~ ^([yY][eE][sS]|[yY])$ ]]; then
                ENABLED_PLUGINS="$ENABLED_PLUGINS\"$KEY\", "
            fi
        done
        
        # Remove trailing comma and space
        ENABLED_PLUGINS="${ENABLED_PLUGINS%, }"
    else
        # Default: Enable all
        ENABLED_PLUGINS="
    # Context
    \"git\", \"env\", \"prod-guard\", \"node\", \"python\", \"rust\", \"go\", \"docker\", \"kubernetes\",
    # Security
    \"secret-detection\", \"audit-log\", \"priv-escalation\", \"suspicious-pattern\", \"package-audit\", \"ssh-gpg-monitor\""
    fi

    info "Creating configuration..."
    cat > "$CONFIG_FILE" << EOF
# MASTerm Configuration
# https://github.com/singhalmridul/MASTerm

[core]
mode = "dev"
log_level = "warn"

[prompt]
format = "powerline"
add_newline = true
left = ["directory", "git_branch", "git_status"]
right = ["cmd_duration"]

[prompt.icons]
mode = "auto"

[prompt.colors]
theme = "catppuccin"

[plugins]
enabled = [$ENABLED_PLUGINS]

[safety]
prod_detection = true
dangerous_commands = [
    "rm -rf",
    "DROP DATABASE",
    "kubectl delete",
    "terraform destroy"
]
EOF
    success "Created $CONFIG_FILE"
}

# Setup Zsh
setup_zsh() {
    local RC_FILE="$HOME/.zshrc"
    
    info "Setting up Zsh..."
    
    # Create .zshrc if it doesn't exist
    touch "$RC_FILE"
    
    # Check if already configured
    if grep -q "masterm init" "$RC_FILE" 2>/dev/null; then
        warn "Zsh already configured"
        return 0
    fi
    
    # Add to .zshrc
    cat >> "$RC_FILE" << 'EOF'

# Cargo/Rust PATH
export PATH="$HOME/.cargo/bin:$PATH"

# MASTerm - Master your Terminal
eval "$(masterm init zsh)"
EOF
    
    success "Added MASTerm to $RC_FILE"
}

# Setup Bash
setup_bash() {
    local RC_FILE="$HOME/.bashrc"
    
    info "Setting up Bash..."
    
    # Create .bashrc if it doesn't exist
    touch "$RC_FILE"
    
    # Check if already configured
    if grep -q "masterm init" "$RC_FILE" 2>/dev/null; then
        warn "Bash already configured"
        return 0
    fi
    
    # Add to .bashrc
    cat >> "$RC_FILE" << 'EOF'

# Cargo/Rust PATH
export PATH="$HOME/.cargo/bin:$PATH"

# MASTerm - Master your Terminal
eval "$(masterm init bash)"
EOF
    
    success "Added MASTerm to $RC_FILE"
}

# Setup Fish
setup_fish() {
    local CONFIG_DIR="$HOME/.config/fish"
    local RC_FILE="$CONFIG_DIR/config.fish"
    
    info "Setting up Fish..."
    
    # Create config directory if it doesn't exist
    mkdir -p "$CONFIG_DIR"
    touch "$RC_FILE"
    
    # Check if already configured
    if grep -q "masterm init" "$RC_FILE" 2>/dev/null; then
        warn "Fish already configured"
        return 0
    fi
    
    # Add to config.fish
    cat >> "$RC_FILE" << 'EOF'

# Cargo/Rust PATH
set -gx PATH $HOME/.cargo/bin $PATH

# MASTerm - Master your Terminal
masterm init fish | source
EOF
    
    success "Added MASTerm to $RC_FILE"
}

# Setup shell based on type
setup_shell() {
    local shell="$1"
    
    case "$shell" in
        zsh)  setup_zsh;;
        bash) setup_bash;;
        fish) setup_fish;;
        *)    warn "Unknown shell: $shell";;
    esac
}

# Print success message
print_success() {
    local shell="$1"
    
    echo ""
    echo -e "${GREEN}${BOLD}╔══════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}${BOLD}║         MASTerm installed successfully! 🚀                   ║${NC}"
    echo -e "${GREEN}${BOLD}╚══════════════════════════════════════════════════════════════╝${NC}"
    echo ""
    echo -e "To start using MASTerm, run:"
    echo ""
    echo -e "  ${CYAN}exec \$SHELL${NC}"
    echo ""
    echo -e "Or simply ${BOLD}restart your terminal${NC}."
    echo ""
    echo -e "Quick commands:"
    echo -e "  ${CYAN}masterm setup${NC}       - Run setup wizard (Recommended)"
    echo -e "  ${CYAN}masterm welcome${NC}     - View status screen"
    echo -e "  ${CYAN}masterm plugins${NC}     - Browse marketplace"
    echo -e "  ${CYAN}masterm doctor${NC}      - Check installation health"
    echo ""
    echo -e "Documentation: ${BLUE}https://github.com/$REPO${NC}"
    echo ""
}

# Main installation
main() {
    local SHELL_TYPE=""
    
    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --shell)
                SHELL_TYPE="$2"
                shift 2
                ;;
            --help|-h)
                echo "MASTerm Installation Script"
                echo ""
                echo "Usage: install.sh [options]"
                echo ""
                echo "Options:"
                echo "  --shell <type>   Specify shell (zsh, bash, fish)"
                echo "  --help           Show this help message"
                exit 0
                ;;
            *)
                shift
                ;;
        esac
    done
    
    header "MASTerm Installation"
    
    # Detect system
    OS=$(detect_os)
    ARCH=$(detect_arch)
    
    if [ -z "$SHELL_TYPE" ]; then
        SHELL_TYPE=$(detect_shell)
    fi
    
    info "System: $OS ($ARCH)"
    info "Shell: $SHELL_TYPE"
    echo ""
    
    # Check dependencies
    if ! has_command git; then
        error "Git is required but not installed"
        exit 1
    fi
    
    if ! has_command curl; then
        error "Curl is required but not installed"
        exit 1
    fi
    
    # Install Rust
    install_rust
    
    # Install MASTerm
    install_from_source
    
    # Create directories
    create_directories
    
    # Create config
    create_config
    
    # Setup shell
    setup_shell "$SHELL_TYPE"
    
    # Print success
    print_success "$SHELL_TYPE"
}

# Run main
main "$@"
