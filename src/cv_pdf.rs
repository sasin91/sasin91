//! Lays a `Cv` out on A4 and hands the placements to `pdf`. The geometry here
//! mirrors the `@page` rule in static/site.css so the printed measure matches
//! what /cv/ prints to, even though the two do not share a layout.

// `wrap` is not called from `main.rs` yet: Task 5 consumes it to lay out the CV.
// Until then `-D warnings` would fail the build on dead code that a later task
// gives a caller, so the allow stays until Task 5 lands.
#![allow(dead_code)]

// Import only what this task uses. `cargo clippy -- -D warnings` fails on an
// unused import, so `Cv` and `write_pdf` are added by Task 5 as they come
// into use, not up front.
use crate::pdf::{Font, POINTS_PER_MM, Page, Placement};

const PAGE_W_MM: f32 = 210.0;
const PAGE_H_MM: f32 = 297.0;
const MARGIN_X_MM: f32 = 16.0;
const MARGIN_Y_MM: f32 = 18.0;
const COLUMN_MM: f32 = PAGE_W_MM - 2.0 * MARGIN_X_MM;

/// One typeset line, already wrapped. `leading_mm` is the distance to the next
/// baseline, so a block's height is just the sum of its leadings.
struct Line {
    text: String,
    font: Font,
    size_pt: f32,
    leading_mm: f32,
    indent_mm: f32,
}

/// Flows lines down a sequence of pages. PDF's origin is the bottom-left, so
/// `y_mm` here is a real page coordinate that decreases as we move down, not a
/// distance from the top.
struct Cursor {
    pages: Vec<Page>,
    y_mm: f32,
}

impl Cursor {
    fn new() -> Self {
        Cursor {
            pages: vec![Page::default()],
            y_mm: PAGE_H_MM - MARGIN_Y_MM,
        }
    }

    fn remaining_mm(&self) -> f32 {
        self.y_mm - MARGIN_Y_MM
    }

    fn break_page(&mut self) {
        self.pages.push(Page::default());
        self.y_mm = PAGE_H_MM - MARGIN_Y_MM;
    }

    fn gap(&mut self, mm: f32) {
        // A gap must never be the reason a page breaks; leading whitespace at
        // the top of a fresh page reads as a mistake.
        if self.remaining_mm() > mm {
            self.y_mm -= mm;
        }
    }

    fn place(&mut self, lines: &[Line]) {
        for line in lines {
            if self.remaining_mm() < line.leading_mm {
                self.break_page();
            }
            self.y_mm -= line.leading_mm;
            let page = self.pages.last_mut().expect("a cursor always has a page");
            page.placements.push(Placement {
                x_mm: MARGIN_X_MM + line.indent_mm,
                y_mm: self.y_mm,
                size_pt: line.size_pt,
                font: line.font,
                text: line.text.clone(),
            });
        }
    }

    /// `break-inside: avoid`, arithmetically: measure the whole block, and if
    /// it will not fit in what is left, start the page before placing any of it.
    fn place_together(&mut self, lines: &[Line]) {
        let height: f32 = lines.iter().map(|l| l.leading_mm).sum();
        // A block taller than a whole page cannot be kept together; placing it
        // normally is better than an infinite supply of blank pages.
        let page_height = PAGE_H_MM - 2.0 * MARGIN_Y_MM;
        if height > self.remaining_mm() && height <= page_height {
            self.break_page();
        }
        self.place(lines);
    }

    fn finish(self) -> Vec<Page> {
        self.pages
    }
}

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

    fn body(text: &str) -> Line {
        Line {
            text: text.into(),
            font: Font::Helvetica,
            size_pt: 10.0,
            leading_mm: 14.0 / POINTS_PER_MM,
            indent_mm: 0.0,
        }
    }

    #[test]
    fn a_fresh_cursor_starts_below_the_top_margin() {
        let cursor = Cursor::new();
        assert!((cursor.remaining_mm() - (PAGE_H_MM - 2.0 * MARGIN_Y_MM)).abs() < 1e-3);
    }

    #[test]
    fn placing_lines_advances_down_the_page() {
        let mut cursor = Cursor::new();
        let before = cursor.remaining_mm();
        cursor.place(&[body("one"), body("two")]);
        let consumed = before - cursor.remaining_mm();
        assert!((consumed - 2.0 * (14.0 / POINTS_PER_MM)).abs() < 1e-3);
    }

    #[test]
    fn overflowing_lines_start_a_new_page() {
        let mut cursor = Cursor::new();
        let lines: Vec<Line> = (0..200).map(|i| body(&format!("line {i}"))).collect();
        cursor.place(&lines);
        let pages = cursor.finish();
        assert!(pages.len() > 1, "200 lines must not fit on one page");
        let placed: usize = pages.iter().map(|p| p.placements.len()).sum();
        assert_eq!(placed, 200, "no line may be dropped at a page break");
    }

    /// Every placement must sit inside the margins. A cursor that forgets to reset
    /// on a page break puts text below the bottom edge, where it is simply gone.
    #[test]
    fn every_placement_lands_inside_the_margins() {
        let mut cursor = Cursor::new();
        let lines: Vec<Line> = (0..200).map(|i| body(&format!("line {i}"))).collect();
        cursor.place(&lines);
        for page in cursor.finish() {
            for placement in &page.placements {
                assert!(
                    placement.y_mm >= MARGIN_Y_MM,
                    "placement below the bottom margin at {}mm",
                    placement.y_mm
                );
                assert!(
                    placement.y_mm <= PAGE_H_MM - MARGIN_Y_MM,
                    "placement above the top margin at {}mm",
                    placement.y_mm
                );
            }
        }
    }

    /// This is `break-inside: avoid` on .cv-role. A role split across a page break
    /// reads as two half-jobs.
    #[test]
    fn a_block_that_would_split_moves_to_the_next_page_whole() {
        let mut cursor = Cursor::new();
        // Fill most of the page, leaving room for about three lines.
        let filler_count = ((PAGE_H_MM - 2.0 * MARGIN_Y_MM) / (14.0 / POINTS_PER_MM)) as usize - 3;
        cursor.place(
            &(0..filler_count)
                .map(|i| body(&format!("f{i}")))
                .collect::<Vec<_>>(),
        );

        let block: Vec<Line> = (0..8).map(|i| body(&format!("block {i}"))).collect();
        cursor.place_together(&block);

        let pages = cursor.finish();
        assert_eq!(pages.len(), 2);
        for i in 0..8 {
            assert!(
                pages[1]
                    .placements
                    .iter()
                    .any(|p| p.text == format!("block {i}")),
                "block line {i} did not move to page 2 with the rest"
            );
        }
    }

    #[test]
    fn a_block_that_fits_is_not_pushed_to_a_new_page() {
        let mut cursor = Cursor::new();
        cursor.place_together(&[body("a"), body("b")]);
        assert_eq!(cursor.finish().len(), 1);
    }
}
