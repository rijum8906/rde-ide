# AGENTS.md

Guidance for contributors and AI coding agents working on rde-ide.

## Project snapshot

rde-ide is a terminal code editor written in Rust, organized as a Cargo
workspace (edition 2024, MSRV 1.85). There is one binary, `rde-ide`, in
`apps/ide`. The workspace has no CI yet, so the local checks below are the
only gate.

- UI: Ratatui 0.30 with crossterm 0.29.
- State: custom piece-table buffer in `domain`.
- Input: custom `Key`, keymap, mode, and key-sequence resolver in
  `editor-core`.
- Testing: inline unit tests; `tempfile` for filesystem tests.

## Commands

Run from the repository root.

```bash
cargo build --workspace
cargo run -p rde-ide

cargo test --workspace
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo doc --workspace --no-deps
```

Every check must pass with exit status 0 before you call work done. When you
filter output through `grep`/`head`, preserve the command's status
(`set -o pipefail` or `PIPESTATUS`); a clean filtered view is not a pass.

## Repository map

| Path | Contents |
| --- | --- |
| `apps/ide/src/app.rs` | `App`: owns workspace, input, UI; event loop and key fall-through |
| `apps/ide/src/main.rs` | Terminal setup/teardown; runs `App` |
| `crates/domain/src/piece_table/` | `PieceTable`: pieces, edits, lines, word motions |
| `crates/domain/` | `Buffer`, `WorkspaceState`, `IdeError`/`DomainError`, file-tree scan |
| `crates/editor-core/src/command/` | `Command` and category enums |
| `crates/editor-core/src/input/` | `Key`, `Keymap`, `InputMode`, `Resolver` |
| `crates/editor/src/execute/` | One free-function handler per command category |
| `crates/editor/src/mutation/` | Reusable text/cursor operations |
| `crates/tui/src/` | `Ui`, layout, geometry, render, state, viewmodel, widgets |
| `crates/config/` | Placeholder for user configuration; not wired up |

## Architectural rules

1. **Dependencies point downward.** `rde-ide` → `{editor, tui, editor-core,
   domain}`; `editor` → `{tui, domain, editor-core}`; `tui` → `{domain,
   editor-core}`; `domain` and `editor-core` are leaves. Never add a
   dependency that points back up.
2. **Terminal types stay at the boundary.** Crossterm is converted to `Key` in
   `crates/editor-core/src/input/key.rs` and read in `apps/ide/src/app.rs`;
   do not import crossterm or ratatui in `domain` or `editor`.
3. **`domain` is UI-agnostic.** No cursors, layout, or rendering types there.
   (It currently declares `ratatui` without using it; don't add such
   dependencies.)
4. **`editor` mutates but never draws.** Handlers receive only the state they
   need and return `DomainResult<()>` when they can fail.
5. **Handlers are free functions**, not `impl App`. `App` lives in the binary
   crate; `editor` must not depend on it.
6. **Lifecycle belongs to the app.** Quit handling, unbound-key fall-through,
   and terminal management live in `apps/ide/src/app.rs`.
7. **UI state lives in `tui`.** Put cursor/scroll state in `tui/src/state`,
   per-frame snapshots in `tui/src/viewmodel`, drawing in `tui/src/widgets`.
8. **Lifecycle is reported, not performed.** `execute` returns an `Effect`
   that the caller applies. Quit commands are intercepted by the dispatcher;
   `App::dispatch` stops the event loop.

## Where does this code go?

| Change | Location |
| --- | --- |
| New key type or normalization rule | `editor-core/src/input/key.rs` |
| New action | `editor-core/src/command/<category>.rs` |
| Behavior for an action | `editor/src/execute/<category>.rs` |
| New `:` ex command | `editor/src/execute/command_line.rs` |
| Reusable buffer/cursor operation | `editor/src/mutation/` |
| Text storage or line/word behavior | `domain/src/piece_table/` |
| Document or workspace behavior | `domain/src/buffer/` or `domain/src/workspace/` |
| Persistent UI state | `tui/src/state/` |
| Data a widget needs each frame | `tui/src/viewmodel/` |
| Drawing | `tui/src/widgets/` |
| Sizing | `tui/src/geometry.rs` |
| Per-frame orchestration | `tui/src/render.rs` |
| App-level coordination | `apps/ide/src/app.rs` |

