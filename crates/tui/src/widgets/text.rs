//! Text layout helpers shared by the floating card widgets.

/// Greedy word wrap on character counts, hard-splitting words wider than the
/// line. Words are kept intact where they fit.
pub(crate) fn wrap(text: &str, width: usize) -> Vec<String> {
    if width == 0 {
        return Vec::new();
    }

    let mut lines = Vec::new();
    let mut current = String::new();

    for word in text.split_whitespace() {
        let mut remaining = word;

        // A word that does not fit after the current words starts a new line.
        if !current.is_empty() && current.chars().count() + 1 + remaining.chars().count() > width {
            lines.push(std::mem::take(&mut current));
        }

        while !remaining.is_empty() {
            let used = current.chars().count();
            let space = usize::from(used > 0);
            let available = width.saturating_sub(used + space);

            if available == 0 {
                lines.push(std::mem::take(&mut current));
                continue;
            }

            let take = available.min(remaining.chars().count());
            let split = remaining
                .char_indices()
                .nth(take)
                .map(|(index, _)| index)
                .unwrap_or(remaining.len());

            if space == 1 {
                current.push(' ');
            }
            current.push_str(&remaining[..split]);
            remaining = &remaining[split..];

            if !remaining.is_empty() {
                lines.push(std::mem::take(&mut current));
            }
        }
    }

    if !current.is_empty() {
        lines.push(current);
    }

    lines
}

/// Keep at most `max_lines`, ending the last kept line with an ellipsis when
/// text was dropped.
pub(crate) fn clamp(mut lines: Vec<String>, max_lines: usize, width: usize) -> Vec<String> {
    if lines.len() > max_lines {
        lines.truncate(max_lines);
        if let Some(last) = lines.last_mut() {
            while last.chars().count() >= width {
                last.pop();
            }
            last.push('…');
        }
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::{clamp, wrap};

    #[test]
    fn wraps_on_word_boundaries() {
        assert_eq!(wrap("alpha beta gamma", 11), ["alpha beta", "gamma"]);
    }

    #[test]
    fn splits_words_wider_than_the_line() {
        assert_eq!(wrap("abcdefgh", 3), ["abc", "def", "gh"]);
    }

    #[test]
    fn clamps_long_text_with_an_ellipsis() {
        let lines = clamp(wrap("one two three four five six", 5), 1, 5);

        assert_eq!(lines.len(), 1);
        assert!(lines[0].ends_with('…'), "{:?}", lines[0]);
        assert!(lines[0].chars().count() <= 5);
    }
}
