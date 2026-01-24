#!/usr/bin/env fish
#
# MASTerm Fish Setup Script
# Run: source setup-fish.fish
#

set CONFIG_DIR "$HOME/.config/fish"
set RC_FILE "$CONFIG_DIR/config.fish"

echo "Setting up MASTerm for Fish..."

# Create config directory if needed
mkdir -p $CONFIG_DIR

# Check if already configured
if not grep -q "masterm init" "$RC_FILE" 2>/dev/null
    echo '
# Cargo/Rust PATH
set -gx PATH $HOME/.cargo/bin $PATH

# MASTerm - Master your Terminal
masterm init fish | source' >> $RC_FILE
    echo "✓ Added MASTerm to $RC_FILE"
else
    echo "⚠ Fish already configured"
end

echo ""
echo "To apply changes, run: exec fish"
echo "Or restart your terminal."
