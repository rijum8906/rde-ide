//! Floating notification cards: a stack of header + body cards.
//!
//! Each notification renders as a card from `widgets/card.rs`: rounded border,
//! level icon and header, right-aligned level label, and a wrapped body
//! clamped to a few lines. Cards are stacked top-to-bottom with a one-row gap
//! and stop at the bottom of the area; anything that does not fit is skipped.
//!
//! The widget is stateless: all it draws comes from the
//! [`NotificationView`] slice handed to it.

use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

use crate::{
    viewmodel::notification::NotificationView,
    widgets::card::{card_height, render_card},
};

/// Body rows a single card may show before it is truncated.
const MAX_BODY_LINES: usize = 3;

/// Blank rows between two adjacent cards.
const GAP: u16 = 1;

/// A stack of notification cards, newest first.
#[derive(Debug)]
pub struct NotificationWidget<'a> {
    views: &'a [NotificationView<'a>],
}

impl<'a> NotificationWidget<'a> {
    pub fn new(views: &'a [NotificationView<'a>]) -> Self {
        Self { views }
    }
}

impl NotificationWidget<'_> {
    /// Rows the visible stack occupies in `area`, excluding the trailing gap.
    pub fn used_height(&self, area: Rect) -> u16 {
        card_layout(self.views, area)
            .last()
            .map(|(card, _)| card.y.saturating_add(card.height).saturating_sub(area.y))
            .unwrap_or(0)
    }
}

impl Widget for NotificationWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 10 || area.height < 3 {
            return;
        }

        for (card, view) in card_layout(self.views, area) {
            render_card(
                view.level,
                view.header,
                view.body,
                MAX_BODY_LINES,
                card,
                buf,
            );
        }
    }
}

/// Rects for the cards that fit in `area`, top to bottom.
fn card_layout<'a>(
    views: &'a [NotificationView<'a>],
    area: Rect,
) -> Vec<(Rect, &'a NotificationView<'a>)> {
    let bottom = area.y.saturating_add(area.height);
    let mut y = area.y;
    let mut cards = Vec::new();

    for view in views {
        let height = card_height(view.body, area.width, MAX_BODY_LINES);
        if y.saturating_add(height) > bottom {
            break;
        }

        cards.push((
            Rect {
                x: area.x,
                y,
                width: area.width,
                height,
            },
            view,
        ));

        y = y.saturating_add(height).saturating_add(GAP);
    }

    cards
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::viewmodel::notification::NotificationLevel;

    fn render(views: &[NotificationView<'_>], area: Rect) -> Buffer {
        let mut buf = Buffer::empty(area);
        NotificationWidget::new(views).render(area, &mut buf);
        buf
    }

    fn row_text(buf: &mut Buffer, width: u16, y: u16) -> String {
        (0..width)
            .map(|x| buf.cell_mut((x, y)).unwrap().symbol().to_string())
            .collect()
    }

    fn card<'a>(level: NotificationLevel, header: &'a str, body: &'a str) -> NotificationView<'a> {
        NotificationView {
            level,
            header,
            body,
        }
    }

    #[test]
    fn cards_are_not_shadowed() {
        let views = [card(NotificationLevel::Info, "Header", "body")];
        let area = Rect {
            x: 0,
            y: 0,
            width: 40,
            height: 6,
        };
        let mut buf = Buffer::empty(area);
        let untouched = buf.cell_mut((0, 4)).unwrap().style();
        NotificationWidget::new(&views).render(area, &mut buf);

        // The row under the card used to be painted with a drop shadow.
        for x in 0..area.width {
            assert_eq!(buf.cell_mut((x, 4)).unwrap().style(), untouched);
        }
    }

    #[test]
    fn draws_header_and_body_in_a_rounded_card() {
        let views = [card(
            NotificationLevel::Success,
            "Build finished",
            "Compiled 12 files in 0.8s",
        )];
        let area = Rect {
            x: 0,
            y: 0,
            width: 40,
            height: 4,
        };
        let mut buf = render(&views, area);

        assert_eq!(buf.cell_mut((0, 0)).unwrap().symbol(), "╭");
        assert_eq!(buf.cell_mut((39, 3)).unwrap().symbol(), "╯");

        let header_row = row_text(&mut buf, area.width, 1);
        assert!(header_row.contains("Build finished"), "{header_row:?}");
        assert!(header_row.contains("DONE"), "{header_row:?}");

        let body_row = row_text(&mut buf, area.width, 2);
        assert!(body_row.contains("Compiled 12 files"), "{body_row:?}");
    }

    #[test]
    fn stacks_cards_with_a_one_row_gap() {
        let views = [
            card(NotificationLevel::Info, "First", "one"),
            card(NotificationLevel::Error, "Second", "two"),
        ];
        let area = Rect {
            x: 0,
            y: 0,
            width: 40,
            height: 10,
        };
        let mut buf = render(&views, area);

        // Each card is 4 rows tall, so the second starts at y = 5.
        assert_eq!(buf.cell_mut((0, 5)).unwrap().symbol(), "╭");
        assert!(
            row_text(&mut buf, area.width, 6).contains("Second"),
            "second card header missing"
        );
        assert!(
            row_text(&mut buf, area.width, 1).contains("First"),
            "first card header missing"
        );
    }

    #[test]
    fn truncates_long_bodies_with_an_ellipsis() {
        let body = "word ".repeat(60);
        let views = [card(NotificationLevel::Warning, "Long", &body)];
        let area = Rect {
            x: 0,
            y: 0,
            width: 30,
            height: 6,
        };
        let mut buf = render(&views, area);

        let text: String = (0..area.height)
            .map(|y| row_text(&mut buf, area.width, y))
            .collect();
        assert!(text.contains('…'), "expected an ellipsis, got {text:?}");
    }

    #[test]
    fn skips_cards_that_do_not_fit() {
        let views = [
            card(NotificationLevel::Info, "Visible", "body"),
            card(NotificationLevel::Info, "Hidden", "body"),
        ];
        let area = Rect {
            x: 0,
            y: 0,
            width: 40,
            height: 4,
        };
        let mut buf = render(&views, area);

        let all: String = (0..area.height)
            .map(|y| row_text(&mut buf, area.width, y))
            .collect();
        assert!(all.contains("Visible"));
        assert!(!all.contains("Hidden"), "overflowing card was drawn");
    }

    #[test]
    fn used_height_counts_only_cards_that_fit() {
        let views = [
            card(NotificationLevel::Info, "One", "body"),
            card(NotificationLevel::Info, "Two", "body"),
        ];
        let area = Rect {
            x: 0,
            y: 2,
            width: 40,
            height: 4,
        };

        assert_eq!(NotificationWidget::new(&views).used_height(area), 4);
    }

    #[test]
    fn falls_back_to_level_name_when_header_is_blank() {
        let views = [card(NotificationLevel::Error, "", "disk full")];
        let area = Rect {
            x: 0,
            y: 0,
            width: 40,
            height: 4,
        };
        let mut buf = render(&views, area);

        assert!(row_text(&mut buf, area.width, 1).contains("Error"));
    }
}
