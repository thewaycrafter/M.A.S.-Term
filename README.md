<p align="center">
  <img src="https://img.shields.io/badge/MASTerm-Master%20your%20Terminal-blueviolet?style=for-the-badge&logo=terminal" alt="MASTerm">
</p>

<h1 align="center">🚀 MASTerm</h1>

<p align="center">
  <strong>Master your Terminal</strong> — A fast, intelligent, cross-shell terminal framework built in Rust.
</p>

<p align="center">
  <a href="#-features">Features</a> •
  <a href="#-installation">Installation</a> •
  <a href="#-shell-setup">Shell Setup</a> •
  <a href="#-usage">Usage</a> •
  <a href="#-plugins">Plugins</a> •
  <a href="#-configuration">Configuration</a> •
  <a href="#-contributing">Contributing</a>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Rust-1.70+-orange?logo=rust" alt="Rust">
  <img src="https://img.shields.io/badge/License-MIT-green" alt="License">
  <img src="https://img.shields.io/badge/Shells-Zsh%20%7C%20Bash%20%7C%20Fish%20%7C%20PowerShell-blue" alt="Shells">
  <img src="https://img.shields.io/github/stars/singhalmridul/MASTerm?style=social" alt="Stars">
</p>

---

## ⚡ Why MASTerm?

| Feature | MASTerm | Oh My Zsh |
|---------|---------|-----------|
| **Startup Time** | < 50ms | 200-500ms |
| **Prompt Render** | < 30ms | 50-100ms |
| **Memory Usage** | ~5MB | ~50MB |
| **Cross-Shell** | ✅ Yes | ❌ Zsh only |
| **Zero Config** | ✅ Yes | ❌ Requires setup |
| **Production Safety** | ✅ Built-in | ❌ No |

---

## ✨ Features

### 🎨 Beautiful Prompts
```
~/projects/api  main +2 ~1  v18.19.0  15ms
❯ _
```

### 🔍 Zero-Config Intelligence
- **Git** — Branch, status, ahead/behind, stash count
- **Languages** — Node.js, Python, Go, Rust, Java versions
- **Containers** — Docker, Kubernetes context
- **Environment** — Dev, staging, production detection

### 🛡️ Production Safety
```
⚠️  PRODUCTION WARNING
You are about to run: rm -rf logs/
Type 'yes' to confirm:
```

### 🎯 Multiple Modes
- **Minimal** — Fastest startup, basic prompt
- **Dev** — Balanced, git + language detection
- **Ops** — Maximum safety, all guards enabled

### 🏢 Enterprise Ready
- Central configuration support
- Plugin allow/deny lists
- Audit logging
- Lockdown mode

---

## 📦 Installation

### Quick Install (Recommended)

```bash
# Using curl
curl -fsSL https://masterm.dev/install.sh | bash

# Using wget
wget -qO- https://masterm.dev/install.sh | bash
```

### From Source

```bash
# Clone the repository
git clone https://github.com/singhalmridul/MASTerm.git
cd MASTerm

# Build and install
cargo build --release
cargo install --path crates/masterm-cli

# Run the installer
masterm install
```

### Package Managers

```bash
# Homebrew (macOS/Linux)
brew install masterm

# Cargo (Rust)
cargo install masterm

# AUR (Arch Linux)
yay -S masterm
```

### Verify Installation

```bash
masterm --version
masterm doctor
```

---

## 🐚 Shell Setup

MASTerm works with **Zsh**, **Bash**, **Fish**, and **PowerShell**. The installer auto-detects your shell, but you can also set it up manually.

### Automatic Setup

```bash
# Auto-detect and install for your current shell
masterm install

# Install for a specific shell
masterm install --shell zsh
masterm install --shell bash
masterm install --shell fish
masterm install --shell powershell
```

### Manual Setup

<details>
<summary><b>🔷 Zsh</b></summary>

Add to your `~/.zshrc`:

```zsh
# MASTerm - Master your Terminal
eval "$(masterm init zsh)"
```

Then reload:
```bash
source ~/.zshrc
# or
exec zsh
```
</details>

<details>
<summary><b>🟢 Bash</b></summary>

Add to your `~/.bashrc`:

```bash
# MASTerm - Master your Terminal
eval "$(masterm init bash)"
```

Then reload:
```bash
source ~/.bashrc
# or
exec bash
```
</details>

<details>
<summary><b>🐟 Fish</b></summary>

Add to your `~/.config/fish/config.fish`:

```fish
# MASTerm - Master your Terminal
masterm init fish | source
```

