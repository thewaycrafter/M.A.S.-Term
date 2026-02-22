# MASTerm Manual 📘

**The Complete Guide to Mastering Your Terminal**

---

## 📚 Table of Contents
1. [Introduction](#introduction)
2. [Installation](#installation)
3. [Getting Started](#getting-started)
   - [Interactive Setup](#interactive-setup)
   - [The Welcome Screen](#the-welcome-screen)
4. [Prompt Engine](#prompt-engine)
   - [Structure](#structure)
   - [Themes](#themes)
   - [Icons](#icons)
5. [Features Deep Dive](#features-deep-dive)
   - [Context Awareness](#context-awareness)
   - [Production Safety Guards](#production-safety-guards)
   - [TUI Dashboard](#tui-dashboard)
   - [AI Command Assistance](#ai-command-assistance)
   - [Workflow Automation](#workflow-automation)
   - [Cloud Sync](#cloud-sync)
6. [Plugin System](#plugin-system)
   - [Marketplace](#marketplace)
   - [Architecture (WASM)](#architecture-wasm)
   - [Creating Plugins](#creating-plugins)
7. [Advanced Configuration](#advanced-configuration)
8. [Troubleshooting](#troubleshooting)

---

## 1. Introduction
MASTerm is a next-generation terminal framework written in Rust. It replaces your shell prompt with a high-performance, intelligent, and beautiful alternative that works across Zsh, Bash, and Fish.

## 2. Installation
### One-Line Install
```bash
curl -fsSL https://raw.githubusercontent.com/theWayCrafter/MASTerm/main/install.sh | bash
```

### Manual Build
```bash
git clone https://github.com/theWayCrafter/MASTerm.git
cd MASTerm
cargo install --path crates/masterm-cli
masterm install
```

## 3. Getting Started
### Interactive Setup
Configuring a terminal shouldn't require editing a text file. Run the wizard:
```bash
masterm setup
```
You will be asked about:
- **Icons**: Nerd Fonts vs Unicode
- **Theme**: Catppuccin, Dracula, or Nord
- **Safety**: Dev vs Ops mode

### The Welcome Screen
Upon logging in, you'll see a dashboard with:
- ASCII Art Banner
- CPU/Memory Usage
- Active Shell Version
If it's missing, run `masterm welcome` manually.

## 4. Prompt Engine
The prompt is divided into:
- **Left**: Contextual info (Dir, Git, Language Version)
- **Right**: Metadata (Duration, Time)

### Themes
Set in `~/.masterm.toml`:
```toml
[prompt.colors]
theme = "dracula" # or "catppuccin", "nord"
```

## 5. Features Deep Dive
### Production Safety Guards
MASTerm detects if you are in a "Production" environment (via ENV vars or git branch).
If you run `rm -rf` while in Production:
**IT BLOCK/WARNS YOU.**
Configure this in `[safety]`:
```toml
dangerous_commands = ["rm -rf", "DROP DATABASE"]
```

### TUI Dashboard
Run `masterm dashboard` to see a full-screen terminal UI monitor.
- **Tab 1**: System Monitor (REAL-TIME graphs)
- **Tab 2**: Configuration Viewer

### AI Command Assistance
Convert natural language to shell commands:
```bash
masterm ask "how to check disk space"
```
It provides a safe, explained command with a risk rating.

### Workflow Automation
Create a `workflows.toml` file in your project:
```toml
[deploy]
description = "Build and ship"
steps = ["cargo test", "./deploy.sh"]
```

Run it:
```bash
masterm run deploy
```

## 6. Plugin System
### Marketplace
Find new capabilities without leaving the terminal:
```bash
masterm plugins search <query>
masterm plugins install <name>
```

### Architecture (WASM)
Plugins are WebAssembly modules. They run in a sandboxed environment with strict permissions defined in `plugin.toml`.

## 7. Advanced Configuration
### Enterprise.toml
Admins can enforce settings globally via `/etc/masterm/enterprise.toml`.
Useful for teams requiring specific safety checks.

## 8. Troubleshooting
**Something wrong?**
Run the doctor:
```bash
masterm doctor
```

**Uninstalling**
```bash
masterm install --uninstall
rm -rf ~/.masterm
```
