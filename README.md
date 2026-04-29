# Readfence

A clean, modern Markdown viewer built for developers and Linux users. Written in Rust using the [iced](https://iced.rs) GUI library.

![Readfence screenshot placeholder](docs/screenshot.png)

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

### From source

Requires [Rust](https://rustup.rs) 1.85 or later.

```sh
git clone https://github.com/username/readfence
cd readfence
cargo build --release
./target/release/readfence
```

The binary will be at `target/release/readfence` (or `readfence.exe` on Windows).

### Linux dependencies

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

## Architecture

```
src/
└── main.rs   — application state, update logic, and view composition
```

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

## Contributing

Contributions are welcome. Please open an issue to discuss significant changes before submitting a pull request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/my-feature`)
3. Commit your changes (`git commit -m 'Add my feature'`)
4. Push to the branch (`git push origin feature/my-feature`)
5. Open a pull request

## License

MIT — see [LICENSE](LICENSE) for details.
