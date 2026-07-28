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

/// The heading's own lines, with the vertical gap that precedes every
/// section already applied to the cursor. Returned rather than placed so the
/// caller can fold it into one `place_together` unit together with whatever
/// follows: measuring the heading alone -- even against a heuristic buffer
/// for "a first line of content" -- can't see whether the actual next block
/// fits, so a heading can still be orphaned one call frame later. Measuring
/// heading and first block together removes the heuristic instead of tuning it.
fn section_heading(cursor: &mut Cursor, title: &str) -> Vec<Line> {
    cursor.gap(pt(10.0));
    lines(title, Font::HelveticaBold, 11.0, 16.0, 0.0, 0.0)
}

/// Fold `gap_mm` into the first line of `block`, then append `block` onto
/// `heading`. The combined vector's total leading is exactly what placing the
/// heading, then `Cursor::gap(gap_mm)`, then `block` in sequence would have
/// consumed, so handing it to `Cursor::place_together` measures heading and
/// first block as a single unbreakable unit.
fn keep_with_heading(mut heading: Vec<Line>, gap_mm: f32, mut block: Vec<Line>) -> Vec<Line> {
    if let Some(first) = block.first_mut() {
        first.leading_mm += gap_mm;
    }
    heading.append(&mut block);
    heading
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

    let role_block = |role: &crate::cv::Role| {
        entry(
            &format!("{} \u{b7} {}", role.title, role.company),
            &meta(&role.start_label(), role.end_label(), &role.location),
            Some(&role.summary),
            &role.achievements,
        )
    };
    let heading_lines = section_heading(&mut cursor, "Experience");
    let mut roles = cv.roles.iter();
    match roles.next() {
        Some(first) => {
            cursor.place_together(&keep_with_heading(
                heading_lines,
                pt(3.0),
                role_block(first),
            ));
            cursor.gap(pt(6.0));
        }
        None => cursor.place(&heading_lines),
    }
    for role in roles {
        cursor.place_together(&role_block(role));
        cursor.gap(pt(6.0));
    }

    let skill_block = |skill: &crate::cv::Skill| {
        lines(
            &format!("\u{2022}  {}", skill.name),
            Font::Helvetica,
            10.0,
            14.0,
            BULLET_INDENT_MM,
            BULLET_INDENT_MM,
        )
    };
    let heading_lines = section_heading(&mut cursor, "Skills");
    let mut skills = cv.skills.iter();
    match skills.next() {
        Some(first) => cursor.place_together(&keep_with_heading(
            heading_lines,
            pt(3.0),
            skill_block(first),
        )),
        None => cursor.place(&heading_lines),
    }
    for skill in skills {
        cursor.place(&skill_block(skill));
    }

    let edu_block = |education: &crate::cv::Education| {
        entry(
            &format!("{} \u{b7} {}", education.title, education.school),
            &meta(
                &education.start_label(),
                education.end_label(),
                &education.location,
            ),
            education.note.as_deref(),
            &[],
        )
    };
    let heading_lines = section_heading(&mut cursor, "Education");
    if let Some(note) = &cv.education_note {
        let note_block = lines(note, Font::Helvetica, 10.0, 14.0, 0.0, 0.0);
        cursor.place_together(&keep_with_heading(heading_lines, pt(3.0), note_block));
        cursor.gap(pt(4.0));
        for education in &cv.education {
            cursor.place_together(&edu_block(education));
            cursor.gap(pt(6.0));
        }
    } else {
        let mut educations = cv.education.iter();
        match educations.next() {
            Some(first) => {
                cursor.place_together(&keep_with_heading(heading_lines, pt(3.0), edu_block(first)));
                cursor.gap(pt(6.0));
            }
            None => cursor.place(&heading_lines),
        }
        for education in educations {
            cursor.place_together(&edu_block(education));
            cursor.gap(pt(6.0));
        }
    }

    cursor.finish()
}

