//! Shared card rendering for floating widgets.
//!
//! Notifications and toasts draw the same visual element — a rounded card with
//! a level icon, bold header, right-aligned level label, and wrapped body. Only
//! their placement and body height differ.

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Widget},
};

use crate::{
    viewmodel::level::Level,
    widgets::text::{clamp, wrap},
};

const BG: Color = Color::Rgb(26, 26, 34);
const HEADER: Color = Color::Rgb(238, 238, 246);
const BODY: Color = Color::Rgb(186, 188, 202);
const LABEL: Color = Color::Rgb(118, 120, 138);

/// Rows a card needs at `width`: two borders, a header, and the body lines.
pub(crate) fn card_height(body: &str, width: u16, max_body_lines: usize) -> u16 {
    let inner_width = width.saturating_sub(2) as usize;
    3 + wrapped_body(body, inner_width, max_body_lines).len() as u16
}

/// Draw one card at `area`.
pub(crate) fn render_card(
    level: Level,
    header: &str,
    body: &str,
    max_body_lines: usize,
    area: Rect,
    buf: &mut Buffer,
) {
    let accent = level.color();
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(accent))
        .style(Style::default().bg(BG));

    let inner = block.inner(area);
    block.render(area, buf);

    if inner.width == 0 || inner.height == 0 {
        return;
    }

    let max_x = inner.x.saturating_add(inner.width);

    // Header row: icon, title, right-aligned level label.
    let header = if header.trim().is_empty() {
        level.name()
    } else {
        header
    };

    let mut x = write_str(
        buf,
        inner.x,
        inner.y,
        max_x,
        level.icon(),
        Style::default()
            .fg(accent)
            .bg(BG)
            .add_modifier(Modifier::BOLD),
    );
    x = write_char(buf, x, inner.y, max_x, ' ', header_style());
    x = write_str(buf, x, inner.y, max_x, header, header_style());

    let label = level.label();
    let label_x = max_x.saturating_sub(label.chars().count() as u16);
    if label_x > x {
        write_str(buf, label_x, inner.y, max_x, label, label_style());
    }

    // Body rows.
    let lines = wrapped_body(body, inner.width as usize, max_body_lines);
    for (index, line) in lines.iter().enumerate() {
        let y = inner.y.saturating_add(1 + index as u16);
        if y >= inner.y.saturating_add(inner.height) {
            break;
        }
        write_str(buf, inner.x, y, max_x, line, body_style());
    }
}

/// Body lines wrapped to `width`, clamped to `max_body_lines`.
fn wrapped_body(text: &str, width: usize, max_body_lines: usize) -> Vec<String> {
    clamp(wrap(text, width), max_body_lines, width)
}

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

fn write_char(buf: &mut Buffer, x: u16, y: u16, max_x: u16, ch: char, style: Style) -> u16 {
    if x >= max_x {
        return x;
    }
    if let Some(cell) = buf.cell_mut((x, y)) {
        cell.set_char(ch).set_style(style);
    }
    x.saturating_add(1)
}

fn header_style() -> Style {
    Style::default()
        .fg(HEADER)
        .bg(BG)
        .add_modifier(Modifier::BOLD)
}

fn body_style() -> Style {
    Style::default().fg(BODY).bg(BG)
}

fn label_style() -> Style {
    Style::default()
        .fg(LABEL)
        .bg(BG)
        .add_modifier(Modifier::DIM)
}
