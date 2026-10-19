# Architecture

This document describes how rde-ide is put together: the crate graph, the
runtime loop, how a key press becomes a buffer mutation, and where new code
belongs.

## Design goals

1. **Separate state from policy from presentation.** Documents do not know
   about cursors, cursors do not know about widgets, and widgets do not know
   about the workspace.
2. **One action type.** Every user action is a `Command`; every key binding
   produces one.
3. **Small, testable units.** Domain and input logic are plain functions and
   structs with no terminal dependency, so they are unit-testable.
4. **One-way dependencies.** Higher layers depend on lower layers, never the
   reverse.

## Layers and dependencies

```
┌───────────────────────────────────────────────────────────┐
│ apps/ide  (rde-ide)                                       │
│ App, event loop, key fall-through, lifecycle              │
└───────────────┬───────────────────────────────────────────┘
                │
      ┌─────────┼──────────────┐
      ▼         ▼              ▼
┌──────────┐ ┌────────────┐ ┌────────┐
│  editor  │ │ editor-core│ │  tui   │
│ execute/ │ │ commands,  │ │ render,│
│ mutation │ │ input      │ │ view   │
└────┬─────┘ └─────┬──────┘ └───┬────┘
     │             │            │
     └──────┬──────┴────────────┘
            ▼
      ┌──────────┐
      │  domain  │ buffers, workspace, errors
      └──────────┘
```

| Crate | Path | Responsibility | Depends on |
| --- | --- | --- | --- |
| `domain` | `crates/domain` | `PieceTable`, `Buffer`, `WorkspaceState`, error types, file-tree scanning | — |
| `editor-core` | `crates/editor-core` | `Command` taxonomy, key type, keymaps, input modes, key-sequence resolver | — |
| `tui` | `crates/tui` | UI state, layout, geometry, view models, widgets, frame rendering | `domain`, `editor-core` |
| `editor` | `crates/editor` | Command dispatch, editing/motion handlers, buffer and cursor mutations | `domain`, `editor-core`, `tui` |
| `rde-ide` | `apps/ide` | Application shell, event loop, terminal setup, app-level policy | `domain`, `editor-core`, `tui`, `editor` |
| `config` | `crates/config` | Planned user configuration; not wired up yet | `editor-core` |

Rules of thumb:

- `domain` must stay free of terminal and rendering types. It currently lists
  `ratatui` as a dependency although nothing uses it; do not follow that
  precedent.
- `crates/editor-core/src/input/key.rs` is the only place that converts
  crossterm events into the editor's own `Key`. The app's event loop imports
  `ratatui::crossterm::event` to read events but passes them straight through
  `Key::from`.
- `editor` may read and mutate `tui` state (`Ui`) but must not draw.
- Only `apps/ide` decides lifecycle (start, quit) and interprets unbound keys.

## Runtime model

The app is single-threaded and synchronous. `App::run`:

1. Sets `is_running = true` and seeds the layout with the terminal size.
2. Loops while `!should_quit()`:
   - `event::poll(16ms)`; if an event is ready, read it.
     - `Event::Key` → `Key::from(event)` → `App::handle_key`.
     - `Event::Resize(w, h)` → update `ui.layout.screen_size` and recalculate
       geometry.
   - `App::tick()` — expire stale key prefixes and run any command that the
     timeout resolved.
   - Draw one frame via `tui::render::render_into`.
3. Restores the terminal in `main` on every exit path.

There is no async runtime in the loop. `tokio` is a declared dependency of
`rde-ide` but is not used yet.

## Input pipeline

```
crossterm KeyEvent
      │  Key::from (normalize modifiers)
      ▼
   Key { code, mods }
      │  Resolver::feed(mode, key, &keymap)
      ▼
 ResolveResult ── Resolved(cmd) ──> App::handle_key ──> execute(cmd)
      │
      ├─ Pending          wait for more keys
      ├─ Unhandled(keys)  fall through (text entry)
      └─ Abandoned(keys)  prefix broke; currently ignored
```

### Key normalization

