# MASTerm Security Documentation

MASTerm includes a comprehensive cybersecurity layer with 10 built-in security plugins designed to protect developers from common threats, secrets exposure, and dangerous commands.

## 🔐 Security Features Overview

| Feature | Description |
|---------|-------------|
| **Secret Detection** | Detects 25+ patterns of API keys, tokens, passwords |
| **Threat Detection** | Identifies 20+ threat patterns (reverse shells, encoded commands) |
| **Audit Logging** | Cryptographically-signed, append-only command logs |
| **Privilege Escalation** | Environment-aware warnings for sudo/su/doas |
| **Network Monitoring** | Tracks outbound connections via curl/ssh/nc |
| **Package Audit** | Typosquatting detection, malicious package blocklist |
| **File Integrity** | Alerts on access to .ssh, .env, /etc/shadow |
| **SSH/GPG Monitoring** | Tracks key generation, export, and deletion |
| **IP/Domain Reputation** | Check targets against threat intelligence |
| **Sandbox Mode** | Restricted execution environment |

---

## 🔑 CLI Commands

### Security Status

```bash
masterm security status
```

Shows current security configuration and active plugins.

### Analyze Commands

```bash
masterm security check -- "your command here"
```

Analyzes a command for security risks without executing it.

**Example:**

```bash
$ masterm security check -- "export AWS_ACCESS_KEY_ID=AKIAIOSFODNN7EXAMPLE"

🔍 Security Analysis
════════════════════════════════════════════════════════════

  Command: export AWS_ACCESS_KEY_ID=AKIAIOSFODNN7EXAMPLE

  🔐 Secrets Detected:
     ☁️ AWS Credentials (AKIA************MPLE)

  Risk Level: Critical
```

### View Detection Patterns

```bash
masterm security patterns                    # Show all patterns
masterm security patterns --pattern-type secrets   # Secrets only
masterm security patterns --pattern-type threats   # Threats only
```

### Audit Log Management

```bash
masterm security audit show                 # Show recent entries
masterm security audit show --count 50      # Show 50 entries
masterm security audit verify               # Verify log integrity
masterm security audit export --output backup.json  # Export logs
```

### Security Configuration

```bash
masterm security config show                # Show current config
masterm security config enable ip-reputation    # Enable feature
masterm security config disable audit-log       # Disable feature
masterm security config level paranoid      # Set security level
```

### Sandbox Mode

```bash
masterm security sandbox enter              # Enter sandbox
masterm security sandbox enter --allow-net  # Allow network access
masterm security sandbox status             # Check sandbox status
masterm security sandbox exit               # Exit sandbox
```

---

## 🛡️ Security Plugins Detail

### 1. Secret Detection

Detects hardcoded secrets before they're exposed:

| Pattern | Examples |
|---------|----------|
| AWS Keys | `AKIA*`, `ASIA*` |
| GitHub Tokens | `ghp_*`, `gho_*`, `ghs_*`, `ghu_*` |
| GitLab Tokens | `glpat-*` |
| Slack Tokens | `xoxb-*`, `xoxa-*`, `xoxp-*` |
| Stripe Keys | `sk_live_*`, `pk_live_*` |
| Google API Keys | `AIza*` |
| Private Keys | `-----BEGIN * PRIVATE KEY-----` |
| JWT Tokens | `eyJ*.eyJ*.*` |

**Configuration:**

```toml
[plugins.secret-detection]
action = "confirm"  # warn | confirm | block
```

### 2. Audit Logging

Forensic-grade command logging with:

- **SHA-256 hash chains** for tamper detection
- **Secret redaction** before logging
- **JSON format** for SIEM integration
- **Append-only storage**

**Log location:** `~/.masterm/security/audit.log`

**Configuration:**

```toml
[plugins.audit-log]
enabled = true
log_path = "~/.masterm/security/audit.log"
redact_secrets = true
```

### 3. Privilege Escalation Alerts

Environment-aware warnings for privilege escalation:

