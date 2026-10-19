//! Floating notification rendering.
//!
//! Notifications sit in the top-centre of the screen, over the layout.
//! [`render_notifications`] reserves the region, draws the stack, and reports
//! the bottom row so the toast can be placed below it instead of overlapping.

use ratatui::{Frame, layout::Rect};

use crate::{viewmodel::notification::NotificationView, widgets::notification::NotificationWidget};

/// Widest a notification card grows.
const MAX_WIDTH: u16 = 44;

/// Space kept between the card stack and the top edge.
const MARGIN: u16 = 2;

/// Screen size below which notifications are skipped entirely.
const MIN_WIDTH: u16 = 24;
const MIN_HEIGHT: u16 = 8;

/// Draw a stack of notification cards in the top-centre.
///
/// Returns the screen row just past the last card drawn, or `None` when
/// nothing was drawn.
pub fn render_notifications(views: &[NotificationView<'_>], frame: &mut Frame) -> Option<u16> {
    if views.is_empty() {
        return None;
    }

    let area = notification_area(frame.area())?;
    let widget = NotificationWidget::new(views);
    let used = widget.used_height(area);
    if used == 0 {
        return None;
    }

    frame.render_widget(widget, area);

    Some(area.y.saturating_add(used))
}

/// Region the card stack may occupy: top-centre, inset by [`MARGIN`].
fn notification_area(screen: Rect) -> Option<Rect> {
    if screen.width < MIN_WIDTH || screen.height < MIN_HEIGHT {
        return None;
    }

    let width = MAX_WIDTH.min(screen.width.saturating_sub(MARGIN * 2));

    Some(Rect {
        x: screen.x + (screen.width - width) / 2,
        y: screen.y + MARGIN,
        width,
        height: screen.height.saturating_sub(MARGIN * 2),
    })
}

#[cfg(test)]
mod tests {
    use super::notification_area;
    use ratatui::layout::Rect;

    #[test]
    fn places_the_stack_in_the_top_centre() {
        let area = notification_area(Rect {
            x: 0,
            y: 0,
            width: 100,
            height: 30,
        })
        .expect("a normal screen has a notification area");

        assert_eq!(area.width, 44);
        assert_eq!(area.x, 28);
        assert_eq!(area.y, 2);
        assert_eq!(area.height, 26);
    }

    #[test]
    fn keeps_the_card_inside_narrow_screens() {
        let area = notification_area(Rect {
            x: 0,
            y: 0,
            width: 30,
            height: 12,
        })
        .expect("30 columns is still enough");

        assert_eq!(area.width, 26);
        assert_eq!(area.x, 2);
    }

    #[test]
    fn skips_tiny_screens() {
        assert!(
            notification_area(Rect {
                x: 0,
                y: 0,
                width: 20,
                height: 10,
            })
            .is_none()
        );
        assert!(
            notification_area(Rect {
                x: 0,
                y: 0,
                width: 80,
                height: 5,
            })
            .is_none()
        );
    }
}
