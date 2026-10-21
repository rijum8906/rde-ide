# rde-ide

A terminal-based code editor written in Rust, built around a small,
UI-agnostic core. The domain owns documents, the editor owns policy, and the
TUI only renders state.

> **Status:** early development. Basic editing, cursor motion, mode switching,
> and panel toggling work end to end. A number of commands are declared but not
> implemented yet — see [Status](#status).

## Highlights

- **Piece-table buffers** — inserts and deletes touch only the affected
  pieces, not the whole document.
- **Modal, Vim-style input** — modes, multi-key chords (`gg`, `dd`, `b n`),
  and a configurable prefix timeout, all driven by a keymap table.
- **Layered design** — command definitions, execution, domain state, and
  rendering live in separate crates with a one-way dependency graph.
- **Command palette** — press `:` for a centered, modal command editor with
  caret movement, history, and `:w`/`:q`/`:wq`-style ex commands.
- **Ratatui rendering** — tab bar, editor panel with gutter and cursor,
  file-tree side panel, and a status line.
- **Floating feedback** — stacked top-centre notifications and a top-right
  toast, each with a level (info, success, warning, error, panic), header, and
  wrapped body text. Command results such as `E492: Not an editor command`
  appear as an error toast.

## Requirements

- Rust 1.85 or newer (the workspace uses edition 2024).
- A terminal of at least 80 × 24 cells. Smaller terminals get a dedicated
  "screen too small" fallback instead of a broken layout.

## Getting started

From the repository root:

```bash
cargo run -p rde-ide
```

The application currently opens `apps/ide/Cargo.toml` as its initial buffer,
because workspace selection is not yet wired to command-line arguments (see
[Known limitations](#known-limitations)). Press `Esc` in normal mode, or
`Ctrl+Q`, to exit.

## Key bindings

Motions and edits follow Vim conventions where they are implemented.

### Motion (normal mode)

| Key | Action |
| --- | --- |
| `h` / `←` | Move left |
| `j` / `↓` | Move down |
| `k` / `↑` | Move up |
| `l` / `→` | Move right |
| `0` | Start of line |
| `$` | End of line |
| `w` | Start of the next word |
| `b` | Start of the previous word |
| `e` | End of the current or next word |
| `W` | Start of the next big word (whitespace-delimited) |
| `B` | Start of the previous big word |
| `E` | End of the current or next big word |
| `gg` | Start of file |
| `G` | End of file |

Arrow keys also move the cursor in insert mode.

### Modes

| Key | Action |
| --- | --- |
| `i` | Enter insert mode |
| `a` | Move right, then enter insert mode |
| `A` | Move to end of line, then insert |
| `I` | Move to start of line, then insert |
| `o` | Open a new line below, then insert |
| `O` | Open a new line above, then insert |
| `v` | Enter visual mode |
| `:` | Enter command mode |
| `Esc` | Leave insert/visual mode (normal mode: quit) |
| `Ctrl+J`, `Ctrl+K` | Leave insert mode |

### Editing

| Key | Action |
| --- | --- |
| printable char | Insert at the cursor (insert mode) |
| `Enter` | Insert a newline (insert mode) |
| `Backspace` | Delete the character before the cursor (insert mode) |
| `dd` | Delete the current line (normal mode) |

### Command mode

Press `:` in normal mode to open the command palette. It floats in the middle
of the layout.

| Key | Action |
| --- | --- |
| printable char | Insert into the command line |
| `←` / `→` | Move the caret |
| `Home` / `End` | Jump to the start / end of the line |
| `↑` / `↓` | Browse command history |
| `Backspace` | Delete before the caret |
| `Ctrl+U` | Clear the line |
| `Enter` | Run the command |
| `Esc` | Cancel |

Supported ex commands:

| Command | Action |
| --- | --- |
| `:w [path]` | Write the active buffer, optionally to a new path |
| `:q` | Quit when nothing is unsaved |
| `:q!` | Quit regardless of unsaved changes |
| `:wq`, `:x`, `:exit` | Write, then quit |
| `:e <path>` | Open a file relative to the workspace root |
| `:bn`, `:bp` | Switch to the next / previous buffer |

### Layout and session

| Key | Action |
| --- | --- |
| `Ctrl+B` or `Space` then `e` | Toggle the left (file tree) panel |
| `Ctrl+S` | Save the active buffer |
| `Esc` | Quit (normal mode) |
| `Ctrl+Q`, `Ctrl+C` | Quit immediately |

Additional bindings are declared in
[`crates/editor-core/src/input/keymap/`](crates/editor-core/src/input/keymap)
(buffer switching, fullscreen, and so on). Many of their handlers are still
`todo!()`; see [Status](#status).

## Status

Implemented end to end:

- Opening a buffer at startup and rendering it with a gutter, cursor, and
  current-line highlight.
- Insert, newline, backspace, and delete-line edits.
- Single-step, line, file, and word (`w`/`b`/`e`) and big-word (`W`/`B`/`E`)
  motions.
- Normal/insert/visual/command mode transitions, including the edits implied
  by `a`, `A`, `I`, `o`, and `O`.
- A floating command palette with caret editing, history browsing, and the
  `:w`, `:q`, `:q!`, `:wq`, `:e`, `:bn`, and `:bp` ex commands.
- Left panel toggle, saving a buffer, and quitting.
- Multi-key chord resolution with a 500 ms timeout and prefix abandonment.

Scaffolded but not implemented — invoking these currently panics with
`todo!()`:

- Buffer switching and closing (`Tab`, `Ctrl+W`, `b n`/`b p`/`b d`/`b D`).
- Fullscreen mode, the right panel, and panel focus.
- Paragraph, page, and bracket motions.
- Undo/redo (`u`, `Ctrl+R`).
- Yank/paste and motion-prefixed changes (`d{motion}`, `c{motion}`,
  `y{motion}`).
- Save-as, save-all, open-file, and new-scratch-buffer prompts.

## Project layout

```
.
├── apps/
│   └── ide/           # rde-ide: App shell, event loop, binary
├── crates/
│   ├── config/        # planned user configuration (placeholder)
│   ├── domain/        # piece-table buffers, workspaces, errors
│   ├── editor-core/   # commands, keys, keymaps, modes, resolver
│   ├── editor/        # command execution and buffer/cursor mutations
│   └── tui/           # layout, geometry, view models, widgets, rendering
├── ARCHITECTURE.md    # crate responsibilities and data flow
└── AGENTS.md          # contributor and AI-agent guidelines
```

## Development

```bash
cargo build --workspace                          # build everything
cargo test --workspace                           # unit tests
cargo fmt --all -- --check                       # formatting
cargo clippy --workspace --all-targets -- -D warnings
cargo doc --workspace --no-deps --open           # API documentation
```

There is no CI configuration yet, so these commands are the gate. See
[`AGENTS.md`](AGENTS.md) for conventions and
[`ARCHITECTURE.md`](ARCHITECTURE.md) for the design.

## Known limitations

- The startup workspace path is hard-coded in `App::new`; there is no CLI.
- Many commands panic because their handlers are `todo!()` (see
  [Status](#status)).
- `crates/config` is an empty placeholder and is not used by the binary.
- Some crates declare dependencies they do not use yet (`ratatui` in
  `domain`, `tokio` in `rde-ide`, `color-eyre` in `tui`/`editor`).

## License

MIT. See the `license` field in the workspace [`Cargo.toml`](Cargo.toml).
