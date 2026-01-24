#!/bin/bash
#
# MASTerm Zsh Setup Script
# Run: source setup-zsh.sh
#

CARGO_BIN="$HOME/.cargo/bin"
RC_FILE="$HOME/.zshrc"

echo "Setting up MASTerm for Zsh..."

# Add to .zshrc if not already present
if ! grep -q "masterm init" "$RC_FILE" 2>/dev/null; then
    cat >> "$RC_FILE" << 'EOF'

# Cargo/Rust PATH
export PATH="$HOME/.cargo/bin:$PATH"

# MASTerm - Master your Terminal
eval "$(masterm init zsh)"
EOF
    echo "✓ Added MASTerm to $RC_FILE"
else
    echo "⚠ Zsh already configured"
fi

echo ""
echo "To apply changes, run: exec \$SHELL"
echo "Or restart your terminal."