| Environment | Default Action |
|-------------|----------------|
| Development | Warn |
| Staging | Confirm |
| Production | Block (configurable) |

Detected commands: `sudo`, `su`, `doas`, `pkexec`, `setuid`

### 4. Suspicious Pattern Detection

Detects dangerous command patterns:

| Category | Examples |
|----------|----------|
| Reverse Shells | `bash -i >& /dev/tcp/...`, `nc -e /bin/sh` |
| Encoded Commands | `base64 -d | sh`, `eval "$(base64 -d ..."` |
| History Evasion | `unset HISTFILE`, `history -c` |
| Fork Bombs | `:(){ :|:& };:` |
| Data Exfiltration | `curl ... | bash` |

### 5. Network Monitor

Tracks outbound connections via:

- `curl`, `wget`, `httpie`
- `ssh`, `scp`, `sftp`
- `nc`, `netcat`, `ncat`
- `telnet`, `ftp`

### 6. Package Audit

Protects against supply chain attacks:

- **Typosquatting detection** (Levenshtein distance)
- **Malicious package blocklist**
- **Unscoped npm package warnings**

**Supported package managers:** npm, yarn, pnpm, pip, cargo, gem

### 7. File Integrity

Alerts on access to sensitive files:

| Category | Paths |
|----------|-------|
| SSH Keys | `~/.ssh/*`, `id_rsa`, `authorized_keys` |
| GPG Keys | `~/.gnupg/*` |
| Cloud Credentials | `~/.aws/*`, `~/.config/gcloud/*` |
| Environment Files | `.env*` |
| System Auth | `/etc/passwd`, `/etc/shadow`, `/etc/sudoers` |

### 8. SSH/GPG Monitor

Tracks cryptographic key operations:

- Key generation (`ssh-keygen`, `gpg --gen-key`)
- Key loading (`ssh-add`)
- Key export (`gpg --export`)
- Key deletion (`gpg --delete-key`)

### 9. IP/Domain Reputation

Check targets against threat intelligence:

- Local blocklist caching
- Safe domain filtering
- Extensible for external APIs (AbuseIPDB, VirusTotal)

### 10. Sandbox Mode

Restricted execution environment:

- Blocks privilege escalation
- Optional network blocking
- Directory allowlisting

---

## ⚙️ Configuration

### Security Levels

| Level | Description |
|-------|-------------|
| `low` | Minimal protection, warnings only |
| `medium` | Confirmation for dangerous ops |
| `high` | Default, blocks critical threats |
| `paranoid` | Maximum protection, strict blocking |

```toml
# ~/.masterm.toml

[security]
level = "high"

[plugins.secret-detection]
enabled = true
action = "confirm"

[plugins.audit-log]
enabled = true
redact_secrets = true

[plugins.priv-escalation]
dev_action = "warn"
staging_action = "confirm"
prod_action = "block"

[plugins.suspicious-pattern]
block_reverse_shells = true
block_encoded_commands = true

[plugins.sandbox]
allow_network = false
blocked_commands = ["rm -rf /", "mkfs"]
```

---

## 📊 Security Best Practices

1. **Enable audit logging** in production environments
2. **Use sandbox mode** when testing untrusted scripts
3. **Review patterns regularly** with `masterm security patterns`
4. **Check commands before execution** with `masterm security check`
5. **Export audit logs** for compliance requirements

---

## 🔍 Troubleshooting

### Audit log not being written

```bash
masterm security status  # Check if audit-log is enabled
ls -la ~/.masterm/security/  # Check directory permissions
```

### False positives

Add exceptions in your config:

```toml
[plugins.secret-detection]
# Whitelist specific patterns
exclude_patterns = ["test-api-key-*"]
```

### Performance concerns

Security plugins are designed for minimal overhead:

- Pattern matching: < 1ms per command
- Audit logging: async, non-blocking
- Caching for reputation lookups

---

## 📄 License

MIT License — see [LICENSE](LICENSE) for details.
