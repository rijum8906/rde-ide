//! Persistent state for the top bar widget.

/// Horizontal scroll offset for the tab strip.
///
/// Keeps the active tab visible when there are more tabs than fit in the
/// viewport. Scrolling is not implemented yet.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TopBarState {
    pub scroll_x: u16,
}

impl TopBarState {
    /// Create a state with no scrolling applied.
    pub const fn new() -> Self {
        Self { scroll_x: 0 }
    }

    /// Reset horizontal scroll back to zero.
    ///
    /// Intended to be called when the buffer list changes; not yet wired up.
    pub fn reset_scroll(&mut self) {
        self.scroll_x = 0;
    }

    /// Adjust the scroll offset so that the tab covering `tab_left` is
    /// visible in a viewport of `viewport_width`, clamping to the maximum.
    ///
    /// Intended to be called from the widget during render, when the
    /// viewport size is known. Not yet wired up.
    pub fn ensure_visible(
        &mut self,
        tab_left: u16,
        tab_width: u16,
        viewport_width: u16,
        total_width: u16,
    ) {
        let tab_right = tab_left.saturating_add(tab_width);

        if tab_left < self.scroll_x {
            self.scroll_x = tab_left;
        } else if tab_right > self.scroll_x.saturating_add(viewport_width) {
            self.scroll_x = tab_right.saturating_sub(viewport_width);
        }

        let max_scroll = total_width.saturating_sub(viewport_width);
        if self.scroll_x > max_scroll {
            self.scroll_x = max_scroll;
        }
    }
}
