//! Toast widget: one compact card anchored above the bottom bar.
//!
//! The card itself is shared with notifications (`widgets/card.rs`); the toast
//! only differs in that it shows a single message, clamps its body to two
//! lines, and is placed bottom-centre by `render/toast.rs`.

use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

use crate::{
    viewmodel::toast::ToastView,
    widgets::card::{card_height, render_card},
};

/// Body rows a toast may show before truncating; toasts stay compact.
const MAX_BODY_LINES: usize = 2;

/// The toast card.
#[derive(Debug)]
pub struct ToastWidget<'a> {
    view: &'a ToastView<'a>,
}

impl<'a> ToastWidget<'a> {
    pub fn new(view: &'a ToastView<'a>) -> Self {
        Self { view }
    }

    /// Rows the card needs when drawn `width` columns wide.
    pub fn height(view: &ToastView<'_>, width: u16) -> u16 {
        card_height(view.body, width, MAX_BODY_LINES)
    }
}

impl Widget for ToastWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 10 || area.height < 3 {
            return;
        }

        render_card(
            self.view.level,
            self.view.header,
            self.view.body,
            MAX_BODY_LINES,
            area,
            buf,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::viewmodel::level::Level;

    fn view<'a>(level: Level, header: &'a str, body: &'a str) -> ToastView<'a> {
        ToastView {
            level,
            header,
            body,
        }
    }

    fn render(view: &ToastView<'_>, area: Rect) -> Buffer {
        let mut buf = Buffer::empty(area);
        ToastWidget::new(view).render(area, &mut buf);
        buf
    }

    fn row_text(buf: &mut Buffer, width: u16, y: u16) -> String {
        (0..width)
            .map(|x| buf.cell_mut((x, y)).unwrap().symbol().to_string())
            .collect()
    }

    #[test]
    fn draws_header_and_body_in_a_rounded_card() {
        let view = view(
            Level::Error,
            "Command not found",
            "E492: Not an editor command: frobnicate",
        );
        let area = Rect {
            x: 0,
            y: 0,
            width: 60,
            height: 4,
        };
        let mut buf = render(&view, area);

        assert_eq!(buf.cell_mut((0, 0)).unwrap().symbol(), "╭");
        assert!(
            row_text(&mut buf, area.width, 1).contains("Command not found"),
            "header missing"
        );
        assert!(
            row_text(&mut buf, area.width, 2).contains("E492"),
            "body missing"
        );
        assert!(row_text(&mut buf, area.width, 1).contains("ERROR"));
    }

    #[test]
    fn height_grows_with_the_body() {
        let short = view(Level::Info, "Opened", "src/main.rs");
        let long = view(
            Level::Info,
            "Opened",
            "a much longer body that has to wrap onto two lines for sure",
        );

        assert_eq!(ToastWidget::height(&short, 40), 4);
        assert_eq!(ToastWidget::height(&long, 40), 5);
    }

    #[test]
    fn body_is_clamped_to_two_lines() {
        let body = "word ".repeat(80);
        let view = view(Level::Warning, "Long", &body);

        assert_eq!(ToastWidget::height(&view, 30), 5);
    }
}
