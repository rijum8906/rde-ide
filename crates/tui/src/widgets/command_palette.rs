//! Floating command palette: a modal `:` editor drawn over the layout.
//!
//! The widget renders a rounded card with a title, the prompt and command
//! text, a block caret, and a footer of key hints. It is stateless: everything
//! it draws comes from a [`CommandPaletteView`], and it writes directly into
//! the frame buffer.
//!
//! The input scrolls horizontally so the caret stays visible on long commands.

use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Widget},
};

use crate::viewmodel::command_palette::CommandPaletteView;

const BG: Color = Color::Rgb(26, 26, 34);
const SHADOW: Color = Color::Rgb(8, 8, 10);
const ACCENT: Color = Color::Rgb(130, 170, 255);
const TEXT: Color = Color::Rgb(232, 232, 240);
const MUTED: Color = Color::Rgb(118, 120, 138);

/// Card that renders the command line.
#[derive(Debug)]
pub struct CommandPaletteWidget<'a> {
    view: &'a CommandPaletteView<'a>,
}

impl<'a> CommandPaletteWidget<'a> {
    /// Build the widget from its view model.
    pub fn new(view: &'a CommandPaletteView<'a>) -> Self {
        Self { view }
    }
}

impl Widget for CommandPaletteWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 8 || area.height < 3 {
            return;
        }

        // A soft drop shadow makes the card read as a floating layer.
        draw_shadow(area, buf);

        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(ACCENT))
            .style(Style::default().bg(BG))
            .title(Line::from(Span::styled(
                " Command ",
                Style::default()
                    .fg(ACCENT)
                    .bg(BG)
                    .add_modifier(Modifier::BOLD),
            )))
            .title_bottom(
                Line::from(Span::styled(
                    " Enter run  ·  Esc cancel  ·  ↑↓ history  ·  Ctrl+U clear ",
                    Style::default().fg(MUTED).bg(BG),
                ))
                .alignment(Alignment::Center),
            );

        let inner = block.inner(area);
        block.render(area, buf);

        if inner.width == 0 || inner.height == 0 {
            return;
        }

        self.render_input(inner, buf);
    }
}

impl CommandPaletteWidget<'_> {
    /// Draw the prompt, text, and caret on the middle row of the card.
    fn render_input(&self, inner: Rect, buf: &mut Buffer) {
        let y = inner.y + inner.height / 2;
        let max_x = inner.x + inner.width;
        let mut x = write_str(buf, inner.x, y, max_x, self.view.prompt, prompt_style());

        // One blank column between the prompt and the text.
        x = write_char(buf, x, y, max_x, ' ', text_style());

        let available = max_x.saturating_sub(x) as usize;
        if available == 0 {
            return;
        }

        let chars: Vec<char> = self.view.text.chars().collect();
        let cursor = self.view.cursor.min(chars.len());

        // Scroll just enough to keep one cell free for the caret.
        let scroll = cursor.saturating_sub(available.saturating_sub(1));

        for (index, ch) in chars.iter().enumerate().skip(scroll).take(available) {
            let cell_x = x + (index - scroll) as u16;
            write_char(buf, cell_x, y, max_x, *ch, text_style());
        }

        let cursor_x = x + (cursor - scroll) as u16;
        if cursor_x < max_x {
            let under = chars.get(cursor).copied().unwrap_or(' ');
            write_char(buf, cursor_x, y, max_x, under, caret_style());
        }
    }
}

/// Paint a one-cell offset shadow behind the card, clipped to the buffer.
fn draw_shadow(area: Rect, buf: &mut Buffer) {
    let shadow = Rect {
        x: area.x.saturating_add(2),
        y: area.y.saturating_add(1),
        width: area.width,
        height: area.height,
    }
    .intersection(buf.area);

    if shadow.width == 0 || shadow.height == 0 {
        return;
    }

    for y in shadow.y..shadow.y + shadow.height {
        for x in shadow.x..shadow.x + shadow.width {
            if let Some(cell) = buf.cell_mut((x, y)) {
                cell.set_style(Style::default().bg(SHADOW));
            }
        }
    }
}

/// Write a string starting at `x`, clipped to `max_x`. Returns the next free x.
fn write_str(buf: &mut Buffer, mut x: u16, y: u16, max_x: u16, s: &str, style: Style) -> u16 {
    for ch in s.chars() {
        if x >= max_x {
            break;
        }
        write_char(buf, x, y, max_x, ch, style);
        x = x.saturating_add(1);
    }
    x
}

/// Write a single character at `x`, clipped to `max_x`. Returns the next x.
fn write_char(buf: &mut Buffer, x: u16, y: u16, max_x: u16, ch: char, style: Style) -> u16 {
    if x >= max_x {
        return x;
    }
    if let Some(cell) = buf.cell_mut((x, y)) {
        cell.set_char(ch).set_style(style);
    }
    x.saturating_add(1)
}

fn prompt_style() -> Style {
    Style::default()
        .fg(ACCENT)
        .bg(BG)
        .add_modifier(Modifier::BOLD)
}

fn text_style() -> Style {
    Style::default().fg(TEXT).bg(BG)
}

fn caret_style() -> Style {
    Style::default().fg(BG).bg(TEXT)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render(view: &CommandPaletteView<'_>, area: Rect) -> Buffer {
        let mut buf = Buffer::empty(area);
        CommandPaletteWidget::new(view).render(area, &mut buf);
        buf
    }

    fn row_text(buf: &mut Buffer, width: u16, y: u16) -> String {
        (0..width)
            .map(|x| buf.cell_mut((x, y)).unwrap().symbol().to_string())
            .collect()
    }

    #[test]
    fn draws_rounded_card_with_prompt_and_text() {
        let view = CommandPaletteView {
            prompt: ":",
            text: "w foo",
            cursor: 5,
        };
        let area = Rect {
            x: 0,
            y: 0,
            width: 40,
            height: 3,
        };
        let mut buf = render(&view, area);

        assert_eq!(buf.cell_mut((0, 0)).unwrap().symbol(), "╭");
        assert_eq!(buf.cell_mut((39, 2)).unwrap().symbol(), "╯");
        assert!(row_text(&mut buf, area.width, 1).contains(": w foo"));
    }

    #[test]
    fn keeps_caret_visible_on_long_commands() {
        let text = "e ".to_string() + &"a".repeat(80);
        let view = CommandPaletteView {
            prompt: ":",
            text: &text,
            cursor: text.chars().count(),
        };
        let area = Rect {
            x: 0,
            y: 0,
            width: 30,
            height: 3,
        };
        let mut buf = render(&view, area);

        let caret = (0..area.width).any(|x| {
            let cell = buf.cell_mut((x, 1)).unwrap();
            cell.style().bg == Some(TEXT) && cell.style().fg == Some(BG)
        });
        assert!(caret, "caret should stay inside the card");
    }
}