/// Render `cv` as a complete PDF.
pub fn render(cv: &Cv) -> Vec<u8> {
    let title = format!("{} \u{2014} CV", cv.site.name);
    write_pdf(&title, PAGE_W_MM, PAGE_H_MM, &layout(cv))
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

    /// The placements `layout` produces, joined with a separator that a
    /// wrapped fragment can never straddle -- each `Placement` is already one
    /// wrapped line, so joining them the way the two-word boundary between
    /// lines already works (a hard break, never a space) keeps a short opening
    /// fragment from false-matching across two unrelated lines.
    fn placement_text(cv: &Cv) -> String {
        layout(cv)
            .into_iter()
            .flat_map(|page| page.placements.into_iter().map(|p| p.text))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// The opening `words` words of `text`, exactly as `the_pdf_carries_every_role_and_achievement`'s
    /// predecessor checked achievements: short enough to survive wrapping onto
    /// one line, long enough that a coincidence elsewhere in the CV is
    /// implausible.
    fn opening(text: &str, words: usize) -> String {
        text.split_whitespace()
            .take(words)
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Every field that reaches the page must actually be there -- not a
    /// sample of them. Built from `layout()`, not from rendered PDF bytes:
    /// `education.title` ("Strøm, styring & IT") and `role.location`
    /// ("Næstved", "Høng") carry Danish letters that `write_pdf` encodes as
    /// single WinAnsi/latin-1 bytes (0xF8 for 'ø', for instance), and
    /// `String::from_utf8_lossy` turns an isolated byte like that into
    /// U+FFFD -- a byte-search over rendered PDF text would fail on those
    /// fields for the wrong reason.
    ///
    /// Contact and links are checked against the exact composed line
    /// `layout` builds, not a bare substring of one field: `cv.contact.town`
    /// ("Slagelse") and `cv.site.title` ("Software developer") each recur
    /// elsewhere in the CV (a role's location, a role's title), so a bare
    /// substring check still passes even if the contact line itself were
    /// dropped from the layout entirely. The header, every role heading and
    /// every education heading are checked the same way -- `cv.site.title`
    /// alone would still false-pass via the Syncronet role's identical
    /// title, and `role.title`/`role.company` alone would still false-pass
    /// via "Web developer" (three roles) and "JUICE ApS" (two stints) -- and
    /// role and education entries are additionally checked against their
    /// full `meta()` line, for the same reason: "Copenhagen" and "Slagelse"
    /// each appear as more than one entry's location.
    #[test]
    fn the_pdf_carries_every_field_that_layout_places() {
        let cv = real_cv();
        let text = placement_text(&cv);

        // `cv.site.name` and `cv.site.title` are adjacent placements, joined
        // by `placement_text`'s "\n" exactly like any other two consecutive
        // lines; composing them here is what makes a dropped title fail
        // instead of quietly matching the Syncronet role's identical text.
        let header = format!("{}\n{}", cv.site.name, cv.site.title);
        assert!(text.contains(&header), "missing header: {header:?}");

        let contact = format!(
            "{}, {} \u{b7} {} \u{b7} {}",
            cv.contact.town, cv.contact.postcode, cv.contact.phone, cv.contact.email
        );
        assert!(text.contains(&contact), "missing contact line: {contact}");

        let links = format!("{} \u{b7} {}", cv.site.links.github, cv.site.links.linkedin);
        assert!(text.contains(&links), "missing links line: {links}");

        for paragraph in &cv.intro {
            let frag = opening(paragraph, 5);
            assert!(text.contains(&frag), "missing intro paragraph: {frag}");
        }

        for role in &cv.roles {
            // The composed heading `layout` actually places, not the two
            // fields separately: "Web developer" (three roles) and "JUICE
            // ApS" (two stints) each recur, so checking `title` or `company`
            // alone would still pass with one role's heading deleted.
            let heading = format!("{} \u{b7} {}", role.title, role.company);
            assert!(text.contains(&heading), "missing role heading: {heading}");
            let meta_line = meta(&role.start_label(), role.end_label(), &role.location);
            assert!(
                text.contains(&meta_line),
                "missing role meta line: {meta_line}"
            );
            let summary = opening(&role.summary, 4);
            assert!(text.contains(&summary), "missing role summary: {summary}");
            for achievement in &role.achievements {
                let frag = opening(achievement, 4);
                assert!(text.contains(&frag), "missing achievement: {frag}");
            }
        }

        for skill in &cv.skills {
            let frag = opening(&skill.name, 3);
            assert!(text.contains(&frag), "missing skill: {frag}");
        }

        for education in &cv.education {
            // Same reasoning as the role heading above: the composed string
            // `entry(...)` actually places, not `title`/`school` separately.
            let heading = format!("{} \u{b7} {}", education.title, education.school);
            assert!(
                text.contains(&heading),
                "missing education heading: {heading}"
            );
            let meta_line = meta(
                &education.start_label(),
                education.end_label(),
                &education.location,
            );
            assert!(
                text.contains(&meta_line),
                "missing education meta line: {meta_line}"
            );
            if let Some(note) = &education.note {
                let frag = opening(note, 4);
                assert!(text.contains(&frag), "missing education note: {frag}");
            }
        }

        if let Some(note) = &cv.education_note {
            let frag = opening(note, 4);
            assert!(text.contains(&frag), "missing education_note: {frag}");
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
    ///
    /// Driven off `layout()`'s placements rather than an enumerated field
    /// list: a field list drifts the moment a new one starts reaching the
    /// page (`site.links.github`/`linkedin` and `education_note` were both
    /// rendered without ever being checked here), while every placement is,
    /// by construction, every character that ships.
    #[test]
    fn every_character_in_the_real_cv_is_representable() {
        for placement in layout(&real_cv()).iter().flat_map(|page| &page.placements) {
            for c in placement.text.chars() {
                assert!(
                    crate::pdf::winansi_byte(c).is_some(),
                    "{c:?} (U+{:04X}) in {:?} has no WinAnsi form and would ship as '?'",
                    c as u32,
                    placement.text
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
