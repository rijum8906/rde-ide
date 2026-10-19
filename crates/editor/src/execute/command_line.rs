//! Command-line (`:`) editing commands.
//!
//! Editing is delegated to [`BottomBarState`], which owns the input buffer,
//! caret, and history. On submit the trimmed line is parsed as a small subset
//! of Vim's ex commands:
//!
//! | Command | Action |
//! | --- | --- |
//! | `:w [path]` | Write the active buffer, optionally to `path` |
//! | `:q` | Quit when no buffer has unsaved changes |
//! | `:q!` | Quit regardless of unsaved changes |
//! | `:wq`, `:x`, `:exit` | Write, then quit |
//! | `:e <path>` | Open `path` (relative to the workspace root) |
//! | `:bn`, `:bp` | Switch to the next/previous buffer |
//!
//! Anything else reports `E492` on the status line.

use std::path::PathBuf;

use domain::{
    error::{DomainError, DomainResult},
    workspace::WorkspaceState,
};
use editor_core::{
    command::CommandLineCommand,
    input::{Input, mode::InputMode},
};
use tui::ui::Ui;

use super::Effect;

/// Handle one command-line action.
///
/// Returns [`Effect::Quit`] when a submitted command asks the app to exit.
pub fn handle_command_line_command(
    ws: &mut WorkspaceState,
    input: &mut Input,
    ui: &mut Ui,
    cmd: CommandLineCommand,
) -> DomainResult<Effect> {
    match cmd {
        CommandLineCommand::Submit => return Ok(submit(ws, input, ui)),
        CommandLineCommand::Open => {
            ui.widgets.bottom_bar.state.clear_command_line();
            input.mode = InputMode::Command;
        }
        CommandLineCommand::Cancel => {
            ui.widgets.bottom_bar.state.clear_command_line();
            input.mode = InputMode::Normal;
        }
        CommandLineCommand::Push(c) => ui.widgets.bottom_bar.state.push_char(c),
        CommandLineCommand::Pop => ui.widgets.bottom_bar.state.backspace(),
        CommandLineCommand::Clear => ui.widgets.bottom_bar.state.clear_command_line(),
        CommandLineCommand::CursorLeft => ui.widgets.bottom_bar.state.move_cursor_left(),
        CommandLineCommand::CursorRight => ui.widgets.bottom_bar.state.move_cursor_right(),
        CommandLineCommand::CursorHome => ui.widgets.bottom_bar.state.move_cursor_home(),
        CommandLineCommand::CursorEnd => ui.widgets.bottom_bar.state.move_cursor_end(),
        CommandLineCommand::HistoryPrev => ui.widgets.bottom_bar.state.history_prev(),
        CommandLineCommand::HistoryNext => ui.widgets.bottom_bar.state.history_next(),
    }

    Ok(Effect::None)
}

/// Take the typed line, record it in history, leave command mode, and run it.
fn submit(ws: &mut WorkspaceState, input: &mut Input, ui: &mut Ui) -> Effect {
    let line = ui.widgets.bottom_bar.state.take_command_line();
    ui.widgets.bottom_bar.state.reset_history_nav();
    input.mode = InputMode::Normal;

    let line = line.trim().to_string();
    if line.is_empty() {
        return Effect::None;
    }

    ui.widgets.bottom_bar.state.push_history(line.clone());
    run_ex(ws, ui, &line)
}

/// Execute a parsed ex command line.
fn run_ex(ws: &mut WorkspaceState, ui: &mut Ui, line: &str) -> Effect {
    let (name, arg) = split_ex(line);

    match name {
        "" => Effect::None,

        "w" | "write" => {
            save(ws, ui, arg);
            Effect::None
        }

        "q" | "quit" => quit(ws, ui, false),
        "q!" => quit(ws, ui, true),

        "wq" | "wq!" | "x" | "xit" | "exit" => {
            if save(ws, ui, arg) {
                Effect::Quit
            } else {
                Effect::None
            }
        }

        "e" | "edit" => {
            edit(ws, ui, arg);
            Effect::None
        }

        "bn" | "bnext" => {
            switch_buffer(ws, ui, true);
            Effect::None
        }

        "bp" | "bprev" => {
            switch_buffer(ws, ui, false);
            Effect::None
        }

        other => {
            error(ui, format!("E492: Not an editor command: {other}"));
            Effect::None
        }
    }
}

/// Split `":w path"` into the command name and its single argument.
///
/// The argument keeps interior spaces (`:w my notes.md`); surrounding
/// whitespace and a leading `:` are dropped.
fn split_ex(line: &str) -> (&str, Option<&str>) {
    let line = line.trim();
    let line = line.strip_prefix(':').unwrap_or(line).trim_start();

    let mut parts = line.splitn(2, char::is_whitespace);
    let name = parts.next().unwrap_or("");
    let arg = parts.next().map(str::trim).filter(|arg| !arg.is_empty());

    (name, arg)
}