`Key` wraps a small `KeyCode` enum and a bitflag `Mods`. SHIFT is folded into
the character for character keys (`Shift+w` arrives as `'W'`) and kept as a
modifier only for non-character keys. This keeps keymaps free of duplicate
shifted/unshifted bindings.

### Keymaps

`Keymap` maps `(InputMode, KeySeq) -> Command`, where a `KeySeq` is a vector of
keys (a chord). Bindings are grouped by command category in
`crates/editor-core/src/input/keymap/` and merged by `default_keymap()`.
`Keymap::is_prefix` answers whether a sequence is the beginning of a longer
binding.

### Resolver

`Resolver` is a stateful matcher with one pending sequence at a time:

- A complete binding that is also a prefix (ambiguous) is held; if the 500 ms
  `PREFIX_TIMEOUT` expires, `poll_timeout` fires the remembered exact match.
- Changing the input mode mid-sequence drops the pending state.
- A broken single-key sequence is `Unhandled` (the app turns it into text
  insertion in insert/command mode). A broken multi-key sequence is
  `Abandoned`.
- `Pending` is a `SmallVec<[Key; 4]>`, so short chords do not allocate.

## Command model

`Command` (`crates/editor-core/src/command/mod.rs`) is the single top-level
action type:

```rust
pub enum Command {
    Session(SessionCommand),
    Workspace(WorkspaceCommand),
    Motion(MotionCommand),
    Mode(ModeCommand),
    Layout(LayoutCommand),
    Edit(EditCommand),
    CommandLine(CommandLineCommand),
}
```

Each category lives in `crates/editor-core/src/command/<category>.rs` and is
re-exported from `command`.

Dispatch lives in `crates/editor/src/execute/mod.rs`:

| Command | Handler | Receives |
| --- | --- | --- |
| `Session` | `handle_session_command` | workspace |
| `Workspace` | `handle_navigation_command` | workspace, UI |
| `Motion` | `handle_motion_command` | workspace, UI |
| `Mode` | `handle_mode_command` | workspace, input, UI |
| `Layout` | `handle_layout_command` | workspace, UI |
| `Edit` | `handle_edit_command` | workspace, UI |
| `CommandLine` | `handle_command_line_command` | workspace, input, UI |

Handlers are free functions, not methods on the app. The app shell owns the
lifecycle and passes in the state a handler is allowed to touch. Handlers
return `DomainResult<()>` where they can fail; the dispatcher converts to
`IdeError` with `?`.

**Effects:** `execute` returns an `Effect` (`None` or `Quit`) that the caller
must apply. Quit commands are recognised by the dispatcher itself and never
reach `handle_session_command`; `App::dispatch` turns `Effect::Quit` into
`is_running = false`. This keeps lifecycle in the binary crate while still
letting `:q`/`:wq` from the command palette stop the application.

## Domain model

### Piece table (`domain::buffer::piece_table`)

A document is an immutable `original` string plus an append-only `add` string,
with a list of `Piece`s referencing byte ranges in one of them.

- `insert` appends the new text to `add` and splices a piece into the list,
  splitting the piece at the insertion point when needed.
- `delete` rebuilds the piece list, dropping fully covered pieces and trimming
  partially covered ones.
- `total_len` is the document length in **bytes**.
- Character-oriented queries (`chars`, `char_count`, `line`, `line_range`)
  iterate the pieces and count `char`s.

### Buffer (`domain::buffer`)

`Buffer` wraps a piece table with an `Origin` (`File { path }` or
`Scratch { name }`) and a `SaveState` (`NeverSaved`, `Clean`, `Dirty`). It
exposes:

- byte-oriented mutation: `insert(byte_offset, text)`, `delete(byte_range)`;
- character-oriented queries: `line_range`, `line_char_len`,
  `char_to_byte_offset`, `char_count`;
- persistence: `open_file`, `save`, `save_to_path`, `rename_to`.

### Workspace (`domain::workspace`)

