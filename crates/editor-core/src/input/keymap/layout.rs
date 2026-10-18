use crate::{
    command::command::{Command, LayoutCommand},
    input::{
        key::{Key, KeyExt},
        keymap::Keymap,
        mode::Mode,
    },
};

pub(super) fn default_layout_keymap(keymap: &mut Keymap) -> &mut Keymap {
    let normal = Mode::Normal;
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
