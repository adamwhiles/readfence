# Changelog

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
