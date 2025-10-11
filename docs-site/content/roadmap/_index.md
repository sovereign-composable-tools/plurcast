+++
title = "Roadmap"
description = "Development roadmap and future vision for Plurcast"
weight = 4
+++

# Roadmap

Plurcast follows a **foundation-first** development approach. Each phase builds solid infrastructure before adding new features.

## Current Status: Alpha Release (v0.2.0)

**✅ Foundation Complete** - Multi-platform posting with secure credential management

### What Works Today

- **Multi-platform posting**: Nostr, Mastodon, Bluesky
- **Concurrent execution**: Posts to all platforms simultaneously
- **Secure credentials**: OS keyring + encrypted file fallback
- **Local database**: SQLite for post history and metadata
- **Rich CLI tools**: Comprehensive help, meaningful exit codes
- **Unix composability**: Works seamlessly with pipes and scripts
- **JSON output**: Machine-readable for automation

### Current Tools

| Tool | Status | Purpose |
|------|---------|---------|
| `plur-post` | ✅ **Stable** | Post content to platforms |
| `plur-history` | ✅ **Stable** | Query posting history |
| `plur-creds` | ✅ **Stable** | Manage credentials securely |
| `plur-setup` | ✅ **Stable** | Interactive configuration wizard |

## Phase 1: Platform Solidification (Current Focus)

**Goal**: Rock-solid multi-platform support before advancing to UI/UX

### Platform Robustness
- **Rate limiting**: Proper handling of platform limits
- **Error recovery**: Exponential backoff, retry logic
- **Edge cases**: Network failures, auth token refresh
- **Content validation**: Character limits, encoding issues

### Developer Experience
- **Comprehensive testing**: All platforms, all scenarios
- **Better error messages**: Actionable feedback
- **Platform quirks documentation**: Known limitations and workarounds
- **Performance optimization**: Faster posting, lower latency

### Timeline: **Ongoing** (2-4 weeks)

## Phase 2: Rich Interfaces (Next)

**Goal**: Terminal and desktop interfaces that preserve Unix principles

### Terminal UI with Ratatui

**Why Ratatui?**
- Native Rust integration
- Terminal-first philosophy
- Preserves composability
- Fast, responsive interface

**Features**:
```
┌─ Plurcast TUI ─────────────────────────────────┐
│ [Compose] [History] [Platforms] [Settings]    │
├───────────────────────────────────────────────┤
│ Status: ✓ nostr ✓ mastodon ✗ bluesky         │
│                                               │
│ Compose New Post:                            │
│ ┌─────────────────────────────────────────┐   │
│ │ Hello decentralized world! #rust       │   │
│ │                                         │   │
│ └─────────────────────────────────────────┘   │
│ Chars: 34/∞  [Post] [Draft] [Cancel]         │
│                                               │
│ Recent Posts:                                 │
│ 2025-10-11 15:30 | Hello world              │
│   ✓ nostr:note1abc... ✓ mastodon:12345     │
└───────────────────────────────────────────────┘
```

**Technical Design**:
- **Backend**: Uses existing CLI tools as library
- **Data flow**: Same database, same configuration
- **Composability**: Still reads stdin, outputs to stdout
- **Exit codes**: Meaningful status for scripting

### Desktop GUI with Tauri

**Why Tauri?**
- Rust-native backend
- Small binary size
- Cross-platform deployment
- Web technologies for UI

**Architecture**:
```
┌─────────────────┐    ┌──────────────────┐
│   Tauri GUI     │    │   CLI Backend    │
│  (TypeScript)   │◄──►│   (Rust)         │
└─────────────────┘    └──────────────────┘
          │                       │
          │              ┌──────────────────┐
          └──────────────►│ libplurcast      │
                         │ (Shared Library) │
                         └──────────────────┘
```

**Features**:
- **Multi-account support**: Switch between different credential sets
- **Visual feedback**: Real-time posting status
- **Draft management**: Save and edit drafts
- **History browser**: Rich post history with search
- **Settings UI**: Visual configuration management

### Timeline: **3-6 months**

## Phase 3: Semantic Intelligence (Future Vision)

**Goal**: Local-first AI features that enhance without compromising privacy

### Local Vector Embeddings

**Why Local?**
- **Privacy**: No cloud API calls, no data leaks
- **Offline capability**: Works without internet
- **Performance**: No network latency for search
- **Cost**: No API fees or usage limits

**Technical Implementation**:
```rust
// Hypothetical API
use libplurcast::semantic::{SemanticDatabase, EmbeddingModel};

let semantic_db = SemanticDatabase::new(
    &config.database.path,
    EmbeddingModel::AllMiniLML6V2, // Local transformer model
)?;

// Semantic search
let similar_posts = semantic_db
    .search("posts about feeling excited")
    .limit(10)
    .await?;

// Content recommendations
let suggested_content = semantic_db
    .suggest_topics()
    .based_on_recent_posts(7) // last 7 days
    .await?;
```

### Features

**Enhanced Search**:
```bash
# Traditional keyword search
plur-history --search "rust"

# Semantic search (new)
plur-history --semantic "programming languages I've mentioned"
plur-history --semantic "posts about feeling frustrated"
plur-history --similar-to "abc-123"  # Find similar posts
```

**Content Intelligence**:
```bash
# Topic suggestions based on history
plur-suggest topics --days 7

# Content analysis
plur-analyze sentiment --content "draft post..."
plur-analyze topics --content "draft post..."

# Duplicate detection
plur-check duplicate --content "new post..."
```

