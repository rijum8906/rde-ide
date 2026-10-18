use crate::{
    command::command::{
        Command,
        MotionCommand::{Down, FileEnd, FileStart, Left, LineEnd, LineStart, Right, Up},
    },
    input::{
        key::{Key, KeyCode, KeyExt},
        keymap::Keymap,
        mode::Mode,
    },
};

pub(super) fn default_motion_keymap(keymap: &mut Keymap) -> &mut Keymap {
    let normal = Mode::Normal;
    keymap.bind(normal, vec![Key::char('h')], Command::Motion(Left));
    keymap.bind(
        normal,
        vec![Key::plain(KeyCode::Left)],
        Command::Motion(Left),
    );
    keymap.bind(normal, vec![Key::char('j')], Command::Motion(Down));
    keymap.bind(
        normal,
        vec![Key::plain(KeyCode::Down)],
        Command::Motion(Down),
    );
    keymap.bind(normal, vec![Key::char('k')], Command::Motion(Up));
    keymap.bind(normal, vec![Key::plain(KeyCode::Up)], Command::Motion(Up));
    keymap.bind(normal, vec![Key::char('l')], Command::Motion(Right));
    keymap.bind(
        normal,
        vec![Key::plain(KeyCode::Right)],
        Command::Motion(Right),
    );
    keymap.bind(
        normal,
        vec![Key::char('g'), Key::char('g')],
        Command::Motion(FileStart),
    );
    keymap.bind(normal, vec![Key::char('G')], Command::Motion(FileEnd));
    keymap.bind(normal, vec![Key::char('0')], Command::Motion(LineStart));
    keymap.bind(normal, vec![Key::char('$')], Command::Motion(LineEnd));

    let insert = Mode::Insert;
    keymap.bind(
        insert,
        vec![Key::plain(KeyCode::Left)],
        Command::Motion(Left),
    );
    keymap.bind(
        insert,
        vec![Key::plain(KeyCode::Down)],
        Command::Motion(Down),
    );
    keymap.bind(insert, vec![Key::plain(KeyCode::Up)], Command::Motion(Up));
    keymap.bind(
        insert,
        vec![Key::plain(KeyCode::Right)],
        Command::Motion(Right),
    );

    keymap
}
