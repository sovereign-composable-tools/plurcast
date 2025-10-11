+++
title = "Philosophy"
description = "The Unix philosophy and design principles behind Plurcast"
weight = 1
+++

# Philosophy

Why does the world need another social media tool? And why build it with Unix principles from the 1970s?

## The Problem with Social Media Tools

Most social media management tools today suffer from common problems:

- **Vendor Lock-in**: You're tied to a specific service or platform
- **Cloud Dependency**: Your data lives on someone else's servers
- **Privacy Concerns**: Analytics, tracking, and data collection
- **Complexity**: Bloated GUIs for simple tasks
- **Platform Silos**: Each platform needs its own tool

## The Unix Solution

Unix philosophy, developed in the 1970s, provides timeless principles for building robust, composable tools:

### 1. Do One Thing Well

Each Plurcast tool has a focused purpose:

- `plur-post` - Posts content to platforms
- `plur-history` - Queries posting history
- `plur-creds` - Manages credentials securely
- `plur-setup` - Interactive configuration

**Why this matters**: Simple tools are easier to understand, debug, and extend. They're also more reliable because they have fewer responsibilities.

```bash
# Each tool does exactly what you expect
plur-post "Hello world"        # Posts content
plur-history --limit 10        # Shows recent posts
plur-creds audit              # Checks security
```

### 2. Compose Naturally

Unix tools work together through pipes and text streams:

```bash
# Generate content with fortune, post to Nostr
fortune | plur-post --platform nostr

# Process draft, make substitutions, then post
cat draft.txt | sed 's/draft/final/g' | plur-post

# Post command output
echo "Current load: $(uptime)" | plur-post

# Chain with other tools
date +"Status update: %Y-%m-%d %H:%M" | plur-post --platform mastodon
```

**Why this matters**: Composability means infinite possibilities. You can combine Plurcast with any other command-line tool to create workflows we never imagined.

### 3. Text Streams as Universal Interface

Everything flows through text:

```bash
# Human-readable output (default)
plur-post "Test"
# nostr:note1abc123...
# mastodon:12345

# Machine-readable output (JSON)
plur-post "Test" --format json
# [{"platform":"nostr","success":true,"post_id":"note1..."}]

# Works with pipes
echo "Test" | plur-post | grep "nostr"
```

**Why this matters**: Text is universal. Any programming language, script, or tool can work with text. JSON provides structure when needed.

### 4. Meaningful Exit Codes

Scripts can handle errors properly:

```bash
#!/bin/bash
if plur-post "Important announcement"; then
    echo "✓ Posted successfully"
    notify-send "Posted to social media"
else
    case $? in
        1) echo "⚠ Partial failure - some platforms succeeded" ;;
        2) echo "✗ Authentication error" ;;
        3) echo "✗ Invalid input" ;;
    esac
fi
```

**Why this matters**: Proper error handling enables reliable automation and clear feedback.

## Local-First Philosophy

Beyond Unix principles, Plurcast embraces **local-first** design:

### Your Data, Your Machine

```bash
# All data stays local
~/.local/share/plurcast/posts.db    # SQLite database
~/.config/plurcast/config.toml      # Configuration
```

- **No cloud dependencies** - Works offline for local operations
- **No tracking** - No analytics, no phone-home behavior
- **No vendor lock-in** - Standard formats (SQLite, TOML, JSON)

### Secure by Default

```bash
# Credentials stored securely
plur-creds set nostr              # OS keyring (preferred)
plur-creds audit                  # Security validation
```

- **OS keyring integration** - Windows Credential Manager, macOS Keychain, Linux Secret Service
- **Encrypted file fallback** - Password-protected storage
- **No plain text secrets** - Unless explicitly configured

### Privacy Respecting

- **No data collection** - We don't know what you post
- **No external dependencies** - No third-party services
- **Transparent behavior** - Open source, auditable code

## Agent-Friendly Design

Plurcast works equally well for humans and AI agents:

### Consistent Interface

```bash
# Same commands work in scripts and interactively
plur-post "Hello world"
echo "Hello world" | plur-post
```

### Rich Metadata

```bash
# JSON output provides structured data
plur-post "Test" --format json | jq '.[] | select(.success == true)'
```

### Comprehensive Help

```bash
# Self-documenting commands
plur-post --help
plur-history --help
plur-creds --help
```

## Why This Approach Works

### 1. **Simplicity**
Each tool is small enough to understand completely. No hidden complexity.

### 2. **Reliability**
Focused tools have fewer failure modes. Unix principles have been battle-tested for 50+ years.

### 3. **Extensibility**
New platforms can be added without changing existing tools. New workflows emerge from composition.

### 4. **Longevity**
Text-based tools outlast GUI applications. Your scripts will work with future versions.

### 5. **Debuggability**
When something breaks, it's easy to isolate and fix. No black-box behavior.

## The Result

Plurcast isn't just another social media tool. It's a demonstration that:

- **Old principles solve new problems** - Unix philosophy applies to modern challenges
- **Simple tools enable complex workflows** - Composition beats monolithic applications
- **Local-first preserves user agency** - You control your data and workflows
- **Privacy is achievable** - Without sacrificing functionality

## What's Next

This philosophy guides our roadmap:

### Terminal UI (Coming)
Interactive mode that still respects Unix principles:
- Reads from stdin when available
- Outputs to stdout for piping
- Meaningful exit codes

### Desktop GUI (Planned)
Native application that uses CLI tools as backend:
- Composability preserved
- Same data, same configuration
- Visual interface for non-technical users

### Semantic Search (Vision)
Local AI that enhances without compromising:
- No cloud API calls
- Your data stays local
- Enhanced text search capabilities

## Influences

Plurcast draws inspiration from:

- **Unix philosophy** - Doug McIlroy, Ken Thompson, Dennis Ritchie
- **Suckless principles** - Simple, minimal, clean code
- **Local-first software** - Ink & Switch research
- **IndieWeb** - Own your data, control your online presence
- **Rust ecosystem** - Safety, performance, community values

---

**Philosophy drives design. Design drives implementation. Implementation serves users.**

Join us in building tools that respect both your time and your freedom.
