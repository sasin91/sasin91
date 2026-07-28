//! Lays a `Cv` out on A4 and hands the placements to `pdf`. The geometry here
//! mirrors the `@page` rule in static/site.css so the printed measure matches
//! what /cv/ prints to, even though the two do not share a layout.

// `render` is not called from `main.rs` yet: Task 6 wires it into the build.
// Until then `-D warnings` would fail on dead code that Task 6 gives a caller,
// so the allow stays until Task 6 lands.
#![allow(dead_code)]

use crate::cv::Cv;
use crate::pdf::{Font, POINTS_PER_MM, Page, Placement, write_pdf};

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

fn pt(points: f32) -> f32 {
    points / POINTS_PER_MM
}

/// Wrap `text` and return it as lines in one style. `indent_mm` shifts the
/// whole block right; `hanging` is the extra indent applied to every line
/// after the first, which is what makes a bullet's text align under itself
/// rather than under the bullet.
fn lines(
    text: &str,
    font: Font,
    size_pt: f32,
    leading_pt: f32,
    indent_mm: f32,
    hanging_mm: f32,
) -> Vec<Line> {
    wrap(text, font, size_pt, COLUMN_MM - indent_mm - hanging_mm)
        .into_iter()
        .enumerate()
        .map(|(i, text)| Line {
            text,
            font,
            size_pt,
            leading_mm: pt(leading_pt),
            indent_mm: indent_mm + if i == 0 { 0.0 } else { hanging_mm },
        })
        .collect()
}

const BULLET_INDENT_MM: f32 = 5.0;

/// The date range and location under a heading: "September 2024 - February
/// 2026 - Copenhagen". The web page uses an en dash between the dates and an
/// em dash before the location; both are WinAnsi, so they carry over.
fn meta(start: &str, end: Option<String>, location: &str) -> String {
    let end = end.unwrap_or_else(|| "present".into());
    format!("{start} \u{2013} {end} \u{2014} {location}")
}

/// A role or education entry as one keep-together block.
fn entry(heading: &str, meta_line: &str, body: Option<&str>, bullets: &[String]) -> Vec<Line> {
    let mut block = lines(heading, Font::HelveticaBold, 10.5, 14.0, 0.0, 0.0);
    block.extend(lines(meta_line, Font::Helvetica, 8.5, 12.0, 0.0, 0.0));
    if let Some(body) = body {
        block.extend(lines(body, Font::Helvetica, 10.0, 14.0, 0.0, 0.0));
    }
    for bullet in bullets {
        block.extend(lines(
            &format!("\u{2022}  {bullet}"),
            Font::Helvetica,
            10.0,
            14.0,
            BULLET_INDENT_MM,
            BULLET_INDENT_MM,
        ));
    }
    block
}

/// Lay `cv` out into pages. Separate from `render` so tests can assert on
/// placements rather than on parsed PDF bytes.
///
/// Infallible by construction: `Cv::validate` has already run by the time the
/// build reaches here, which is the same guarantee `Role::start_label()` leans
/// on when it unwraps.
///
/// This takes a `&Cv`, not a URL. That is the whole point -- the previous
/// implementation fetched /cv/ over HTTP and shipped the homepage as cv.pdf
/// for several deploys, a failure that is not expressible here.
fn layout(cv: &Cv) -> Vec<Page> {
    let mut cursor = Cursor::new();

    cursor.place(&lines(
        &cv.site.name,
        Font::HelveticaBold,
        20.0,
        24.0,
        0.0,
        0.0,
    ));
    cursor.place(&lines(
        &cv.site.title,
        Font::Helvetica,
        11.0,
        15.0,
        0.0,
        0.0,
    ));

    let contact = format!(
        "{}, {} \u{b7} {} \u{b7} {}",
        cv.contact.town, cv.contact.postcode, cv.contact.phone, cv.contact.email
    );
    cursor.place(&lines(&contact, Font::Helvetica, 9.0, 13.0, 0.0, 0.0));
    let links = format!("{} \u{b7} {}", cv.site.links.github, cv.site.links.linkedin);
    cursor.place(&lines(&links, Font::Helvetica, 9.0, 13.0, 0.0, 0.0));

    cursor.gap(pt(8.0));
    for paragraph in &cv.intro {
        cursor.place(&lines(paragraph, Font::Helvetica, 10.0, 14.0, 0.0, 0.0));
        cursor.gap(pt(4.0));
    }

    section(&mut cursor, "Experience");
    for role in &cv.roles {
        cursor.place_together(&entry(
            &format!("{} \u{b7} {}", role.title, role.company),
            &meta(&role.start_label(), role.end_label(), &role.location),
            Some(&role.summary),
            &role.achievements,
        ));
        cursor.gap(pt(6.0));
    }

    section(&mut cursor, "Skills");
    for skill in &cv.skills {
        cursor.place(&lines(
            &format!("\u{2022}  {}", skill.name),
            Font::Helvetica,
            10.0,
            14.0,
            BULLET_INDENT_MM,
            BULLET_INDENT_MM,
        ));
    }

    section(&mut cursor, "Education");
    if let Some(note) = &cv.education_note {
        cursor.place(&lines(note, Font::Helvetica, 10.0, 14.0, 0.0, 0.0));
        cursor.gap(pt(4.0));
    }
    for education in &cv.education {
        cursor.place_together(&entry(
            &format!("{} \u{b7} {}", education.title, education.school),
            &meta(
                &education.start_label(),
                education.end_label(),
                &education.location,
            ),
            education.note.as_deref(),
            &[],
        ));
        cursor.gap(pt(6.0));
    }

    cursor.finish()
}

