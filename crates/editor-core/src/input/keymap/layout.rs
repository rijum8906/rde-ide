use crate::{
    command::{Command, LayoutCommand},
    input::{
        key::{Key, KeyExt},
        keymap::Keymap,
        mode::InputMode,
    },
};

pub(super) fn default_layout_keymap(keymap: &mut Keymap) -> &mut Keymap {
    let normal = InputMode::Normal;
    keymap.bind(
        normal,
        vec![Key::ctrl('f')],
        Command::Layout(LayoutCommand::ToggleFullscreen),
    );
    keymap.bind(
        normal,
        vec![Key::char(' '), Key::char('e')],
        Command::Layout(LayoutCommand::ToggleLeftPanel),
    );
    keymap.bind(
        normal,
        vec![Key::ctrl('b')],
        Command::Layout(LayoutCommand::ToggleLeftPanel),
    );

    keymap
}