`WorkspaceState` owns the open buffers and the active index. Invariant:
`buffers.is_empty() || active_buffer_idx < buffers.len()`. Every mutator
preserves it (`remove_buffer` clamps the index after removals). The workspace
also tracks the root directory, which the file-tree panel scans.

### Units

The single most important cross-layer rule:

| Layer | Unit |
| --- | --- |
| `EditorPanelState.cursor_line` / `cursor_col` | characters, 0-indexed |
| `PieceTable`/`Buffer` `insert`/`delete` | **bytes** |
| `Buffer::line_range` / `line_char_len` / `char_to_byte_offset` | characters |

Mutations convert at the boundary. `cursor_char_offset` in
`crates/editor/src/mutation/text.rs` maps `(cursor_line, cursor_col)` to a
linear character offset, and `char_to_byte_offset` converts that to bytes.

## UI layer

`tui::ui::Ui` is the root of UI state:

```rust
pub struct Ui {
    pub layout: Layout,     // screen size, panel visibility, focus, display mode
    pub widgets: Widgets,   // per-widget state + geometry
}
```

- `Layout` holds `screen_size: (rows, columns)` — note the order.
- `Widgets` aggregates the top bar, bottom bar, left panel, and editor panel.
  Each has a `Rect` geometry and a state struct that survives across frames.
- `tui::geometry::calc_widgets_geometry` computes those rects from the screen
  size and layout policy. It handles the normal and fullscreen modes, the top
  and bottom bars, the left panel, and the editor panel. The right panel is a
  zero-sized placeholder.

### Rendering

`tui::render::render_into`:

1. Validates the terminal size (80 × 24 minimum). On failure it renders
   `ScreenTooSmall` over the whole frame and stops.
2. Recalculates geometry from the current frame area.
3. Builds one view model per panel and renders it:
   - `render_top_bar` — tabs with active/dirty markers;
   - `render_bottom_bar` — mode badge, filename, cursor position, line count;
   - `render_left_panel` — flattened file-tree rows (top level only);
   - `render_editor_panel` — gutter, lines, current-line highlight, cursor;
   - `render_command_palette` — modal command editor, only in command mode.

### Command palette

In command mode `render/command_palette.rs` centers a 72-column card in the
middle of the screen; `widgets/command_palette.rs` draws it over the layout
with a rounded border, title, drop shadow, and key-hint footer. The widget borrows
the command text and caret from `BottomBarState` (both measured in
characters), scrolls the input horizontally so the caret stays visible, and
draws the caret as an inverted cell.

### State vs. view model

Widgets never read the workspace directly. Each frame the render layer builds
an immutable view model (`tui::viewmodel`) from domain and UI state; the widget
draws it and may mutate only its own state (for example, scroll correction in
`EditorPanelWidget::render`). `tui::state` holds the persistent side of that
split.

## Frame lifecycle

```
poll(16ms) ──> read event ──> handle_key ──> resolver ──> execute ──> mutation
   │                                                                     │
   └──> tick (prefix timeout) ────────────────────────────────────────────┘
   │
   └──> draw: render_into ──> validate size ──> geometry ──> view models ──> widgets
```

## Worked example: `:wq`

1. `:` in normal mode resolves to `ModeCommand::Enter(Command)`, which clears
   the command line and switches `input.mode`.
2. Printable keys are unbound in command mode, so the app's fall-through emits
   `CommandLine::Push(c)`. The palette redraws each frame from
   `BottomBarState`.
3. `Enter` resolves to `Submit`: the line is recorded in history, the mode
   returns to normal, and `run_ex` parses `wq`.
4. `wq` saves the buffer and returns `Effect::Quit`; `App::dispatch` sets
   `is_running = false` and the loop ends.

## Worked example: `dd`

1. The user presses `d`. `Resolver::feed` finds no exact binding but sees that
   `d` is a prefix of `dd`, so it returns `Pending`.
2. The second `d` completes the sequence and is not a prefix of anything else,
   so the resolver returns `Resolved(Command::Edit(DeleteLine))`.
