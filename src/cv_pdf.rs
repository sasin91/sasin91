//! Lays a `Cv` out on A4 and hands the placements to `pdf`. The geometry here
//! mirrors the `@page` rule in static/site.css so the printed measure matches
//! what /cv/ prints to, even though the two do not share a layout.

// `wrap` is not called from `main.rs` yet: Task 5 consumes it to lay out the CV.
// Until then `-D warnings` would fail the build on dead code that a later task
// gives a caller, so the allow stays until Task 5 lands.
#![allow(dead_code)]

// Import only what this task uses. `cargo clippy -- -D warnings` fails on an
// unused import, so `Cv`, `Page`, `Placement` and `write_pdf` are added by
// Tasks 4 and 5 as they come into use, not up front.
use crate::pdf::{Font, POINTS_PER_MM};

/// Break `text` into lines that each fit `width_mm`, greedily.
///
/// A single word wider than the column is emitted on its own line and allowed
/// to overrun: the alternative is hyphenating mid-word, and no word in a CV is
/// that long. Returning it rather than looping is what stops a pathological
/// input from hanging the build.
fn wrap(text: &str, font: Font, size_pt: f32, width_mm: f32) -> Vec<String> {
    let limit = width_mm * POINTS_PER_MM;
    let mut lines = Vec::new();
    let mut line = String::new();

    for word in text.split_whitespace() {
        if line.is_empty() {
            line.push_str(word);
            continue;
        }
        let candidate = format!("{line} {word}");
        if font.width(&candidate, size_pt) <= limit {
            line = candidate;
        } else {
            lines.push(std::mem::take(&mut line));
            line.push_str(word);
        }
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The longest single string in content/cv.toml. If anything overruns the
    /// column it is this.
    const LONGEST_ACHIEVEMENT: &str = "Built a comprehensive ranking and sorting \
        engine, delivering a quick and efficient method of finding candidates \
        and sorting by relevancy";

    #[test]
    fn no_wrapped_line_exceeds_the_column() {
        let lines = wrap(LONGEST_ACHIEVEMENT, Font::Helvetica, 10.0, 178.0);
        assert!(lines.len() > 1, "this text is meant to wrap");
        for line in &lines {
            let width = Font::Helvetica.width(line, 10.0);
            assert!(
                width <= 178.0 * POINTS_PER_MM,
                "line overruns the column at {width}pt: {line:?}"
            );
        }
    }

    #[test]
    fn wrapping_preserves_every_word_in_order() {
        let lines = wrap(LONGEST_ACHIEVEMENT, Font::Helvetica, 10.0, 178.0);
        assert_eq!(
            lines.join(" "),
            LONGEST_ACHIEVEMENT
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
        );
    }

    /// Greedy wrapping must actually fill the line -- an off-by-one that broke
    /// one word early would still pass the overrun test above while doubling
    /// the page count.
    #[test]
    fn wrapping_fills_the_line() {
        let lines = wrap(LONGEST_ACHIEVEMENT, Font::Helvetica, 10.0, 178.0);
        for pair in lines.windows(2) {
            let next_word = pair[1].split_whitespace().next().unwrap();
            let candidate = format!("{} {}", pair[0], next_word);
            assert!(
                Font::Helvetica.width(&candidate, 10.0) > 178.0 * POINTS_PER_MM,
                "line broke early, {candidate:?} would have fit"
            );
        }
    }

    #[test]
    fn a_word_wider_than_the_column_is_placed_rather_than_looping() {
        let lines = wrap(&"x".repeat(400), Font::Helvetica, 10.0, 178.0);
        assert_eq!(lines.len(), 1);
    }

    #[test]
    fn empty_text_produces_no_lines() {
        assert!(wrap("", Font::Helvetica, 10.0, 178.0).is_empty());
        assert!(wrap("   ", Font::Helvetica, 10.0, 178.0).is_empty());
    }
}
