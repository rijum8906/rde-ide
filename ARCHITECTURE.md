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

### Piece table (`domain::piece_table`)

A document is an immutable `original_text` string plus an append-only
`additive_text` string, with a flat `Vec<Piece>` referencing byte ranges in one
of them. Concatenating the pieces in order renders the document.

- `insert(text, byte_index)` appends the text to the additive buffer and
  splices a new additive piece in at the offset, splitting the piece that
  contains it. Appends at EOF merge into the previous additive piece, so a
  typed run stays a single piece.
- `delete(byte_range)` rebuilds the piece list into a reusable scratch vector:
  pieces fully covered by the range are dropped, boundary pieces are trimmed,
  and surviving neighbours in the same source merge. Both ends must be on
  UTF-8 char boundaries; a range that would split a character is rejected with
  `TextError::InvalidDeleteRange` before any mutation.
- Byte length is cached; line breaks (`line_breaks`) are rebuilt from the
  pieces on first read after an edit and cached in a `OnceCell`, so no stale
  list is observable. `line_count` is `line_breaks.len() + 1`, which makes an
  empty document one empty line.
- `line_range`, `line`, and `line_col_at` return byte ranges, line text, and
  a `(line, column)` position. `slice`, `text`, `char_count`,
  `char_to_byte_offset`, and `byte_to_char_offset` walk the pieces and are
  O(document). Word helpers (`next_word_range`, `prev_word_range`, the `big`
  variants, and the start/end conveniences on `Buffer`) locate the starting
  piece once, then scan its slices in order, so their cost is proportional to
  the bytes walked rather than to a piece lookup per character.

`replace_all` drops the piece list and adopts new text as a single original
piece. There is no undo/redo history.

### Buffer (`domain::buffer`)

`Buffer` wraps a `PieceTable` with an `Origin` (`File { path }` or
`Scratch { name }`) and a `SaveState` (`NeverSaved`, `Clean`, `Dirty`). Its
implementation is split like the piece table's: `insert`, `delete`, `line`,
and `word` submodules, with construction, metadata, and persistence in
`mod.rs`. It exposes:

- byte-oriented mutation: `insert(byte_offset, text)`, `delete(byte_range)`,
  `replace_all(text)`;
- character-oriented queries: `line_range`, `line`, `line_count`,
  `line_char_len`, `char_to_byte_offset`, `is_empty`;
- word motions in byte offsets: `next_word_start`/`next_word_end`,
  `prev_word_start`/`prev_word_end`, the `big` variants (`W`/`E`/`B`/`gE`),
  and the underlying `*_range` methods;
- persistence: `open_file`, `save`, `save_to_path`, `rename_to`.

`line_range` converts the piece table's byte range to characters, because the
editor cursor is character-based. Word `*_end` motions return the byte offset
of the word's last character, so a cursor can land on it; `*_range` keeps the
exclusive range. Any content change marks the buffer dirty; only a successful
save marks it clean.

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
   - `render_editor_panel` — gutter, lines, current-line highlight, cursor.
     It corrects the scroll offset and builds only the lines in the viewport; the
     cursor's screen row is relative to `scroll_y`.
   - `render_command_palette` — modal command editor, only in command mode.

Notifications and toasts are drawn by the app shell rather than
`render_into`: `App::render` draws the layout first, then the notification
stack in the top-centre (when `notifications.visible()` is non-empty), then
the toast in the top-right (when `ui.widgets.toast.state.visible()` is some).
`render_notifications` returns the row just past the last card, and the toast
starts one row below it so the two never overlap. The bottom bar no longer
carries messages.

### Command palette

In command mode `render/command_palette.rs` centers a 72-column card in the
middle of the screen; `widgets/command_palette.rs` draws it over the layout
with a rounded border, title, drop shadow, and key-hint footer. The widget borrows
the command text and caret from `BottomBarState` (both measured in
characters), scrolls the input horizontally so the caret stays visible, and
draws the caret as an inverted cell.

### Notifications

`editor::notification` owns the data: a `Notification` pairs a `Level` with a
`NotificationBody` (header + body) and a TTL. `Notifications` keeps the newest
`DEFAULT_CAPACITY` items, drops expired ones in `tick`, and returns the live
ones from `visible()`. The module converts each notification into the TUI's
`NotificationView` (borrowed, so rendering copies no text).

