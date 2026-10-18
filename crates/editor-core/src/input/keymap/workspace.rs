use crate::{
    command::command::{Command, WorkspaceCommand},
    input::{
        key::{Key, KeyCode, KeyExt},
        keymap::Keymap,
        mode::InputMode,
    },
};

pub(super) fn default_workspace_keymap(keymap: &mut Keymap) -> &mut Keymap {
    let normal = InputMode::Normal;
    keymap.bind(
        normal,
        vec![Key::plain(KeyCode::Tab)],
        Command::Workspace(WorkspaceCommand::NextBuffer),
    );
    keymap.bind(
        normal,
        vec![Key::plain(KeyCode::BackTab)],
        Command::Workspace(WorkspaceCommand::PrevBuffer),
    );
    keymap.bind(
        normal,
        vec![Key::ctrl('w')],
        Command::Workspace(WorkspaceCommand::CloseBuffer),
    );
    keymap.bind(
        normal,
        vec![Key::char('H')],
        Command::Workspace(WorkspaceCommand::NextBuffer),
    );
    keymap.bind(
        normal,
        vec![Key::char('b'), Key::char('n')],
        Command::Workspace(WorkspaceCommand::NextBuffer),
    );
    keymap.bind(
        normal,
        vec![Key::char('L')],
        Command::Workspace(WorkspaceCommand::PrevBuffer),
    );
    keymap.bind(
        normal,
        vec![Key::char('b'), Key::char('p')],
        Command::Workspace(WorkspaceCommand::PrevBuffer),
    );
    keymap.bind(
        normal,
        vec![Key::char('b'), Key::char('d')],
        Command::Workspace(WorkspaceCommand::CloseBuffer),
    );
    keymap.bind(
        normal,
        vec![Key::char('b'), Key::char('D')],
        Command::Workspace(WorkspaceCommand::CloseBufferForce),
    );
    keymap.bind(
        normal,
        vec![Key::ctrl('h')],
        Command::Workspace(WorkspaceCommand::FocusLeftPanel),
    );
    keymap.bind(
        normal,
        vec![Key::ctrl('l')],
        Command::Workspace(WorkspaceCommand::FocusRightPanel),
    );
    keymap.bind(
        normal,
        vec![Key::ctrl('i')],
        Command::Workspace(WorkspaceCommand::FocusEditorPanel),
    );

    keymap
}
