Plurcast is a collection of Unix command-line tools for scheduled cross-posting to decentralized social media platforms (Nostr, Bluesky, Mastodon). Following the Unix philosophy, each tool does one thing well and communicates through standard streams and files.

## Core Principles

- **Unix Philosophy**: Small, focused tools that compose via pipes and standard streams
- **Local-First**: All data stored locally in SQLite, no external services required
- **Agent-Friendly**: Designed for both human and AI agent operation through predictable interfaces
- **Decentralized Values**: User ownership, platform independence, self-contained operation

## Tool Suite

- `plur-post` - Post content immediately or as draft
- `plur-queue` - Schedule posts for later
- `plur-send` - Daemon that processes queue
- `plur-history` - Query posting history
- `plur-import` - Import from platform exports
- `plur-export` - Export posts to various formats

## Current Status

Version: 0.1.0-alpha (Foundation Phase)