/// Render `cv` as a complete PDF.
pub fn render(cv: &Cv) -> Vec<u8> {
    let title = format!("{} \u{2014} CV", cv.site.name);
    write_pdf(&title, PAGE_W_MM, PAGE_H_MM, &layout(cv))
}

/// A section heading. `break-after: avoid` from the print stylesheet, done by
/// reserving room for the heading plus a first line of content: a heading
/// stranded alone at the foot of a page reads as a section with nothing in it.
///
/// This breaks the page directly rather than delegating to
/// `Cursor::place_together`: `place_together` measures only the heading's own
/// height, so when the room left is between that height and `needed` it
/// declines to break, the heading lands on the old page, and the first
/// content line then overflows onto a new one -- the exact orphan this
/// function exists to prevent.
fn section(cursor: &mut Cursor, title: &str) {
    cursor.gap(pt(10.0));
    let heading = lines(title, Font::HelveticaBold, 11.0, 16.0, 0.0, 0.0);
    let needed: f32 = heading.iter().map(|l| l.leading_mm).sum::<f32>() + pt(14.0) * 2.0;
    if cursor.remaining_mm() < needed {
        cursor.break_page();
    }
    cursor.place(&heading);
    cursor.gap(pt(3.0));
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

    /// The real content, not a fixture. These tests are the replacement for the
    /// pdftotext guards the deploy workflow used to run, and they are only worth
    /// anything if they check what actually ships.
    fn real_cv() -> Cv {
        let src = std::fs::read_to_string("content/cv.toml").expect("content/cv.toml");
        let cv: Cv = toml::from_str(&src).expect("content/cv.toml must parse");
        cv.validate()
            .expect("content/cv.toml must have valid dates");
        cv
    }

    fn rendered_text(cv: &Cv) -> String {
        // The content streams are latin-1; lossy UTF-8 is enough to search for the
        // ASCII substrings these tests care about.
        String::from_utf8_lossy(&render(cv)).into_owned()
    }

    #[test]
    fn the_pdf_carries_every_section() {
        let text = rendered_text(&real_cv());
        for heading in ["Experience", "Skills", "Education"] {
            assert!(text.contains(heading), "missing section: {heading}");
        }
    }

    #[test]
    fn the_pdf_carries_the_name_and_contact_details() {
        let cv = real_cv();
        let text = rendered_text(&cv);
        assert!(text.contains(&cv.site.name));
        assert!(text.contains(&cv.contact.town));
        assert!(text.contains(&cv.contact.email));
    }

    /// Wrapping breaks achievements across lines, so a whole sentence will not
    /// appear contiguously. Checking the first few words of each is enough to
    /// prove no entry was dropped.
    #[test]
    fn the_pdf_carries_every_role_and_achievement() {
        let cv = real_cv();
        let text = rendered_text(&cv);
        for role in &cv.roles {
            assert!(
                text.contains(&role.company),
                "missing company: {}",
                role.company
            );
            assert!(text.contains(&role.title), "missing title: {}", role.title);
            for achievement in &role.achievements {
                let opening: String = achievement
                    .split_whitespace()
                    .take(4)
                    .collect::<Vec<_>>()
                    .join(" ");
                assert!(text.contains(&opening), "missing achievement: {opening}");
            }
        }
        for education in &cv.education {
            assert!(
                text.contains(&education.school),
                "missing school: {}",
                education.school
            );
        }
        for skill in &cv.skills {
            let opening: String = skill
                .name
                .split_whitespace()
                .take(3)
                .collect::<Vec<_>>()
                .join(" ");
            assert!(text.contains(&opening), "missing skill: {opening}");
        }
    }

    /// A CV is one to two pages. Three is the tolerance; anything beyond it means
    /// the layout is broken, not that the content grew.
    #[test]
    fn the_cv_fits_in_a_sane_number_of_pages() {
        let pages = layout(&real_cv()).len();
        assert!((1..=3).contains(&pages), "rendered {pages} pages");
    }

    /// Guards the one silent failure mode: a character with no WinAnsi form
    /// becomes '?' on a document sent to employers. An em dash or a curly
    /// apostrophe pasted into content/cv.toml must fail here, not in someone's
    /// inbox.
    #[test]
    fn every_character_in_the_real_cv_is_representable() {
        let src = std::fs::read_to_string("content/cv.toml").expect("content/cv.toml");
        let cv: Cv = toml::from_str(&src).expect("content/cv.toml must parse");
        let mut strings: Vec<&str> = vec![
            &cv.site.name,
            &cv.site.title,
            &cv.contact.town,
            &cv.contact.postcode,
            &cv.contact.phone,
            &cv.contact.email,
        ];
        strings.extend(cv.intro.iter().map(String::as_str));
        for role in &cv.roles {
            strings.extend([
                role.title.as_str(),
                role.company.as_str(),
                role.location.as_str(),
                role.summary.as_str(),
            ]);
            strings.extend(role.achievements.iter().map(String::as_str));
        }
        for education in &cv.education {
            strings.extend([
                education.title.as_str(),
                education.school.as_str(),
                education.location.as_str(),
            ]);
            if let Some(note) = &education.note {
                strings.push(note);
            }
        }
        for skill in &cv.skills {
            strings.push(&skill.name);
        }

        for s in strings {
            for c in s.chars() {
                assert!(
                    crate::pdf::winansi_byte(c).is_some(),
                    "{c:?} (U+{:04X}) in {s:?} has no WinAnsi form and would ship as '?'",
                    c as u32
                );
            }
        }
    }

    #[test]
    fn rendering_is_reproducible() {
        let cv = real_cv();
        assert_eq!(render(&cv), render(&cv));
    }

    /// `break-after: avoid` on h2. A heading alone at the foot of a page reads as
    /// a section with nothing in it.
    #[test]
    fn no_page_ends_on_a_section_heading() {
        for page in layout(&real_cv()) {
            let last = page
                .placements
                .last()
                .map(|p| p.text.as_str())
                .unwrap_or_default();
            for heading in ["Experience", "Skills", "Education"] {
                assert_ne!(last, heading, "page ends on the {heading} heading");
            }
        }
    }

    /// Two placements on the same baseline extract with no separator between
    /// them: a bold "Senior" at one x and a roman "Engineer" at the next comes
    /// back from pdfminer and pypdf as "SeniorEngineer", and an applicant
    /// tracking system matching "Senior Engineer" finds nothing.
    ///
    /// This layout emits exactly one placement per baseline, so it cannot happen
    /// today. The test exists because the obvious future change -- setting a role
    /// title in bold and its company in roman on one line -- reintroduces it
    /// silently, and the PDF looks perfectly correct while doing so.
    #[test]
    fn no_two_placements_share_a_baseline() {
        for (index, page) in layout(&real_cv()).into_iter().enumerate() {
            let mut baselines: Vec<String> = page
                .placements
                .iter()
                .map(|p| format!("{:.3}", p.y_mm))
                .collect();
            let before = baselines.len();
            baselines.sort();
            baselines.dedup();
            assert_eq!(
                baselines.len(),
                before,
                "page {index} has two placements on one baseline; they will \
                 extract as one run-together word"
            );
        }
    }
}