## Adding a command

1. Add a variant to the matching enum in `crates/editor-core/src/command/`.
2. Handle it in the category handler in `crates/editor/src/execute/`.
   Implement the operation in `crates/editor/src/mutation/` if it is reusable.
3. Bind it in the matching file in
   `crates/editor-core/src/input/keymap/`.
4. Add a keymap test for chords or shifted keys.
5. Update the module docs if the module's responsibility changed, and the
   key-binding tables in `README.md` if user-visible behavior changed.

## Conventions

- **Module headers:** every `.rs` file starts with a `//!` header describing
  what the module owns. Update it when the responsibility changes.
- **Units:** cursor positions are 0-indexed **characters**; `Buffer::insert`
  and `Buffer::delete` take **byte** offsets. Convert at the mutation boundary
  via `cursor_char_offset` / `char_to_byte_offset`.
- **Screen size is `(rows, columns)`** everywhere (`Layout::screen_size`).
- **Workspace access:** use `active_buffer()` / `active_buffer_mut()` and keep
  the invariant `buffers.is_empty() || active_buffer_idx < buffers.len()`.
- **Errors:** `DomainError`/`DomainResult` inside libraries,
  `IdeError`/`Result` at the app boundary. Do not `unwrap()` in library code;
  return an error.
- **Unimplemented commands:** `todo!()` is acceptable only for commands that
  are declared but not bound to a working path. Remember that invoking one
  panics at runtime; say so in the docs and avoid binding new keys to them.
- **Tests:** inline `#[cfg(test)] mod tests` at the bottom of the module;
  use `tempfile` for filesystem tests and never hardcode user paths.
- **Formatting:** default rustfmt. Run `cargo fmt --all` before finishing.
- **Comments:** explain *why*, not *what*. Keep existing doc comments accurate
  instead of layering a new comment on top of a stale one.

## Commits

- Only commit when the user asks.
- Use conventional commit subjects: `feat(scope): ...`, `fix(scope): ...`,
  `refactor: ...`, `docs: ...`, `test: ...`.
- Keep commits focused; stage only files related to the change. Do not commit
  `target/` (it is gitignored).
- Do not push, force-push, or rewrite history unless explicitly asked.

## Known limitations and pitfalls

- `App::new` opens a hard-coded absolute path. There is no CLI argument
  parsing yet; don't rely on the path existing on another machine.
- These commands panic today because their handlers are `todo!()`: buffer
  switching/closing through key bindings (`Tab`, `Ctrl+W`,
  `b n`/`b p`/`b d`/`b D`), fullscreen (`Ctrl+F`), panel focus,
  paragraph/page motions, yank/paste, save-as, save-all, open-file, and
  new-scratch-buffer. The command palette's `:bn`, `:bp`, `:e`, and
  `:w <path>` cover some of these paths. Word motions (`w`/`b`/`e`) and
  undo/redo (`u`/`Ctrl+R`) are implemented.
- `Quit`/`ForceQuit` are intercepted in `App::handle_key`, not in
  `App::tick`.
- The bottom bar receives a 0-indexed `(col, line)` although its view model
  documents `(line, col)`.
- Some dependencies are declared but unused (`tokio` in `rde-ide`,
  `color-eyre` in `tui` and `editor`, `ratatui` in `domain` and `editor`,
  `crossterm` in `editor`). Removing them is welcome when you touch those
  manifests; don't add more.

## Definition of done

- The change is in the right layer and does not invert dependencies.
- New public items and changed modules have accurate `//!`/`///` docs.
- `cargo test --workspace` passes, and new behavior has a unit test where it
  is testable without a terminal.
- `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D
  warnings`, `cargo doc --workspace --no-deps`, and `cargo check --workspace`
  are clean.
- `README.md` and `ARCHITECTURE.md` are updated if user-visible behavior,
  structure, or limitations changed.
- No checks were weakened, no `#[allow]` added to silence a lint, and no
  assertions removed to make a test pass.
