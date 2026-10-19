//! Shared severity levels for floating widgets (notifications and toasts).
//!
//! The level decides the accent colour, the icon, and the short label a card
//! shows. Keeping it in one place is what makes a toast look like a
//! notification with a different placement.

use ratatui::style::Color;

/// Severity of a transient message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    Info,
    Success,
    Warning,
    Error,
    Panic,
}

impl Level {
    /// Short uppercase label shown at the right of a card header.
    pub fn label(self) -> &'static str {
        match self {
            Self::Info => "INFO",
            Self::Success => "DONE",
            Self::Warning => "WARN",
            Self::Error => "ERROR",
            Self::Panic => "PANIC",
        }
    }

    /// Human-readable name, used when a card has no header text.
    pub fn name(self) -> &'static str {
        match self {
            Self::Info => "Info",
            Self::Success => "Success",
            Self::Warning => "Warning",
            Self::Error => "Error",
            Self::Panic => "Panic",
        }
    }

    /// Single-width glyph drawn before the header.
    pub fn icon(self) -> &'static str {
        match self {
            Self::Info => "•",
            Self::Success => "✓",
            Self::Warning => "!",
            Self::Error => "×",
            Self::Panic => "▲",
        }
    }

    /// Accent colour for the border, icon, and level label.
    pub fn color(self) -> Color {
        match self {
            Self::Info => Color::Rgb(120, 170, 255),
            Self::Success => Color::Rgb(120, 210, 140),
            Self::Warning => Color::Rgb(230, 180, 80),
            Self::Error => Color::Rgb(240, 110, 110),
            Self::Panic => Color::Rgb(235, 120, 200),
        }
    }
}