Then reload:
```bash
source ~/.config/fish/config.fish
# or
exec fish
```
</details>

<details>
<summary><b>💠 PowerShell</b></summary>

Add to your PowerShell profile (`$PROFILE`):

```powershell
# MASTerm - Master your Terminal
Invoke-Expression (& masterm init powershell)
```

Then reload:
```powershell
. $PROFILE
```
</details>

---

## 🎮 Usage

### Basic Commands

```bash
# Check installation health
masterm doctor

# View current configuration
masterm config show

# Edit configuration
masterm config edit

# Clear cache
masterm cache clear
```

### Mode Switching

Quickly switch between different operating modes:

```bash
# See current mode
masterm mode

# Switch to minimal mode (fastest)
masterm mode minimal

# Switch to dev mode (balanced)
masterm mode dev

# Switch to ops mode (maximum safety)
masterm mode ops
```

**Mode Comparison:**

| Mode | Startup | Plugins | Safety |
|------|---------|---------|--------|
| `minimal` | Fastest | None | Basic |
| `dev` | Fast | Git, Languages | Standard |
| `ops` | Normal | All | Maximum |

### Performance Profiling

```bash
# Profile startup time
masterm profile startup

# Profile prompt generation
masterm profile prompt

# Profile plugin load times
masterm profile plugins
```

### Shell Completions

Generate completions for your shell:

```bash
# Zsh
masterm completions zsh > ~/.zsh/completions/_masterm

# Bash
masterm completions bash > /etc/bash_completion.d/masterm

# Fish
masterm completions fish > ~/.config/fish/completions/masterm.fish

# PowerShell
masterm completions powershell >> $PROFILE
```

---

## 🔌 Plugins

MASTerm comes with built-in plugins and supports external plugins.

### Built-in Plugins

| Plugin | Description | Auto-Activates When |
|--------|-------------|---------------------|
| `git` | Git branch, status, ahead/behind | `.git` directory exists |
| `env` | Environment detection (dev/staging/prod) | Always |
| `prod-guard` | Production safety warnings | In production environment |
| `node` | Node.js version | `package.json` exists |
| `python` | Python version + virtualenv | `pyproject.toml`, `requirements.txt` |
| `go` | Go version | `go.mod` exists |
| `rust` | Rust version | `Cargo.toml` exists |
| `docker` | Docker context | `Dockerfile` exists |
| `kubernetes` | K8s context | `kubectl` available |

### Managing Plugins

```bash
# List all plugins
masterm plugins list

# Show detailed plugin status
masterm plugins list --status

# Get plugin info
masterm plugins info git

# Enable a plugin
masterm plugins enable node

# Disable a plugin
masterm plugins disable python
```

### Plugin Configuration

Configure plugins in `~/.masterm.toml`:

```toml
[plugins]
# Explicitly enable/disable plugins
enabled = ["git", "node", "python"]
disabled = ["java"]

# Git plugin settings
[plugins.git]
show_stash = true
show_ahead_behind = true
truncate_branch = 30

# Node plugin settings
[plugins.node]
show_version = true
```

### Installing External Plugins

```bash
# From plugin registry (coming soon)
masterm plugins install <plugin-name>

# From Git URL
masterm plugins install https://github.com/user/masterm-plugin-example

# Remove a plugin
masterm plugins remove <plugin-name>

# Update all plugins
masterm plugins update
```

---

## ⚙️ Configuration

MASTerm uses TOML configuration files with a 3-tier hierarchy:

1. **Enterprise** — `/etc/masterm/enterprise.toml` (system-wide)
2. **User** — `~/.masterm.toml` (personal)
3. **Project** — `.masterm.toml` (per-project)

### Quick Configuration

```bash
# Edit your config
masterm config edit

# Show current config
masterm config show

# Show effective (merged) config
masterm config show --effective

# Validate config syntax
masterm config validate

# Reset to defaults
masterm config reset
```

### Full Configuration Example

