#!/bin/bash
#
# MASTerm Bash Setup Script
# Run: source setup-bash.sh
#

CARGO_BIN="$HOME/.cargo/bin"
RC_FILE="$HOME/.bashrc"

echo "Setting up MASTerm for Bash..."

# Add to .bashrc if not already present
if ! grep -q "masterm init" "$RC_FILE" 2>/dev/null; then
    cat >> "$RC_FILE" << 'EOF'

# Cargo/Rust PATH
export PATH="$HOME/.cargo/bin:$PATH"

# MASTerm - Master your Terminal
eval "$(masterm init bash)"
EOF
    echo "✓ Added MASTerm to $RC_FILE"
else
    echo "⚠ Bash already configured"
fi

echo ""
echo "To apply changes, run: exec \$SHELL"
echo "Or restart your terminal."
