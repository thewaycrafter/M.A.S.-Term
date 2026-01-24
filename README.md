# MASTerm 🚀

**Master your Terminal** - A fast, intelligent, cross-shell terminal framework.

[![CI](https://github.com/masterm-dev/masterm/workflows/CI/badge.svg)](https://github.com/masterm-dev/masterm/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

## Why MASTerm?

MASTerm is a modern terminal framework designed to be:

- ⚡ **Fast** - < 50ms startup time (vs 200-500ms for Oh My Zsh)
- 🧠 **Intelligent** - Zero-config context awareness
- 🐚 **Cross-shell** - Works with Zsh, Bash, Fish, and PowerShell
- 🔒 **Safe** - Production environment guards
- 🏢 **Enterprise-ready** - Central config, audit logs, lockdown mode

## Features

### 🎨 Beautiful Prompts
```
~/projects/api  main +2 ~1  v18.19.0  PROD
❯ _
```

### 🔍 Context Detection
- Git branch, status, ahead/behind
- Language versions (Node, Python, Go, Rust, Java)
- Container context (Docker, Kubernetes)
- Environment detection (dev/staging/prod)

### 🛡️ Production Safety
```
⚠️  PRODUCTION WARNING
You are about to run: rm -rf logs/
Type 'yes' to confirm:
```

### ⚙️ Declarative Configuration
```toml
# ~/.masterm.toml
[prompt]
format = "powerline"
theme = "catppuccin"

[safety]
prod_detection = true
dangerous_commands = ["rm -rf", "DROP DATABASE"]
```

## Installation

### Quick Install
```bash
curl -fsSL https://masterm.dev/install.sh | bash
```

### From Source
```bash
git clone https://github.com/masterm-dev/masterm.git
cd masterm
cargo install --path crates/masterm-cli
masterm install
```

### Package Managers
```bash
# Homebrew (macOS/Linux)
brew install masterm

# Cargo
cargo install masterm
```

## Quick Start

```bash
# Install for your shell
masterm install

# Check installation
masterm doctor

# Switch modes
masterm mode dev      # Development mode (default)
masterm mode ops      # Operations mode (extra safety)
masterm mode minimal  # Minimal mode (fastest)
```

## Configuration

MASTerm uses a 3-tier configuration system:

1. **Enterprise** (`/etc/masterm/enterprise.toml`) - System-wide settings
2. **User** (`~/.masterm.toml`) - Personal preferences
3. **Project** (`.masterm.toml`) - Project-specific overrides

### Example Configuration

```toml
[core]
mode = "dev"  # minimal, dev, ops

[prompt]
format = "powerline"
add_newline = true
left = ["directory", "git_branch", "git_status"]
right = ["cmd_duration"]

[prompt.icons]
mode = "auto"  # auto, nerd, unicode, ascii, none

[prompt.colors]
theme = "catppuccin"  # catppuccin, dracula, nord

[plugins]
enabled = ["git", "node", "python"]
disabled = []

[safety]
prod_detection = true
dangerous_commands = [
    "rm -rf",
    "DROP DATABASE",
    "kubectl delete",
    "terraform destroy"
]
```

## CLI Commands

| Command | Description |
|---------|-------------|
| `masterm install` | Install MASTerm for your shell |
| `masterm doctor` | Diagnose installation issues |
| `masterm config show` | Display current configuration |
| `masterm config edit` | Open config in editor |
| `masterm plugins list` | List installed plugins |
| `masterm mode <mode>` | Switch between modes |
| `masterm profile startup` | Profile startup performance |
| `masterm update` | Update to latest version |

## Plugin System

MASTerm includes built-in plugins:

| Plugin | Description |
|--------|-------------|
| `git` | Git branch, status, ahead/behind |
| `env` | Environment type detection |
| `prod-guard` | Production safety guards |
| `node` | Node.js version |
| `python` | Python version + virtualenv |
| `go` | Go version |
| `rust` | Rust version |

## Enterprise Features

- **Central Configuration** - Deploy settings across your organization
- **Plugin Allowlists** - Control which plugins can be installed
- **Audit Logging** - Track configuration changes
- **Lockdown Mode** - Prevent user modifications

```toml
# /etc/masterm/enterprise.toml
[enterprise]
name = "Acme Corp"

[enterprise.lockdown]
config = true
plugins = true

[enterprise.plugins]
allowed = ["git", "env", "prod-guard"]
```

## Performance

| Metric | MASTerm | Oh My Zsh |
|--------|---------|-----------|
| Startup | < 50ms | 200-500ms |
| Prompt render | < 30ms | 50-100ms |
| Memory | ~5MB | ~50MB |

## Shell Support

| Shell | Status |
|-------|--------|
| Zsh | ✅ Full support |
| Bash | ✅ Full support |
| Fish | ✅ Full support |
| PowerShell | ✅ Full support |

## Contributing

We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

MIT License - see [LICENSE](LICENSE) for details.

---

**MASTerm** - Master your Terminal ⚡
