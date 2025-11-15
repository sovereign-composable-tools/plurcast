+++
title = "Community"
description = "Join the Plurcast community - contribute, discuss, and help shape the future"
weight = 5
+++

# Community

Plurcast is an open-source project built by and for the community. Whether you're a developer, user, or just curious about decentralized social media, there's a place for you.

## 🚀 **Project Links**

### Primary
- **[GitHub Repository](https://github.com/plurcast/plurcast)** - Source code, issues, releases
- **[GitHub Discussions](https://github.com/plurcast/plurcast/discussions)** - Community conversations
- **[GitHub Issues](https://github.com/plurcast/plurcast/issues)** - Bug reports and feature requests

### Documentation
- **[This Site](https://plurcast.github.io)** - Comprehensive documentation
- **[README](https://github.com/plurcast/plurcast/blob/main/README.md)** - Quick project overview
- **[TESTING_OVERVIEW](https://github.com/plurcast/plurcast/blob/main/TESTING_OVERVIEW.md)** - Testing guides

## 💬 **Join the Conversation**

### GitHub Discussions

Our primary community space for:
- **[General Discussion](https://github.com/plurcast/plurcast/discussions/categories/general)** - Questions, ideas, showcase
- **[Feature Requests](https://github.com/plurcast/plurcast/discussions/categories/ideas)** - Propose new features
- **[Help & Support](https://github.com/plurcast/plurcast/discussions/categories/q-a)** - Get help from the community
- **[Development](https://github.com/plurcast/plurcast/discussions/categories/development)** - Technical discussions

### Social Media

Find us on the platforms we support:
- **Nostr**: `npub1...` (coming soon)
- **Mastodon**: `@plurcast@hachyderm.io` (coming soon)  
- **SSB**: Support via pub servers (experimental)

## 🤝 **Ways to Contribute**

### For Developers

#### Code Contributions
```bash
# Fork and clone
git clone https://github.com/yourusername/plurcast.git
cd plurcast

# Create feature branch
git checkout -b feature/amazing-feature

# Make changes, test, commit
cargo test
git commit -m "Add amazing feature"

# Push and create PR
git push origin feature/amazing-feature
```

**Areas where we need help:**
- **Platform support**: New decentralized platforms
- **Error handling**: Better error messages and recovery
- **Testing**: More comprehensive test coverage
- **Performance**: Optimization and profiling
- **Documentation**: Code comments and examples

#### Technical Writing
- **API documentation**: Document internal APIs
- **Architecture guides**: Explain system design
- **Platform guides**: Setup instructions for new platforms
- **Troubleshooting**: Common issues and solutions

### For Users

#### Testing & Feedback
- **Alpha testing**: Try new features before release
- **Bug reports**: [Report issues](https://github.com/plurcast/plurcast/issues/new?template=bug_report.md)
- **Feature requests**: [Suggest improvements](https://github.com/plurcast/plurcast/discussions/new?category=ideas)
- **Platform testing**: Test on different operating systems

#### Documentation
- **User guides**: Write tutorials and how-tos
- **Use cases**: Share your workflows and scripts
- **Examples**: Contribute code examples
- **Translations**: Help localize documentation

#### Community Building
- **Answer questions**: Help others in discussions
- **Share knowledge**: Write blog posts, make videos
- **Spread the word**: Tell others about Plurcast
- **Organize**: Local meetups, online events

## 🎯 **Current Needs**

### High Priority
1. **Windows testing** - We need more Windows users to test
2. **Mastodon instances** - Test with different Fediverse platforms
3. **Error message improvements** - Make errors more actionable
4. **Documentation examples** - Real-world usage scenarios

### Medium Priority
1. **Performance profiling** - Identify bottlenecks
2. **Integration examples** - Scripts and automation
3. **Platform adapters** - Support for new protocols
4. **Accessibility** - Screen reader compatibility

### Future Needs
1. **TUI/GUI design** - UI/UX design input
2. **Localization** - Translation to other languages
3. **Package management** - Homebrew, Chocolatey, etc.
4. **CI/CD improvements** - Better automation

## 📋 **Contribution Guidelines**

### Code Standards
- **Rust style**: Follow `cargo fmt` and `cargo clippy`
- **Tests required**: All new features need tests
- **Documentation**: Public APIs must be documented
- **Commit messages**: Use [Conventional Commits](https://www.conventionalcommits.org/)

### Pull Request Process
1. **Fork the repository**
2. **Create feature branch** from `main`
3. **Write tests** for new functionality
4. **Update documentation** if needed
5. **Run full test suite**: `cargo test`
6. **Submit PR** with clear description

### Issue Guidelines
- **Search existing issues** before creating new ones
- **Use issue templates** when available
- **Provide clear reproduction steps** for bugs
- **Include system information** (OS, Rust version, etc.)

## 🏆 **Recognition**

### Contributors
All contributors are recognized in:
- **README contributors section**
- **Release notes** for significant contributions
- **GitHub contributor stats**
- **Annual summary posts**

### Types of Recognition
- **Code contributors**: Feature development, bug fixes
- **Documentation contributors**: Guides, examples, translations
- **Community contributors**: Helping others, organizing events
- **Testing contributors**: Bug reports, platform testing

## 📚 **Learning Resources**

### For New Contributors
- **[Rust Book](https://doc.rust-lang.org/book/)** - Learn Rust basics
- **[Tokio Tutorial](https://tokio.rs/tokio/tutorial)** - Async Rust
- **[clap Documentation](https://docs.rs/clap/)** - CLI argument parsing
- **[SQLx Guide](https://github.com/launchbadge/sqlx)** - Database operations

### Project-Specific
- **[Architecture Overview](https://github.com/plurcast/plurcast/blob/main/ARCHITECTURE.md)** - System design
- **[Testing Guide](https://github.com/plurcast/plurcast/blob/main/TESTING_OVERVIEW.md)** - How to test
- **[Security Guidelines](https://github.com/plurcast/plurcast/blob/main/SECURITY.md)** - Security practices

## 🌟 **Community Values**

### Inclusivity
- **Welcoming environment** for all skill levels
- **Respectful communication** in all interactions
- **Accessibility-first** approach to features
- **Multiple contribution paths** beyond just code

### Quality
- **User experience first** - Features must be useful
- **Security-conscious** - Privacy and safety matter
- **Documentation-driven** - If it's not documented, it doesn't exist
- **Test-driven** - Features must be reliable

### Sustainability
- **Maintainable code** - Future contributors should understand it
- **Clear roadmap** - Predictable development direction
- **Community-driven** - Not dependent on any single person
- **Long-term thinking** - Decisions consider future impact

## 📈 **Project Status**

### Current Stats
- **Contributors**: {{ contributors_count | default(value="Growing community") }}
- **Stars**: ⭐ [Check GitHub](https://github.com/plurcast/plurcast)
- **Issues**: [Open Issues](https://github.com/plurcast/plurcast/issues)
- **Discussions**: [Active Discussions](https://github.com/plurcast/plurcast/discussions)

### Milestones
- **✅ v0.1.0**: Initial Nostr support
- **✅ v0.2.0**: Multi-platform support + security
- **🚧 v0.3.0**: Platform hardening (current)
- **🔮 v0.4.0**: Terminal UI
- **🔮 v1.0.0**: Stable API + semantic features

## 🎉 **Get Started Contributing**

### 1. Set Up Development Environment
```bash
# Clone and build
git clone https://github.com/plurcast/plurcast.git
cd plurcast
cargo build

# Run tests
cargo test

# Try the tools
./target/debug/plur-post --help
```

### 2. Find Your First Issue
- **[Good First Issues](https://github.com/plurcast/plurcast/labels/good%20first%20issue)** - Perfect for newcomers
- **[Help Wanted](https://github.com/plurcast/plurcast/labels/help%20wanted)** - Community assistance needed
- **[Documentation](https://github.com/plurcast/plurcast/labels/documentation)** - Writing contributions

### 3. Join the Discussion
- **[Introduce yourself](https://github.com/plurcast/plurcast/discussions/categories/general)** in GitHub Discussions
- **Ask questions** - No question is too basic
- **Share your use case** - How do you want to use Plurcast?

## 📞 **Contact**

### Maintainers
- **Primary**: Check [GitHub contributors](https://github.com/plurcast/plurcast/graphs/contributors)
- **Security**: See [SECURITY.md](https://github.com/plurcast/plurcast/blob/main/SECURITY.md)

### Community Channels
- **GitHub Discussions**: Primary communication channel
- **GitHub Issues**: Bug reports and feature requests
- **Email**: Coming soon for security issues

---

**Ready to contribute?** 

Start by [introducing yourself](https://github.com/plurcast/plurcast/discussions) and telling us how you'd like to help. Every contribution, no matter how small, makes Plurcast better for everyone.

**Building the future of decentralized social media, together.** 🚀
