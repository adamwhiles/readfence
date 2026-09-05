# Changelog

## 0.5.0 - 2026-09-04

- Tables now render as a real grid of cells instead of monospace text, so columns stay aligned no matter which glyphs (emoji, symbols, fallback fonts) the cells contain; inline bold, italic, code, links, and strikethrough render inside cells, and links in cells are clickable
- Readfence remembers your session: the documents you had open, which one was active, the font size, the sidebar and view mode, and the window size all come back on the next launch (files passed on the command line take precedence)
- New Outline section in the sidebar lists the document's headings; click one to jump to that section
- Find in document: `Ctrl+F` opens a search bar with a match count, `Enter`/`Shift+Enter` and `F3`/`Shift+F3` step through hits, `Esc` closes it; hits are highlighted in both the rendered and source views
- Zoom with `Ctrl+scroll`; `Ctrl+0` resets the text size
- Copying is more forgiving: `Ctrl+C` and `Ctrl+A` work in the reading view even when no block has keyboard focus, select-all now shows the selection in every block, clicking a block clears stale selections elsewhere, copied list items and quote lines keep their line breaks, and the status bar confirms what was copied
- `Esc` also closes the About menu

## 0.4.2 - 2026-08-26

- Readfence can now install updates in place: the update banner's Install button downloads the new release, verifies its checksum, swaps the executable, and offers a one-click restart that reopens your documents
- New About menu in the toolbar with the installed version, manual update checks, and a link to the GitHub page
- Flatpak and other store-managed installs continue to update through the store; the in-app installer is hidden there

## 0.4.1 - 2026-08-25

- The reading view now fills the width of the window instead of being capped at a narrow column
- Wide tables and code blocks scroll horizontally instead of being cut off at the edge of the page

## 0.4.0 - 2026-07-18

The biggest Readfence release yet: a full rendering overhaul, image support, a curated theme system, and update notifications.

### Rendering

- **Tables** render as aligned columns with bold headers, honoring `:---:` / `---:` alignment, instead of collapsing into misaligned plain text
- **Inline styling** now renders: `**bold**`, `*italic*`, inline `code`, and `~~strikethrough~~` inside any block, including list items, quotes, and table cells
- **Lists** get real bullets (`•`/`◦`/`▪` by nesting depth), accent-colored markers, correct nesting, and tighter item spacing; task lists show proper checkboxes
- **Blockquotes** carry an accent bar over a soft tint; GitHub-style alerts (`> [!NOTE]` etc.) get a bold accent label
- **Headings** are bold with a clear size ramp; H1/H2 get hairline underlines
- Comfortable reading line-height, a capped text measure on wide windows, and a tuned vertical rhythm throughout
- Fixed: nested lists losing the parent item's text, stray indentation in table rows, YAML front matter leaking into documents

### Images

- Local images referenced by relative or absolute path render in place at natural size, never upscaled
- Remote images download asynchronously and swap in when ready
- SVG support, including README badges at their natural size
- Broken references degrade to a quiet placeholder

### Themes and app polish

- Theme list curated to fifteen palettes that all render well; low-quality combinations removed
- Light themes now read like proper documents: white page on a grey canvas with softer shadows
- Your theme choice is saved and restored on the next launch; first launch defaults to Moonfly
- Styled theme picker, slim rounded scrollbars, and a responsive toolbar that adapts down to small windows (with a sensible minimum window size)

### Updates

- Readfence now checks GitHub releases at launch and every six hours, showing a quiet banner with a one-click jump to the download page when a newer version is available
- Dismissing a notice silences that version for the session; Flatpak installs rely on the store and skip the check

## 0.3.4 - 2026-07-09

- Open files passed on the command line or via a file-manager "Open with" action
- Register a desktop entry and icon, and associate Markdown files with Readfence
- Initial Flatpak packaging

## 0.3.3 - 2026-07-08

- Reworked the Markdown rendering pipeline
- Improved the visual presentation of rendered documents
