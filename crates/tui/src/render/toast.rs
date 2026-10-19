//! Toast rendering.
//!
//! The toast sits in the top-right corner, below the notification stack when
//! one is visible. Its height depends on the wrapped body, so the rect is
//! measured from the widget before it is placed.

use ratatui::{Frame, layout::Rect};

use crate::{viewmodel::toast::ToastView, widgets::toast::ToastWidget};

/// Widest a toast grows.
const MAX_WIDTH: u16 = 44;

/// Space kept between the toast and the screen edges.
const MARGIN: u16 = 2;

/// Blank rows between the notification stack and the toast below it.
const GAP: u16 = 1;

/// Screen size below which the toast is skipped entirely.
const MIN_WIDTH: u16 = 24;
const MIN_HEIGHT: u16 = 8;

/// Draw the toast card in the top-right corner, starting below `below` (the
/// row just past the notification stack; `0` when there is none).
pub fn render_toast(view: &ToastView<'_>, frame: &mut Frame, below: u16) {
    let Some(area) = toast_area(frame.area(), view, below) else {
        return;
    };

    frame.render_widget(ToastWidget::new(view), area);
}

/// Top-right rect sized to the card's content.
fn toast_area(screen: Rect, view: &ToastView<'_>, below: u16) -> Option<Rect> {
    if screen.width < MIN_WIDTH || screen.height < MIN_HEIGHT {
        return None;
    }

    let width = MAX_WIDTH.min(screen.width.saturating_sub(MARGIN * 2));
    let height = ToastWidget::height(view, width);
    let y = if below == 0 {
        screen.y.saturating_add(MARGIN)
    } else {
        below.saturating_add(GAP)
    };

    if y.saturating_add(height) > screen.y.saturating_add(screen.height) {
        return None;
    }

    let x = screen
        .x
        .saturating_add(screen.width.saturating_sub(width + MARGIN));

    Some(Rect {
        x,
        y,
        width,
        height,
    })
}

#[cfg(test)]
mod tests {
    use super::toast_area;
    use crate::viewmodel::{level::Level, toast::ToastView};
    use ratatui::layout::Rect;

    fn view<'a>(header: &'a str, body: &'a str) -> ToastView<'a> {
        ToastView {
            level: Level::Info,
            header,
            body,
        }
    }

    fn screen() -> Rect {
        Rect {
            x: 0,
            y: 0,
            width: 100,
            height: 30,
        }
    }

    #[test]
    fn sits_in_the_top_right_corner() {
        let view = view("Opened", "src/main.rs");
        let area = toast_area(screen(), &view, 0).expect("a normal screen has room");

        assert_eq!(area.width, 44);
        assert_eq!(area.x, 54);
        assert_eq!(area.height, 4);
        assert_eq!(area.y, 2);
    }

    #[test]
    fn starts_below_the_notification_stack() {
        let view = view("Opened", "src/main.rs");
        let area = toast_area(screen(), &view, 10).expect("there is still room");

        assert_eq!(area.y, 11);
    }

    #[test]
    fn keeps_the_toast_inside_narrow_screens() {
        let view = view("Saved", "src/main.rs");
        let area = toast_area(
            Rect {
                x: 0,
                y: 0,
                width: 30,
                height: 12,
            },
            &view,
            0,
        )
        .expect("30 columns is enough");

        assert_eq!(area.width, 26);
        assert_eq!(area.x, 2);
    }

    #[test]
    fn skips_when_there_is_no_room_left() {
        let view = view("Saved", "src/main.rs");

        assert!(
            toast_area(
                Rect {
                    x: 0,
                    y: 0,
                    width: 20,
                    height: 10,
                },
                &view,
                0
            )
            .is_none()
        );
        assert!(toast_area(screen(), &view, 28).is_none());
    }
}
