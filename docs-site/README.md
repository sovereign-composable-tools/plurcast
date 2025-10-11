# Plurcast Documentation Site

This is the **comprehensive documentation and marketing site** for Plurcast, built with [Zola](https://www.getzola.org/) (a fast static site generator written in Rust).

## 🎯 **Purpose**

This site serves as:
- **Introduction** to Plurcast's philosophy and approach
- **Complete documentation** for installation and usage
- **Roadmap** showing current status and future vision
- **Community hub** for contributors and users
- **Living wiki** that evolves with the project

## 🏗️ **Architecture**

### Technology Stack
- **[Zola](https://www.getzola.org/)** - Rust-based static site generator
- **Custom theme** - Terminal-inspired design
- **Markdown content** - Easy to edit and maintain
- **No JavaScript dependencies** - Fast loading, accessible

### Design Philosophy
- **Terminal aesthetic** - Monospace fonts, dark theme, command-line feel
- **Developer-focused** - Code examples, technical depth
- **Unix-inspired** - Simple, composable, focused
- **Accessible** - Works without JavaScript, screen reader friendly

## 📁 **Structure**

```
docs-site/
├── config.toml           # Zola configuration
├── content/              # Markdown content
│   ├── _index.md        # Homepage
│   ├── philosophy/      # Unix philosophy and design principles
│   ├── getting-started/ # Installation and setup guides
│   ├── roadmap/         # Development roadmap and vision
│   └── community/       # Contribution guidelines and community
├── templates/           # Zola templates
│   ├── base.html       # Base template with navigation
│   ├── index.html      # Homepage template (unused)
│   ├── section.html    # Section template for main pages
│   └── page.html       # Page template for content pages
├── public/             # Generated site (gitignored)
└── README.md           # This file
```

## 🚀 **Development**

### Prerequisites
- **[Zola](https://www.getzola.org/documentation/getting-started/installation/)** - Static site generator
  ```bash
  # Windows (with winget)
  winget install getzola.zola
  
  # macOS (with Homebrew)
  brew install zola
  
  # Linux (with package manager or download binary)
  # See: https://github.com/getzola/zola/releases
  ```

### Local Development

```bash
# Navigate to docs directory
cd docs-site

# Serve locally with live reload
zola serve

# Open in browser
# http://127.0.0.1:1111 (default port)
```

### Building

```bash
# Check for errors
zola check

# Build production site
zola build

# Output goes to public/ directory
```

### Content Editing

All content is in **Markdown** with **TOML front matter**:

```markdown
+++
title = "Page Title"
description = "Page description for SEO"
weight = 1          # Order in navigation
template = "section.html"  # Optional template override
+++

# Page Content

Your markdown content here...
```

## 🎨 **Design System**

### Color Palette
```css
:root {
    --bg-dark: #0d1117;        /* Main background */
    --bg-light: #21262d;       /* Cards, header */
    --bg-lighter: #30363d;     /* Code blocks */
    --text-primary: #c9d1d9;   /* Main text */
    --text-secondary: #8b949e; /* Secondary text */
    --accent-primary: #58a6ff;  /* Links, brand */
    --success: #56d364;        /* Success states */
    --warning: #e3b341;        /* Warnings */
    --error: #f85149;          /* Errors */
}
```

### Typography
- **Headings**: System fonts (Inter)
- **Body**: System fonts (Inter)
- **Code**: Monospace (JetBrains Mono, Fira Code, etc.)

### Components
- **Terminal windows**: Code examples with headers
- **Feature cards**: Grid layout for features
- **Navigation**: Sticky header with site-wide nav
- **Buttons**: Primary and secondary styles

## 📝 **Content Guidelines**

### Writing Style
- **Clear and concise** - Unix tools aesthetic
- **Technical depth** - Developers are the primary audience  
- **Examples-heavy** - Show, don't just tell
- **Actionable** - Every guide should be immediately usable

### Code Examples
```bash
# Always include comments for context
echo "Hello world" | plur-post

# Show expected output when helpful
# Output: nostr:note1abc123...

# Use realistic examples
fortune | plur-post --platform nostr
```

### Sections
Each major section should include:
- **Clear purpose** - What will the reader learn?
- **Prerequisites** - What do they need to know first?
- **Step-by-step instructions** - Easy to follow
- **Examples** - Working code they can copy
- **Next steps** - Where to go from here

## 🔧 **Configuration**

### Site Settings
Key configuration in `config.toml`:

```toml
base_url = "https://plurcast.github.io"  # Update for deployment
title = "Plurcast - Unix Tools for the Decentralized Web"
description = "Documentation and philosophy"

# Menu items
[[extra.menu]]
url = "/"
name = "Home"
# ... more menu items
```

### Features
- **Search**: Built-in search index
- **Syntax highlighting**: Code blocks with theme
- **RSS feeds**: Automatic generation
- **SEO**: Meta tags, Open Graph

## 🚀 **Deployment**

### GitHub Pages (Recommended)

Create `.github/workflows/deploy.yml`:

```yaml
name: Deploy Documentation

on:
  push:
    branches: [main]
    paths: [docs-site/**]

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup Zola
        uses: taiki-e/install-action@zola
        
      - name: Build
        run: |
          cd docs-site
          zola build
          
      - name: Deploy
        uses: peaceiris/actions-gh-pages@v3
        with:
          github_token: ${{ secrets.GITHUB_TOKEN }}
          publish_dir: ./docs-site/public
```

### Alternative Deployment Options
- **Netlify**: Connect GitHub repo, set build command `cd docs-site && zola build`
- **Vercel**: Similar setup with Zola
- **Self-hosted**: Build locally, deploy `public/` directory

## 📊 **Content Status**

### Completed Pages ✅
- **Homepage** (`/`) - Project introduction and overview
- **Philosophy** (`/philosophy/`) - Unix principles and design philosophy  
- **Getting Started** (`/getting-started/`) - Installation and setup guide
- **Roadmap** (`/roadmap/`) - Development phases and future vision
- **Community** (`/community/`) - Contribution guidelines and community

### Planned Pages 🚧
- **Documentation** (`/documentation/`) - Complete CLI reference
- **Examples** (`/examples/`) - Real-world usage scenarios
- **API Reference** (`/api/`) - Library documentation (future)
- **Blog** (`/blog/`) - Project updates and deep dives (future)

### Future Enhancements 🔮
- **Interactive terminal demos** - Asciinema integration
- **Search functionality** - Already configured, needs content
- **Multi-language support** - Internationalization
- **Dark/light theme toggle** - Currently dark-only

## 🎨 **Visual Identity**

### Design Inspiration
- **GitHub's dark theme** - Professional, developer-focused
- **Terminal applications** - Monospace, command-line aesthetic
- **Unix documentation** - Clear, technical, no-nonsense

### Brand Elements
- **Logo**: Text-based "plurcast" in monospace
- **Colors**: Dark theme with blue accents
- **Typography**: Mix of system fonts and monospace
- **Voice**: Technical, clear, Unix-philosophy inspired

## 🤝 **Contributing to Docs**

### Quick Edits
1. **Edit markdown files** in `content/` directory
2. **Test locally** with `zola serve`
3. **Submit PR** with changes

### New Sections
1. **Create directory** in `content/`
2. **Add `_index.md`** with front matter
3. **Update navigation** in `config.toml` if needed
4. **Test and submit PR**

### Style Guide
- **Headings**: Use sentence case ("Getting started" not "Getting Started")
- **Code blocks**: Always specify language for syntax highlighting
- **Links**: Use descriptive text, not "click here"
- **Examples**: Prefer real-world scenarios over toy examples

## 📈 **Analytics & Performance**

### Performance Targets
- **Load time**: < 1 second on 3G
- **Lighthouse score**: > 95 on all metrics
- **Bundle size**: < 100KB per page
- **Accessibility**: WCAG AA compliant

### Monitoring
- **No analytics** by default (privacy-first)
- **GitHub Pages insights** for basic metrics
- **Optional analytics** via configuration for serious deployments

## 🔧 **Maintenance**

### Regular Tasks
- **Update content** as features are added
- **Fix broken links** after releases
- **Refresh examples** to match current syntax
- **Update roadmap** progress

### Version Alignment
- **Keep in sync** with main project releases
- **Update version numbers** in config and content
- **Refresh screenshots** and examples after UI changes

## 🎯 **Success Metrics**

### User Success
- **Time to first post** - Can new users post within 5 minutes?
- **Question reduction** - Fewer "how do I..." issues on GitHub
- **Community growth** - More contributors and discussions

### Content Success
- **Clarity** - Users find what they need quickly
- **Completeness** - All major features documented
- **Accuracy** - Examples work as written
- **Discoverability** - Good search and navigation

---

## 🚀 **Getting Started**

Ready to work on the documentation?

```bash
# 1. Install Zola
winget install getzola.zola  # Windows
# or brew install zola       # macOS
# or see zola.org for Linux

# 2. Start development server
cd docs-site
zola serve

# 3. Open browser
# http://127.0.0.1:1111

# 4. Edit content in content/ directory
# Changes auto-reload in browser

# 5. Build for production
zola build
```

**Questions?** Open an issue or discussion on the main [Plurcast repository](https://github.com/plurcast/plurcast).

**Contributing?** See the [Community page](/community/) for guidelines.

---

**This documentation site grows with the project.** As Plurcast evolves from alpha to stable, this site will become the definitive resource for users, contributors, and anyone interested in Unix-style decentralized social media tools.