**Smart Drafts**:
- **Auto-categorization**: Automatically tag drafts by topic
- **Similar post detection**: "You posted something similar 3 days ago"
- **Content suggestions**: "You often post about X on weekends"

### Technical Stack

**Embedding Models** (Local inference):
- **all-MiniLM-L6-v2**: Fast, good quality for short text
- **sentence-transformers**: Multilingual support
- **Custom models**: Domain-specific embeddings

**Implementation Libraries**:
- **candle-rs**: Rust-native ML inference
- **ort**: ONNX runtime for Rust
- **tokenizers**: Fast text tokenization

**Vector Storage**:
- **SQLite with vector extension**: Simple, integrated
- **faiss-rs**: High-performance vector similarity
- **hnswlib-rs**: Approximate nearest neighbors

### Timeline: **6-12 months**

## Phase 4: Advanced Features (Long-term)

### Platform Expansion
- **ActivityPub**: Support for Fediverse beyond Mastodon
- **AT Protocol**: Enhanced Bluesky features
- **New protocols**: IPFS-based platforms, other decentralized networks

### Automation & Scheduling
- **plur-queue**: Intelligent post scheduling
- **plur-send**: Background daemon for automated posting
- **Content pipelines**: RSS feeds, GitHub activity, system monitoring

### Data Portability
- **plur-import**: Import from Twitter archives, Mastodon exports
- **plur-export**: Export to various formats (JSON, CSV, YAML)
- **Migration tools**: Move between different Plurcast installations

### Advanced Analytics
- **Local analytics**: No external tracking, just local insights
- **Engagement patterns**: Best times to post, platform preferences
- **Content performance**: Which topics resonate with your audience

## Design Principles (All Phases)

### 1. **Foundation First**
Never add new features that compromise the foundation. TUI/GUI build on CLI tools, not replace them.

### 2. **Local-First Always**
Every feature must work offline. Cloud features are optional enhancements, never requirements.

### 3. **Unix Philosophy**
Preserve composability. New tools should integrate with existing workflows, not replace them.

### 4. **Privacy by Design**
No feature should require sending user data to external services. Local processing preferred.

### 5. **Agent-Friendly**
Both humans and AI agents should be able to use every feature. JSON APIs for everything.

## Community Input

### What We'd Love to Hear

- **Platform priorities**: Which platforms should we support next?
- **Use cases**: How do you want to use Plurcast?
- **Integration needs**: What tools should we integrate with?
- **UI preferences**: Terminal vs. GUI vs. both?

### How to Contribute

- **GitHub Issues**: [Feature requests and bug reports](https://github.com/plurcast/plurcast/issues)
- **Discussions**: [Architecture and design conversations](https://github.com/plurcast/plurcast/discussions)
- **Pull Requests**: Code contributions welcome
- **Documentation**: Help improve guides and examples

## Technical Roadmap

### Performance Targets

| Operation | Current | Target (Phase 2) | Target (Phase 3) |
|-----------|---------|------------------|------------------|
| Post to 3 platforms | ~2s | ~1s | ~800ms |
| History query | ~50ms | ~20ms | ~10ms |
| Semantic search | N/A | N/A | ~100ms |
| TUI response | N/A | ~16ms | ~16ms |

### Platform Support

| Platform | Phase 1 | Phase 2 | Phase 3 | Phase 4 |
|----------|---------|---------|---------|---------|
| Nostr | ✅ | ✅ | ✅ | ✅ |
| Mastodon | ✅ | ✅ | ✅ | ✅ |
| Bluesky | ✅ | ✅ | ✅ | ✅ |
| ActivityPub | - | - | 🔮 | ✅ |
| IPFS-based | - | - | - | 🔮 |

### Binary Sizes (Estimated)

| Component | Current | Target |
|-----------|---------|---------|
| plur-post | ~8MB | ~6MB |
| plur-tui | N/A | ~12MB |
| plur-gui | N/A | ~25MB |

## Release Schedule

### v0.3.0 - Platform Hardening (Q1 2025)
- Enhanced error handling
- Rate limiting compliance
- Performance optimizations
- Comprehensive testing

### v0.4.0 - Terminal UI (Q2 2025)
- Interactive TUI with Ratatui
- Multi-account support
- Enhanced history browsing

### v0.5.0 - Desktop GUI (Q3 2025)
- Tauri-based desktop application
- Visual configuration
- Draft management

### v1.0.0 - Semantic Features (Q4 2025)
- Local vector embeddings
- Semantic search
- Content intelligence
- Stable API

## Success Metrics

### Technical Success
- **Reliability**: 99.9% successful posts under normal conditions
- **Performance**: Sub-second multi-platform posting
- **Compatibility**: Works on Windows, macOS, Linux
- **Security**: No credential leaks, pass security audits

### User Success
- **Adoption**: 1000+ active users
- **Retention**: 90% of users still active after 3 months
- **Contribution**: 10+ external contributors
- **Documentation**: Comprehensive guides for all features

### Ecosystem Success
- **Integration**: 5+ third-party tools integrate with Plurcast
- **Extensions**: Community-created platform adapters
- **Teaching**: Used in courses about Unix philosophy
- **Influence**: Other projects adopt similar approaches

---

**The journey is as important as the destination.** We're building Plurcast methodically, ensuring each phase provides real value while preparing for the next.

**Want to influence the roadmap?** Join the conversation on [GitHub Discussions](https://github.com/plurcast/plurcast/discussions).
