//! Fallback shown when the terminal is smaller than the supported minimum.
//!
//! Instead of drawing a broken layout, the whole frame is replaced by this
//! widget, which reports the minimum and current dimensions. Very small
//! viewports get a compact, borderless variant.

use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Paragraph, Widget, Wrap},
};

use crate::constant::{MIN_SCREEN_HEIGHT, MIN_SCREEN_WIDTH};

/// A warning widget displayed when the terminal viewport is smaller than required.
pub struct ScreenTooSmall {
    cur_w: u16,
    cur_h: u16,
}

impl ScreenTooSmall {
    pub fn new(cur_w: u16, cur_h: u16) -> Self {
        Self { cur_w, cur_h }
    }
}

// Implementing Ratatui's Widget trait makes it a first-class component
impl Widget for ScreenTooSmall {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // For extremely small viewports, render compact text without borders
        if area.width < 32 || area.height < 6 {
            let text = vec![
                Line::from(Span::styled(
                    "Screen too small!",
                    Style::default().fg(Color::Red).bold(),
                )),
                Line::from(Span::styled(
                    format!(
                        "{}x{} < {}x{}",
                        self.cur_w, self.cur_h, MIN_SCREEN_WIDTH, MIN_SCREEN_HEIGHT
                    ),
                    Style::default().fg(Color::Yellow),
                )),
            ];
            Paragraph::new(text)
                .alignment(Alignment::Center)
                .render(area, buf);
            return;
        }

        // Determine warning box bounds (centered within frame area)
        let box_width = 54.min(area.width);
        let box_height = 9.min(area.height);

        let vertical_margin = area.height.saturating_sub(box_height) / 2;
        let horizontal_margin = area.width.saturating_sub(box_width) / 2;

        let popup_area = Rect {
            x: area.x + horizontal_margin,
            y: area.y + vertical_margin,
            width: box_width,
            height: box_height,
        };

        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Yellow))
            .title(Line::from(vec![
                Span::styled(" ⚠ ", Style::default().fg(Color::Yellow).bold()),
                Span::styled(
                    "Screen Size Too Small",
                    Style::default().fg(Color::Red).bold(),
                ),
                Span::raw(" "),
            ]))
            .title_alignment(Alignment::Center);

        let mut lines = Vec::new();
        if box_height >= 7 {
            lines.push(Line::from(""));
        }
        lines.push(Line::from(vec![
            Span::raw("Minimum required: "),
            Span::styled(
                format!("{} x {}", MIN_SCREEN_WIDTH, MIN_SCREEN_HEIGHT),
                Style::default().fg(Color::Green).bold(),
            ),
        ]));
        lines.push(Line::from(vec![
            Span::raw("Current size:     "),
            Span::styled(
                format!("{} x {}", self.cur_w, self.cur_h),
                Style::default().fg(Color::Red).bold(),
            ),
        ]));
        if box_height >= 8 {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "Please enlarge the terminal window to continue.",
                Style::default().fg(Color::Yellow),
            )));
        }

        Paragraph::new(lines)
            .block(block)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true })
            .render(popup_area, buf);
    }
}
