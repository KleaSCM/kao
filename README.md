# Kao

> A fast, keyboard-driven desktop app for finding and copying your favorite kaomojis and text emoticons (ᵔᴥᵔ)

![License](https://img.shields.io/badge/license-MIT-pink)
![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey)
![Tauri](https://img.shields.io/badge/built%20with-Tauri-24C8DB)

---

## Why This Exists

got borde built this `(づ￣ ³￣)づ`?

**Kaomoji Picker** gives you instant, offline access to hundreds of kaomojis with:

- **Lightning-fast fuzzy search** - Find exactly what you need in milliseconds
- **Full keyboard navigation** - Never touch your mouse
- **Persistent favorites** - Add custom kaomojis that sync across sessions
- **Smart tagging** - Search by emotion, category, or character
- **One-click copying** - Straight to your clipboard, ready to paste

---

## Demo

[GIF/Screenshot placeholder - showing search, keyboard nav, and copy action]

---

## Features

### Powerful Search

```
Search box → "happy"        → (◕‿◕)  (ﾉ◕ヮ◕)ﾉ*:･ﾟ✧  ヽ(♡‿♡)ノ
           → "cat:Joy"      → All kaomojis in Joy category
           → "tag:blush"    → All tagged with "blush"
           → "cat:Love tag:kiss" → Combine filters!
```

### Global Shortcuts

| Key | Action |
|-----|--------|
| `Ctrl + Shift + Space` | Toggle app visibility (Global) |
| `Arrow Keys` | Navigate through results |
| `Enter` | Copy selected kaomoji |
| `Escape` | Clear search / Hide app |
| `Type to search` | Instant filtering |

### Customization & Enhancements

- **Auto-Paste Mode** - Instantly paste copied kaomojis into your active app
- **Global Hotkey** - Access Kao from anywhere with `Ctrl+Shift+Space`
- **Favorites System** - Star your most-used kaomojis for quick access
- **Add your own kaomojis** - With custom tags and categories
- **Persistent History** - Remembers your last 20 copied items across restarts
- **Persistent Storage** - Your custom entries and favorites are saved locally and privately

### Categories Included

- **Joy** - Happy, excited, celebratory
- **Love** - Hearts, affection, romance
- **Sadness** - Crying, disappointed, worried
- **Angry** - Mad, frustrated, annoyed
- **Animals** - Cats, dogs, bears, and more
- **Decoration** - Borders, dividers, sparkles
- **Large (ASCII Art)** - Multi-line text art

---

## Installation

### Download Pre-built Binaries

Grab the latest release for your platform from the [Releases](https://github.com/KleaSCM/kao/releases) page:

- **Windows**: `.msi` installer
- **macOS**: `.dmg` or `.app`
- **Linux**: `.AppImage` or `.deb`

### Build from Source

```bash
# Prerequisites: Rust + Node.js + pnpm

# Clone the repo
git clone https://github.com/kleascm/kao.git
cd kaomoji-picker

# Install dependencies
pnpm install

# Run in dev mode
pnpm tauri dev

# Build for production
pnpm tauri build
```

---

## Architecture

Built with modern, performant technologies:

**Frontend**

- **Svelte 5** - Reactive UI with runes (`$state`, `$derived`)
- **Fuse.js** - Fuzzy search engine
- **TypeScript** - Type safety

**Backend**

- **Rust** - Native performance and safety
- **Tauri** - Lightweight desktop framework (~3MB binary!)
- **Atomic file operations** - Corruption-resistant data persistence

```
┌─────────────────────────────────────────────┐
│              Frontend (Svelte)              │
│  • Fuzzy Search (Fuse.js)                   │
│  • Keyboard Navigation                      │
│  • Reactive State Management                │
└──────────────────┬──────────────────────────┘
                   │ IPC Commands
┌──────────────────▼──────────────────────────┐
│              Backend (Rust)                 │
│  • Clipboard Management                     │
│  • Persistent Storage (JSON)                │
│  • Atomic File Writes                       │
│  • Cross-platform Compatibility             │
└─────────────────────────────────────────────┘
```

---

## Usage Tips

### Search Syntax

- **Simple search**: Just type what you're looking for
  - `happy` → finds all happy-related kaomojis
- **Category filter**: `cat:CategoryName`
  - `cat:Animals` → only show animals
- **Tag filter**: `tag:tagname`
  - `tag:cute` → only show items tagged "cute"
- **Combine filters**: `cat:Joy tag:blush happy`
  - Filters by category AND tag AND fuzzy search!

### Adding Custom Kaomojis

1. Click the **＋** button
2. Paste your kaomoji in the Character field
3. Add comma-separated tags (e.g., `happy, excited, party`)
4. Choose a category
5. Click **Save**

Your custom entries are stored locally at:

- **Windows**: `%APPDATA%\kao\kaomojis.user.json`
- **macOS**: `~/Library/Application Support/kao/kaomojis.user.json`
- **Linux**: `~/.local/share/kao/kaomojis.user.json`

---

## Data Safety

Your data is precious! We use **atomic file operations** to prevent corruption:

- ✅ Writes to temporary files first
- ✅ Uses platform-specific atomic operations (ReplaceFileW on Windows, atomic rename on Unix)
- ✅ Automatic corruption detection and backup
- ✅ All data stored locally - no cloud, no tracking

If corruption is detected, the app automatically backs up the corrupted file with a timestamp before recovering.

---

### Development Setup

```bash
# Fork and clone the repo
git clone https://github.com/kleascm/kao.git

# Install dependencies
pnpm install

# Run in development mode with hot reload
pnpm tauri dev

# Run tests (if applicable)
pnpm test
```

### Code Style

- **Rust**: Follow standard Rust conventions
- **TypeScript/Svelte**: Prettier + ESLint
- **Comments**: Document the "why" behind complex logic
- **Commits**: Clear, descriptive messages

---

## Tech Stack

| Layer | Technology | Why? |
|-------|-----------|------|
| Framework | Tauri | Native performance, tiny bundle size (~3MB) |
| Frontend | Svelte 5 | Reactive, minimal runtime overhead |
| Backend | Rust | Memory safety, zero-cost abstractions |
| Search | Fuse.js | Fast fuzzy search with configurable scoring |
| Storage | JSON | Human-readable, easy to backup/edit |
| Build | pnpm + Vite | Fast builds, efficient dependency management |

---

## License

MIT License - see [LICENSE](LICENSE) for details

---

<div align="center">

**[Download](https://github.com/kleascm/kao/releases)** • **[Report Bug](https://github.com/kleascm/kao/issues)** • **[Request Feature](https://github.com/kleascm/kao/issues)**

(づ￣ ³￣)づ

</div>