```toml
# ~/.masterm.toml - MASTerm Configuration

[core]
# Mode: minimal, dev, ops
mode = "dev"

# Log level: trace, debug, info, warn, error
log_level = "warn"

[prompt]
# Prompt style: powerline, simple, minimal
format = "powerline"

# Add newline before prompt
add_newline = true

# Left prompt segments (order matters)
left = ["directory", "git_branch", "git_status"]

# Right prompt segments
right = ["cmd_duration"]

[prompt.icons]
# Icon mode: auto, nerd, unicode, ascii, none
# 'auto' detects Nerd Fonts and falls back gracefully
mode = "auto"

[prompt.colors]
# Theme: catppuccin, dracula, nord
theme = "catppuccin"

# Custom color overrides
[prompt.colors.overrides]
directory = "#89b4fa"
git_branch = "#a6e3a1"

[plugins]
# Explicitly enabled/disabled plugins
enabled = []
disabled = []

[safety]
# Enable production environment detection
prod_detection = true

# Patterns that indicate production
prod_patterns = [
    "**/prod/**",
    "**/production/**",
    "/var/www/**"
]

# Commands requiring confirmation in production
dangerous_commands = [
    "rm -rf",
    "DROP DATABASE",
    "kubectl delete",
    "terraform destroy",
    "git push --force"
]

# Warning style: banner, background, border, icon
warning_style = "banner"

[cache]
# Cache TTL in seconds
ttl = 300

# Max cache size in MB
max_size = 100

[telemetry]
# Anonymous usage statistics (off by default)
enabled = false
```

---

## 🎨 Themes

MASTerm includes three beautiful built-in themes:

### Catppuccin (Default)
```toml
[prompt.colors]
theme = "catppuccin"
```

### Dracula
```toml
[prompt.colors]
theme = "dracula"
```

### Nord
```toml
[prompt.colors]
theme = "nord"
```

### Icon Modes

```toml
[prompt.icons]
# auto - Detect Nerd Fonts, fallback to Unicode/ASCII
mode = "auto"

# nerd - Force Nerd Font icons (requires Nerd Font)
mode = "nerd"

# unicode - Use Unicode symbols
mode = "unicode"

# ascii - Plain ASCII (most compatible)
mode = "ascii"

# none - No icons
mode = "none"
```

---

## 🏢 Enterprise Features

For team and organization deployments:

### Central Configuration

Create `/etc/masterm/enterprise.toml`:

```toml
[enterprise]
enabled = true
org_id = "acme-corp"

[lockdown]
# Prevent users from changing these settings
locked_settings = [
    "safety.prod_detection",
    "telemetry.enabled"
]

[plugins]
# Only allow these plugins
allowlist = ["git", "env", "prod-guard"]

# Block these plugins
denylist = ["telemetry-plugin"]

[safety]
# Force these patterns in all environments
force_prod_patterns = [
    "/var/www/**",
    "**/production/**"
]

# Force these dangerous command patterns
force_dangerous_commands = [
    "rm -rf /",
    "DROP DATABASE"
]
```

---

## 🔧 Troubleshooting

### Run Diagnostics

```bash
masterm doctor
```

This checks:
- ✅ Binary in PATH
- ✅ Config file syntax
- ✅ Shell integration
- ✅ Plugin directory
- ✅ Required dependencies
- ✅ Performance baseline

### Common Issues

<details>
<summary><b>Prompt not showing</b></summary>

1. Ensure MASTerm is in your PATH:
   ```bash
   which masterm
   ```

2. Re-run the installer:
   ```bash
   masterm install
   ```

3. Reload your shell:
   ```bash
   exec $SHELL
   ```
</details>

<details>
<summary><b>Icons not displaying</b></summary>

Install a [Nerd Font](https://www.nerdfonts.com/) and configure your terminal to use it. Or switch to Unicode/ASCII mode:

```toml
[prompt.icons]
mode = "unicode"  # or "ascii"
```
</details>

<details>
<summary><b>Slow startup</b></summary>

1. Check what's slow:
   ```bash
   masterm profile startup
   ```

2. Switch to minimal mode:
   ```bash
   masterm mode minimal
   ```

3. Disable unused plugins in config.
</details>

---

## 📊 Performance

| Metric | Target | Typical |
|--------|--------|---------|
| Startup | < 50ms | ~30ms |
| Prompt render | < 30ms | ~15ms |
| Plugin load | < 10ms each | ~5ms |
| Memory | < 10MB | ~5MB |

Benchmark yourself:
```bash
masterm profile startup
masterm profile prompt
```

---

## 🤝 Contributing

We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

```bash
# Clone
git clone https://github.com/singhalmridul/MASTerm.git
cd MASTerm

# Build
cargo build

# Test
cargo test

# Run locally
cargo run -- doctor
```

---

## 📄 License

MIT License — see [LICENSE](LICENSE) for details.

---

<p align="center">
  <b>MASTerm</b> — Master your Terminal ⚡
</p>

<p align="center">
  Made with ❤️ by <a href="https://github.com/singhalmridul">Mridul Singhal</a>
</p>
