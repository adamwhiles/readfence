[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Latest Release](https://img.shields.io/github/v/release/adamwhiles/readfence)](https://github.com/adamwhiles/readfence/releases/latest)
[![Build](https://img.shields.io/github/actions/workflow/status/adamwhiles/readfence/release.yml?label=build)](https://github.com/adamwhiles/readfence/actions)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-blue)](https://github.com/adamwhiles/readfence/releases/latest)

# Readfence

A clean, modern Markdown viewer built for developers and Linux users. Written in Rust using the [iced](https://iced.rs) GUI library.

![Readfence screenshot placeholder](https://readfence.com/screenshot.png)

## Features

- **Rendered Markdown by default** — beautifully formatted output with heading hierarchy, code blocks, inline code, bold, italic, links, and blockquotes
- **Source view** — toggle to see raw Markdown at any time
- **Multi-file sidebar** — open multiple files and switch between them; hide the sidebar for a distraction-free reading experience
- **Fullscreen / maximized mode** — one click or `F11` for focused reading
- **Adjustable font size** — increase or decrease with `Ctrl+=` / `Ctrl+-`
- **Rich theme library** — powered by iced's built-in theme system: Dark, Light, Dracula, Nord, Gruvbox, Solarized, Tokyo Night, Catppuccin, Oxocarbon, and more
- **Cross-platform** — Windows, Linux, and macOS
- **Clickable links** — opens URLs in your default system browser
- **Keyboard shortcuts** — full keyboard control for power users

## Installation

Prebuilt binaries and installers are published on the [GitHub Releases](https://github.com/adamwhiles/readfence/releases/latest) page.

### Windows

Windows releases currently target `x86_64-pc-windows-msvc`.

- Download `readfence-x86_64-pc-windows-msvc.msi` from the latest release and run the installer.
- If you prefer a portable build, download `readfence-x86_64-pc-windows-msvc.zip` and run `readfence.exe` directly.

The MSI is built in the release pipeline with WiX and includes normal Windows installer behavior such as add/remove programs integration.

### macOS

macOS releases are built for both Apple Silicon (`aarch64-apple-darwin`) and Intel (`x86_64-apple-darwin`).

Install the latest release with the generated shell installer:

```sh
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/adamwhiles/readfence/releases/latest/download/readfence-installer.sh | sh
```

By default, the installer places `readfence` in `$CARGO_HOME/bin`, or `~/.cargo/bin` if `CARGO_HOME` is not set.

### Linux

Linux releases are built for `x86_64-unknown-linux-gnu` and `aarch64-unknown-linux-gnu`.

Install the latest release with the generated shell installer:

```sh
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/adamwhiles/readfence/releases/latest/download/readfence-installer.sh | sh
```

By default, the installer places `readfence` in `$CARGO_HOME/bin`, or `~/.cargo/bin` if `CARGO_HOME` is not set.

### From source

Requires [Rust](https://rustup.rs) 1.85 or later.

```sh
git clone https://github.com/adamwhiles/readfence
cd readfence
cargo build --release
./target/release/readfence
```

The binary will be at `target/release/readfence` (or `readfence.exe` on Windows).

### Linux build dependencies

On Linux, `rfd` (the file dialog library) requires GTK 3 development headers:

```sh
# Debian / Ubuntu
sudo apt install libgtk-3-dev

# Fedora
sudo dnf install gtk3-devel

# Arch Linux
sudo pacman -S gtk3
```

## Usage

Launch Readfence and use the **Open** button (or `Ctrl+O`) to open one or more Markdown files.

### Keyboard shortcuts

| Shortcut      | Action                    |
|---------------|---------------------------|
| `Ctrl+O`      | Open file(s)              |
| `Ctrl+B`      | Toggle sidebar            |
| `F11`         | Toggle maximize / restore |
| `Ctrl+=`      | Increase font size        |
| `Ctrl+-`      | Decrease font size        |

### Themes

Select a theme from the dropdown in the toolbar. Available themes include:

- **Dark** / **Light** — iced defaults
- **Dracula** — the popular dark theme
- **Nord** — arctic, north-bluish colour palette
- **Gruvbox Dark** — retro groove dark theme
- **Solarized Dark** — precision colours for machines and people
- **Tokyo Night** — a clean dark theme inspired by Tokyo at night
- **Catppuccin Mocha** — soothing pastel dark theme
- **Oxocarbon** — IBM carbon design inspired

Built with the [Elm architecture](https://guide.elm-lang.org/architecture/) pattern as implemented by iced:

- **State** — `App` struct holds all application state
- **Messages** — typed enum describes every possible event
- **Update** — pure state transitions per message
- **View** — declarative UI derived from state

### Dependencies

| Crate   | Purpose                          |
|---------|----------------------------------|
| `iced`  | Cross-platform GUI framework     |
| `rfd`   | Native async file dialogs        |
| `open`  | Open URLs in the system browser  |
| `tokio` | Async file I/O                   |

## Roadmap

### Recently shipped

| Feature | Description |
|---|---|
| ✅ **Auto-reload on file change** | Watches open files and reloads instantly when saved — live preview alongside your editor |

### Up next

| Feature | Description |
|---|---|
| **Drag and drop to open** | Drop any `.md` file onto the window to open it without using the file dialog |
| **Recent files** | Quickly reopen previously viewed files from a persistent recent files list |
| **Zoom with Ctrl+scroll** | Scale font size with the scroll wheel while holding Ctrl |

### Planned

| Feature | Description |
|---|---|
| **Table of contents panel** | Auto-generated heading outline in the sidebar with jump-to-section navigation |
| **Find in document** | Ctrl+F search bar that highlights matches in rendered or source view |
| **Local image rendering** | Render images referenced by relative path so `![](./img.png)` works as expected |
| **Persistent settings** | Remember window size, theme, font size, and last open files across restarts |

### On the radar

| Feature | Description |
|---|---|
| **Open folder** | Open an entire directory and browse all Markdown files from the sidebar |
| **YAML front matter support** | Detect and display front matter (Hugo, Jekyll, Obsidian) cleanly instead of as raw text |
| **Word count & reading time** | Status bar showing word count and estimated reading time for the open document |

Have a feature request? [Open an issue](https://github.com/adamwhiles/readfence/issues) to discuss it.

## Contributing

Contributions are welcome. Please open an issue to discuss significant changes before submitting a pull request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/my-feature`)
3. Commit your changes (`git commit -m 'Add my feature'`)
4. Push to the branch (`git push origin feature/my-feature`)
5. Open a pull request

## License

MIT — see [LICENSE](LICENSE) for details.
