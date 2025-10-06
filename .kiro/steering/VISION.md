# Plurcast: Vision & Philosophy

**Related Documentation**:
- [Architecture](./ARCHITECTURE.md) - Technical implementation details
- [Roadmap](./ROADMAP.md) - Development phases and progress
- [Tools](./TOOLS.md) - Tool specifications and usage
- [Future](./FUTURE.md) - Extensibility and future plans

---

## Project Vision

Plurcast is a collection of Unix command-line tools for scheduled cross-posting to decentralized social media platforms. Following the Unix philosophy, each tool does one thing well, communicating through standard streams and files. Built with mature, open-source Rust libraries.

## Core Principles

### Unix Philosophy
- **Do one thing well**: Each binary handles a single responsibility
- **Text streams**: Universal interface between components
- **Composability**: Tools combine via pipes and standard Unix utilities
- **Silence is golden**: Only output what's needed, errors to stderr
- **Exit codes**: Meaningful status codes for scripting
- **Agent-friendly**: LLM agents can operate the tools just like humans

### Agent-Aware Design Philosophy

Plurcast is built with an **agent-aware philosophy** - by following Unix principles, the tools are inherently accessible to both humans and AI agents:

**Why Unix Philosophy Enables AI Agents:**
- **Predictable interfaces**: Standard input/output streams are easy for agents to manipulate
- **Composable workflows**: Agents can chain commands just like shell scripts
- **Clear contracts**: `--help` text and exit codes provide discoverable interfaces
- **Stateless operations**: Each tool call is independent, easier to reason about
- **JSON output modes**: Machine-readable formats via `--format json`
- **No hidden state**: Configuration in files, not in-memory sessions

**Agent Capabilities:**
```bash
# Agent can discover capabilities
plur-post --help | agent-parse

# Agent can compose workflows
agent: plur-history --since yesterday --format json |
       jq '.[] | select(.platform=="nostr")' |
       plur-export --format markdown

# Agent can handle errors via exit codes
if ! plur-post "content"; then
  agent: retry with --platform nostr only
fi
```

**Human-Agent Parity:**
- What a human can do via CLI, an agent can automate
- Agents discover features through help text and man pages
- Tools respond identically whether called by human or agent
- No special "API mode" - Unix tools ARE the API

This agent-aware design means Plurcast works seamlessly with:
- Claude Code and other coding assistants
- Shell script automation
- CI/CD pipelines
- Custom agent workflows
- Future agentic tools

The Unix philosophy isn't just good design - it's **agent-native design**.

### Decentralized Values
- **Local-first**: All data stored locally in SQLite
- **Self-contained**: No external services required for core functionality
- **User ownership**: Complete control over data and configuration
- **Platform independence**: Easy import/export, no lock-in

## Design Philosophy Summary

Plurcast embodies three interlocking principles:

1. **Unix Philosophy**: Tools that do one thing well, compose via text streams, work for both humans and agents
2. **Decentralized Values**: Local-first, user-owned data, platform independence
3. **Consciousness-Serving Technology**: Reveals patterns rather than manipulates, enhances awareness rather than creates dependency

This creates software that:
- Humans can learn and compose
- Agents can discover and automate
- Serves authentic expression over algorithmic control
- Extends gracefully from CLI to TUI to GUI
- Works equally well in 2025 and 2035

## Name Etymology

**Plurcast** = Latin *plur(i)* (many) + *cast* (broadcast)

"Cast to many" - perfectly captures the essence of cross-posting to multiple decentralized platforms while maintaining a clean, Unix-friendly name.

## Licensing & Community

**License**: MIT or Apache 2.0 (TBD)
**Repository**: GitHub (plurcast/plurcast)
**Community**: Focus on users who value data ownership and Unix principles

---

**Version**: 0.1.0-alpha
**Last Updated**: 2025-10-05
**Status**: Active Development - Phase 1 (Foundation) ~85% Complete