`tui::render::notification::render_notifications` reserves a top-centre
region (44 columns max, 2-cell margin) and `widgets/notification.rs` stacks
rounded cards there: a coloured icon and bold header with a right-aligned
level label, and a wrapped body clamped to three lines with an ellipsis.
Cards that do not fit are skipped.

### Toasts

A toast is the transient result of a command (`:w` succeeded, `:q` refused, an
unknown ex command). The data lives in `tui::state::toast`: `ToastState` holds
at most one `ToastMessage` with a `Level`, header, body, and TTL; a new toast
replaces the old one, and `App::tick` calls `poll_expiry`. Command handlers
call `ui.widgets.toast.state.error(header, body)` (or `info`/`success`/
`warning`) instead of setting a status-line message.

`tui::render::toast::render_toast` measures the card (3 rows plus up to two
body lines) and anchors it in the top-right corner, two columns from the edge
and two rows from the top, or one row below the notification stack when one
is visible. `widgets/toast.rs` renders through the same `widgets/card.rs` used
by notifications, so both share the border, level icon/label, and wrapping
code; toasts just keep their body to two lines instead of three.

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

Two error types in `crates/domain/src/error/mod.rs`:

- `DomainError` / `DomainResult<T>` — library-level failures (bad offsets,
  invalid UTF-8, I/O on a buffer, missing buffer).
- `IdeError` / `Result<T>` — application boundary. It wraps `DomainError` and
  `std::io::Error`, and covers terminal-specific failures such as
  `ScreenTooSmall`.

Domain and editor functions return `DomainResult`; the app and dispatcher
return `Result`. `?` converts `DomainError` into `IdeError`.

Two intentional gaps: `handle_session_command` ignores the result of
`buffer.save()` (`let _ =`), and `render_into` discards the screen-size error
after switching to the fallback widget. Surfacing these through a toast is
planned.

## Testing

- Unit tests are inline `#[cfg(test)] mod tests` blocks at the bottom of the
  module they exercise.
- Filesystem tests use `tempfile` (`domain` dev-dependency) and never touch
  user paths.
- Current coverage is 166 tests: 106 in `domain` (piece-table insert/delete,
  lines, word motions, model-based edit/replay, `Buffer` integration tests,
  persistence, workspace and file-tree behavior), 34 in `tui` (command-line
  state plus palette, notification, toast, and text-wrapping rendering), 17 in
  `editor` (ex-command parsing, quit behavior, notification/toast bookkeeping,
  and word-motion cursor placement), and 9 in `editor-core` (keymap bindings
  and mode display).
- There are no integration or TUI snapshot tests yet.

## Performance notes

- Edits rewrite only the affected pieces: `insert` splits the piece at the
  offset, and `delete` rebuilds the list from the surviving prefix, trimmed
  boundary pieces, and suffix. Cost is O(pieces) per edit.
- Character- and line-oriented queries (`slice`, `text`, `line`, line/column
  conversion, char/byte conversion) walk the piece list, so they are
  O(document) and invalidate nothing.
- Word motions (`w`, `b`, `e`, `W`, `B`, `E`) locate the starting piece once
  and then scan character slices in the motion direction. Their cost is
  proportional to the bytes traversed, not to a piece lookup per character;
  they still walk the piece list to find the starting position.
- The editor view fetches only the visible viewport (`scroll_y` through
  `scroll_y + height`), so redraw cost does not grow with document length.
- The left panel re-scans the root directory on every frame
  (`scan_dir` in `render/left_panel.rs`). This is filesystem I/O per frame and
  should be cached and refreshed on change.

## Known gaps

- The startup path is hard-coded in `App::new`.
- Buffer switching/closing through key bindings, fullscreen, the right
  panel, paragraph/page motions, yank/paste, and undo/redo are `todo!()`;
  invoking them panics. (`:bn`/`:bp` work through the command palette; word
  motions `w`/`b`/`e`/`W`/`B`/`E` are implemented.)
- The bottom bar always renders in status mode; `BottomBarMode::Command` is
  never selected because the command palette now owns command-line display.
- `crates/config` is an empty placeholder, and several crates declare unused
  dependencies.