3. `App::handle_key` checks for quit commands, then calls
   `editor::execute::execute`.
4. `handle_edit_command` calls `mutation::edit::delete_line`.
5. `delete_line` resolves the cursor line's character range, extends it over
   the trailing newline (or the preceding one on the last line), converts to
   byte offsets, deletes, and clamps the cursor onto a valid line.
6. The next frame renders the shortened buffer.

## Extending the editor

### Add a command

1. Add a variant to the matching enum in `crates/editor-core/src/command/`.
2. Add the match arm in the category handler under
   `crates/editor/src/execute/`.
3. Add a binding in the matching file under
   `crates/editor-core/src/input/keymap/`.
4. Add a keymap test if the binding is non-trivial (chords, shifted keys).
5. If the command needs a new reusable operation, put it in
   `crates/editor/src/mutation/` rather than in the handler.

### Add a panel or widget

1. State: `crates/tui/src/state/`.
2. View model: `crates/tui/src/viewmodel/`.
3. Widget: `crates/tui/src/widgets/`.
4. Add the widget's state and geometry to `Widgets` in
   `crates/tui/src/widgets/mod.rs`.
5. Allocate its rect in `tui::geometry` and render it from `tui::render`.
6. Update `Widgets`/`Ui` documentation headers.

### Add app-level behavior

Anything that needs the lifecycle, raw terminal events, or coordination
between layers belongs in `apps/ide/src/app.rs`, not in the lower crates.

## Error handling

Two error types in `crates/domain/src/error.rs`:

- `DomainError` / `DomainResult<T>` — library-level failures (bad offsets,
  invalid UTF-8, I/O on a buffer, missing buffer).
- `IdeError` / `Result<T>` — application boundary. It wraps `DomainError` and
  `std::io::Error`, and covers terminal-specific failures such as
  `ScreenTooSmall`.

Domain and editor functions return `DomainResult`; the app and dispatcher
return `Result`. `?` converts `DomainError` into `IdeError`.

Two intentional gaps: `handle_session_command` ignores the result of
`buffer.save()` (`let _ =`), and `render_into` discards the screen-size error
after switching to the fallback widget. Surfacing these through the status
line is planned.

## Testing

- Unit tests are inline `#[cfg(test)] mod tests` blocks at the bottom of the
  module they exercise.
- Filesystem tests use `tempfile` (`domain` dev-dependency) and never touch
  user paths.
- Current coverage is 39 tests: 17 in `domain` (workspace and buffer
  behavior), 9 in `tui` (command-line state and palette rendering), 7 in
  `editor` (ex-command parsing and quit behavior), and 6 in `editor-core`
  (keymap and mode display).
- There are no integration or TUI snapshot tests yet.

## Performance notes

- Piece-table insert/delete cost is proportional to the number of pieces
  touched, not document length.
- `chars()`, `char_count()`, `line(idx)`, and `line_range(idx)` walk the
  document; `line_range` is a linear scan from the start. Building the editor
  view calls `buffer.line(i)` for every line each frame, which is
  quadratic in document size. Rendering only the viewport and caching line
  offsets are the obvious next optimizations.
- The left panel re-scans the root directory on every frame
  (`scan_dir` in `render/left_panel.rs`). This is filesystem I/O per frame and
  should be cached and refreshed on change.
- `char_to_byte_offset` is a linear scan and runs on every edit.

## Known gaps

- The startup path is hard-coded in `App::new`.
- Buffer switching/closing through key bindings, fullscreen, the right panel,
  word/page motions, and yank/paste/undo are `todo!()`; invoking them panics.
  (`:bn`/`:bp` work through the command palette.)
- Vertical scrolling is computed in `EditorPanelState` but not applied by the
  editor widget: lines are always drawn from line 0 and the cursor is placed
  by absolute line index, so the cursor disappears once it moves below the
  viewport.
- The bottom bar always renders in status mode; `BottomBarMode::Command` is
  never selected because the command palette now owns command-line display.
- `crates/config` is an empty placeholder, and several crates declare unused
  dependencies.