/// Write the active buffer, optionally to a new path.
///
/// Returns `true` when the write succeeded.
fn save(ws: &mut WorkspaceState, ui: &mut Ui, path: Option<&str>) -> bool {
    let target = path.map(|path| resolve_path(ws, path));

    let result = match ws.active_buffer_mut() {
        Some(buffer) => match &target {
            Some(target) => buffer.save_to_path(target),
            None => buffer.save(),
        },
        None => {
            error(ui, "E32: No buffer to write");
            return false;
        }
    };

    match result {
        Ok(()) => {
            let (name, lines) = ws
                .active_buffer()
                .map(|buffer| (buffer.display_name().to_string(), buffer.line_count()))
                .unwrap_or_default();
            info(ui, format!("\"{name}\" {lines}L written"));
            true
        }
        Err(DomainError::NoPath) => {
            error(ui, "E32: No file name (use :w <path>)");
            false
        }
        Err(err) => {
            error(ui, format!("E212: Can't write file: {err}"));
            false
        }
    }
}

/// Quit unless a buffer is dirty and the user did not force it.
fn quit(ws: &WorkspaceState, ui: &mut Ui, force: bool) -> Effect {
    if !force && ws.has_unsaved_changes() {
        error(ui, "E37: No write since last change (add ! to override)");
        Effect::None
    } else {
        Effect::Quit
    }
}

/// Open a file, resolved relative to the workspace root.
fn edit(ws: &mut WorkspaceState, ui: &mut Ui, arg: Option<&str>) {
    let Some(arg) = arg else {
        error(ui, "E32: No file name");
        return;
    };

    let path = resolve_path(ws, arg);
    match ws.open_file(&path) {
        Ok(()) => {
            reset_editor_view(ws, ui);
            let name = ws
                .active_buffer()
                .map(|buffer| buffer.display_name().to_string())
                .unwrap_or_default();
            info(ui, format!("\"{name}\""));
        }
        Err(err) => error(ui, format!("E484: Can't open file {arg}: {err}")),
    }
}

/// Switch to the next or previous buffer and reset the viewport.
fn switch_buffer(ws: &mut WorkspaceState, ui: &mut Ui, forward: bool) {
    if ws.buffer_count() < 2 {
        info(ui, "E85: Only one buffer");
        return;
    }

    if forward {
        ws.next_buffer();
    } else {
        ws.prev_buffer();
    }

    reset_editor_view(ws, ui);
}

/// Resolve an ex-path argument against the workspace root.
fn resolve_path(ws: &WorkspaceState, path: &str) -> PathBuf {
    let path = std::path::Path::new(path);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        ws.root_dir().join(path)
    }
}

/// Move the cursor back to the start and refresh the cached line count.
fn reset_editor_view(ws: &WorkspaceState, ui: &mut Ui) {
    let line_count = ws
        .active_buffer()
        .map(|buffer| buffer.line_count())
        .unwrap_or(0);
    let state = &mut ui.widgets.editor_panel.state;
    state.reset();
    state.set_line_count(line_count);
}

fn info(ui: &mut Ui, text: impl Into<String>) {
    ui.widgets.bottom_bar.state.info(text);
}

fn error(ui: &mut Ui, text: impl Into<String>) {
    ui.widgets.bottom_bar.state.error(text);
}

#[cfg(test)]
mod tests {
    use super::{Effect, run_ex, split_ex};
    use domain::{buffer::Buffer, workspace::WorkspaceState};
    use tui::ui::Ui;

    fn workspace() -> WorkspaceState {
        let mut ws = WorkspaceState::new(".");
        ws.push_buffer(Buffer::scratch("scratch"));
        ws
    }

    #[test]
    fn splits_command_without_argument() {
        assert_eq!(split_ex("w"), ("w", None));
        assert_eq!(split_ex("q!"), ("q!", None));
    }

    #[test]
    fn splits_command_and_argument() {
        assert_eq!(split_ex("w file.txt"), ("w", Some("file.txt")));
        assert_eq!(split_ex(":e  src/main.rs"), ("e", Some("src/main.rs")));
    }

    #[test]
    fn keeps_spaces_inside_argument() {
        assert_eq!(split_ex("w my notes.md"), ("w", Some("my notes.md")));
    }

    #[test]
    fn empty_line_has_no_command() {
        assert_eq!(split_ex("   "), ("", None));
        assert_eq!(split_ex(":"), ("", None));
    }

    #[test]
    fn forced_quit_requests_app_exit() {
        let mut ws = workspace();
        let mut ui = Ui::default();

        assert_eq!(run_ex(&mut ws, &mut ui, "q!"), Effect::Quit);
    }

    #[test]
    fn quit_is_refused_while_a_buffer_is_dirty() {
        let mut ws = workspace();
        ws.active_buffer_mut().unwrap().insert(0, "x").unwrap();
        let mut ui = Ui::default();

        assert_eq!(run_ex(&mut ws, &mut ui, "q"), Effect::None);
        assert!(ui.widgets.bottom_bar.state.message.is_some());
    }

    #[test]
    fn unknown_commands_report_an_error() {
        let mut ws = workspace();
        let mut ui = Ui::default();

        assert_eq!(run_ex(&mut ws, &mut ui, "frobnicate"), Effect::None);
        let message = ui.widgets.bottom_bar.state.message.as_ref().unwrap();
        assert!(message.text.contains("E492"));
    }
}
