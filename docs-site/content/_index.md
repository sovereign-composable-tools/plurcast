+++
title = "Plurcast - Unix Tools for the Decentralized Web"
description = "Command-line tools for posting to Nostr, Mastodon, and Bluesky. Built on Unix principles."
template = "section.html"
+++

# Cast to Many

**Unix tools for the decentralized social web**

Plurcast is a collection of command-line tools for posting to decentralized social media platforms like Nostr, Mastodon, and Bluesky. Following Unix philosophy, each tool does one thing well and composes naturally with other command-line utilities.

## Quick Start

```bash
# Post to all enabled platforms
echo "Hello decentralized world!" | plur-post
# Output:
# nostr:note1abc123...
# mastodon:12345
# bluesky:at://did:plc:xyz.../app.bsky.feed.post/abc

# Post to specific platform
plur-post "Nostr-only message" --platform nostr

# Query your posting history
plur-history --limit 10
```

## Why Plurcast?

### 🔧 **Unix Philosophy**
- **Do one thing well**: Each tool has a focused purpose
- **Compose naturally**: `fortune | plur-post --platform nostr`
- **Text streams**: Works seamlessly with pipes and scripts

### 🔒 **Privacy-First**
- **Local database**: All data stays on your machine
- **Secure credentials**: OS keyring integration (Windows/macOS/Linux)
- **No tracking**: No analytics, no cloud dependencies

### 🌐 **Decentralized-Native**
- **Multi-platform**: Post to Nostr, Mastodon, and Bluesky simultaneously
- **Protocol-aware**: Native support for each platform's features
- **Future-ready**: Easy to add new decentralized platforms

### 🚀 **Developer-Friendly**
- **JSON output**: Perfect for scripts and automation
- **Agent-friendly**: Works equally well for humans and AI agents
- **Meaningful exit codes**: Proper error handling for scripts

## Current Status

**Alpha Release (v0.2.0)** - Multi-platform support with secure credential management

### ✅ What Works Today
- Post to Nostr, Mastodon, and Bluesky
- Concurrent multi-platform posting
- Local SQLite database for post history
- Secure credential storage (OS keyring + encrypted files)
- Rich CLI tools with comprehensive help
- Unix-style composability

### 🚧 What's Coming
- **Terminal UI** with Ratatui for interactive posting
- **Desktop GUI** with Tauri for non-technical users
- **Semantic search** with local AI embeddings
- **Advanced scheduling** and automation features

## Get Started

Choose your path:

- **[Quick Start](/getting-started/)** - Get posting in 5 minutes
- **[Philosophy](/philosophy/)** - Why we built Plurcast this way
- **[Documentation](/documentation/)** - Complete reference
- **[Roadmap](/roadmap/)** - What's coming next

## Installation

```bash
# From source (Rust required)
git clone https://github.com/plurcast/plurcast.git
cd plurcast
cargo install --path plur-post

# Quick test
plur-post --help
```

## Community

- **GitHub**: [plurcast/plurcast](https://github.com/plurcast/plurcast)
- **Issues**: [Report bugs or request features](https://github.com/plurcast/plurcast/issues)
- **Discussions**: [Join the conversation](https://github.com/plurcast/plurcast/discussions)

---

**Plurcast** - Cast to many, own your data, follow Unix principles.
