# cv.pdf in the site binary — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Generate `public/cv.pdf` directly from the `Cv` struct during `cargo run --release`, and delete the headless-Chrome step from `.github/workflows/deploy.yml`.

**Architecture:** Three new modules. `src/pdf_metrics.rs` holds machine-derived Adobe Core-14 glyph widths as pure data. `src/pdf.rs` is a CV-agnostic writer: it measures and encodes text, and serializes absolutely-positioned placements into PDF bytes. `src/cv_pdf.rs` lays a `&Cv` out on A4 — wrapping, pagination, keep-together — and calls the writer. `main.rs` writes the result next to `public/cv/index.html`.

**Tech Stack:** Rust 2024 edition, no new dependencies. Tests are in-module `#[cfg(test)]` blocks, matching `src/cv.rs`.

**Spec:** `docs/superpowers/specs/2026-07-28-cv-pdf-writer-design.md`

## Global Constraints

- **No new crate dependencies.** `Cargo.toml` is unchanged by this work. The whole point is fewer moving parts.
- **`cargo fmt --check` and `cargo clippy -- -D warnings` must pass.** CI runs both and treats warnings as errors.
- **Page geometry is fixed:** A4, 210mm × 297mm. Margins 18mm top and bottom, 16mm left and right — the same numbers as the `@page` rule in `static/site.css`. Text column is therefore **178mm** wide and **261mm** tall.
- **`POINTS_PER_MM = 72.0 / 25.4`** (≈ 2.834645). PDF user space is points with origin at the **bottom-left**; the layout cursor measures from the top and converts.
- **Type scale** (chosen once here, referenced by later tasks):

  | Element | Font | Size | Leading |
  |---|---|---|---|
  | Name | Helvetica-Bold | 20pt | 24pt |
  | Title line | Helvetica | 11pt | 15pt |
  | Contact line | Helvetica | 9pt | 13pt |
  | Section heading | Helvetica-Bold | 11pt | 16pt |
  | Role/education heading | Helvetica-Bold | 10.5pt | 14pt |
  | Meta line (dates, location) | Helvetica | 8.5pt | 12pt |
  | Body, summary, bullets | Helvetica | 10pt | 14pt |

- **`render` is infallible.** Dates are already validated by `Cv::validate` before the build reaches this code — the same guarantee `Role::start_label()` relies on when it unwraps.
- **Comment style:** this repo's comments explain *why*, often naming the bug the code prevents (see `src/cv.rs:27-35` and the `@media print` block in `static/site.css`). Match that. Do not add comments that restate the code.
- **Do not touch `.github/workflows/deploy.yml` before Task 7 passes.** Task 7 is a human gate.

---

## Prerequisites

Two background investigations feed this plan. Both write to the session scratchpad:

- `scratchpad/afm_widths.rs` — the derived `HELVETICA` and `HELVETICA_BOLD` width tables. **Task 1 consumes this file. Do not hand-write or "correct" the tables.**

  Provenance, for the module doc comment in Task 1: derived from Adobe's 1997
  Core-14 AFM metrics (`StartFontMetrics 4.1`), taken from Apache PDFBox's
  bundled copies, and cross-checked against URW's Nimbus Sans AFMs (an
  independent metrically-compatible lineage) and Mozilla pdf.js's separately
  transcribed metrics table. All three agree on all 224 code points for both
  faces.

  Built by resolving each WinAnsi code point to a glyph **name** per PDF
  32000-1 Annex D.2, then looking that name up in the AFM's `N` field. The
  AFM's `C` column is AdobeStandardEncoding and carries `-1` for 66 of the 224
  WinAnsi positions — every accented Latin-1 letter among them. Indexing by `C`
  drops `æ`, `ø` and `å`, i.e. `Næstved`, `Høng` and `Strøm`. The generator is
  at `scratchpad/build.py` if the tables ever need regenerating.
The reference writer this work starts from is at `scratchpad/pdf_sample.rs`. It
has been reviewed against ISO 32000-1 by porting it to Python and attacking the
generated files with pypdf (strict mode) and pdfminer.six. **The findings are
inlined into Task 2 below** — there is nothing further to read.

Two of that review's findings are already designed out by Task 1 and need no
work in Task 2:

- The sample's `winansi()` handles only 6 of the 27 assigned CP-1252 code
  points in 0x80–0x9F. The other 21 — including **bullet**, the *opening* curly
  quotes, the OE ligature and the trademark sign — fall through to `?`. Every
  bullet in the CV would render as a literal question mark. Task 1's
  `winansi_byte` carries all 27.
- The sample's `c if (c as u32) < 0x100 => c as u8` arm is Latin-1, not
  WinAnsi. It emits raw C1 control bytes that a viewer then reads back as
  typographic characters (U+0085 becomes `…`, U+0092 becomes `’`), and five of
  them have no glyph at all. Task 1 passes through only 0x20–0x7E and
  0xA0–0xFF, and returns `None` for everything else — which is also why C0
  control characters cannot reach a content stream raw.

One deliberate imprecision, noted so nobody "fixes" it later: U+00A0 maps to
0xA0, which WinAnsi defines as `space` rather than a non-breaking space, and
U+00AD maps to 0xAD, which WinAnsi defines as `hyphen` rather than a soft
hyphen. Neither appears in `content/cv.toml`, and both degrade sensibly.

---

## File Structure

| File | Responsibility |
|---|---|
| `src/pdf_metrics.rs` (create) | Pure data: two `[u16; 224]` width tables, WinAnsi-indexed. No logic. |
| `src/pdf.rs` (create) | WinAnsi encoding, text measurement, `Placement`/`Page`, `write_pdf`. Knows nothing about CVs. |
| `src/cv_pdf.rs` (create) | A4 layout of a `&Cv`: wrapping, pagination, keep-together. `pub fn render(cv: &Cv) -> Vec<u8>`. |
| `src/main.rs` (modify) | Declare the modules; write `public/cv.pdf`. |
| `.github/workflows/deploy.yml` (modify) | Delete the `Render the CV to PDF` step; add `cv.pdf` to the required-files check. |

---

### Task 1: WinAnsi encoding and font metrics

The foundation everything else measures against. A wrong width here shows up as ragged wrapping in the final PDF, so this task's tests are the ones that earn their keep.

**Files:**
- Create: `src/pdf_metrics.rs`
- Create: `src/pdf.rs`
- Modify: `src/main.rs:7-12` (module declarations)

**Interfaces:**
- Consumes: `scratchpad/afm_widths.rs` (the derived width tables).
- Produces:
  - `pdf_metrics::HELVETICA: [u16; 224]`, `pdf_metrics::HELVETICA_BOLD: [u16; 224]`
  - `pdf::Font` — `enum Font { Helvetica, HelveticaBold }`, `Copy + Clone + PartialEq + Eq`
  - `pdf::Font::width(self, text: &str, size_pt: f32) -> f32` — returns **points**. Takes `self` by value, not `&self`: `Font` is `Copy`, and `clippy::trivially_copy_pass_by_ref` fails the build otherwise.
  - `pdf::winansi_byte(c: char) -> Option<u8>` — `None` means the character has no WinAnsi representation
  - `pdf::POINTS_PER_MM: f32`

- [ ] **Step 1: Copy in the derived width tables**

Copy `scratchpad/afm_widths.rs` to `src/pdf_metrics.rs`. Add a module doc comment recording where the data came from and why it is not hand-maintained:

```rust
//! Adobe Core-14 glyph widths for Helvetica and Helvetica-Bold, in 1/1000 em,
//! indexed by WinAnsi code point minus 32 (index 0 is 0x20 space, index 223 is
//! 0xFF ydieresis). Derived from the published AFM metrics, not measured and not
//! guessed: a width that is wrong by a few thousandths shows up as a line that
//! wraps a word early or overruns the margin, and nothing else in the build
//! would catch it. Positions with no glyph in WinAnsi hold 0.
//!
//! Source: see the plan at docs/superpowers/plans/2026-07-28-cv-pdf-writer.md
```

Verify the arrays are exactly 224 entries — do not eyeball this, let the compiler check it by keeping the explicit `[u16; 224]` type annotation.

- [ ] **Step 2: Write the failing tests**

Create `src/pdf.rs` containing only the test module for now:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_width_tables_match_the_published_metrics() {
        assert_eq!(crate::pdf_metrics::HELVETICA[('A' as usize) - 32], 667);
        assert_eq!(crate::pdf_metrics::HELVETICA[(' ' as usize) - 32], 278);
        assert_eq!(crate::pdf_metrics::HELVETICA[('M' as usize) - 32], 833);
        assert_eq!(crate::pdf_metrics::HELVETICA[('i' as usize) - 32], 222);
        assert_eq!(crate::pdf_metrics::HELVETICA_BOLD[('A' as usize) - 32], 722);
        assert_eq!(crate::pdf_metrics::HELVETICA_BOLD[(' ' as usize) - 32], 278);
    }

    /// Date ranges sit in a column on the page only if every digit is the same
    /// width. Helvetica's are; asserting it here means a corrupted table shows
    /// up as a test failure rather than as a wobbling "September 2024" column.
    #[test]
    fn digits_are_tabular_in_both_faces() {
        for table in [
            &crate::pdf_metrics::HELVETICA,
            &crate::pdf_metrics::HELVETICA_BOLD,
        ] {
            let zero = table[('0' as usize) - 32];
            assert_ne!(zero, 0);
            for digit in '1'..='9' {
                assert_eq!(table[(digit as usize) - 32], zero, "digit {digit}");
            }
        }
    }

    /// Every character that can appear on the page must have a width. A zero
    /// here would silently collapse a glyph to nothing when wrapping.
    #[test]
    fn every_printable_winansi_code_point_has_a_width() {
        for code in 0x20u8..=0xFFu8 {
            // The six unassigned WinAnsi positions legitimately hold zero.
            if matches!(code, 0x7F | 0x81 | 0x8D | 0x8F | 0x90 | 0x9D) {
                continue;
            }
            let index = code as usize - 32;
            assert_ne!(
                crate::pdf_metrics::HELVETICA[index], 0,
                "Helvetica has no width for 0x{code:02X}"
            );
            assert_ne!(
                crate::pdf_metrics::HELVETICA_BOLD[index], 0,
                "Helvetica-Bold has no width for 0x{code:02X}"
            );
        }
    }

    #[test]
    fn width_is_in_points_and_scales_with_size() {
        let at_one = Font::Helvetica.width("A", 1.0);
        assert!((at_one - 0.667).abs() < 1e-6, "got {at_one}");
        let at_twelve = Font::Helvetica.width("A", 12.0);
        assert!((at_twelve - 0.667 * 12.0).abs() < 1e-4, "got {at_twelve}");
    }

    #[test]
    fn width_sums_across_a_string() {
        // "AB" = 667 + 667 thousandths of an em.
        let expected = (667.0 + 667.0) / 1000.0 * 10.0;
        assert!((Font::Helvetica.width("AB", 10.0) - expected).abs() < 1e-4);
    }

    #[test]
    fn danish_letters_encode_as_latin1() {
        assert_eq!(winansi_byte('æ'), Some(0xE6));
        assert_eq!(winansi_byte('ø'), Some(0xF8));
        assert_eq!(winansi_byte('å'), Some(0xE5));
        assert_eq!(winansi_byte('Ø'), Some(0xD8));
        assert_eq!(winansi_byte('Æ'), Some(0xC6));
    }

    /// 0x80-0x9F is the range where WinAnsi (CP-1252) and ISO-8859-1 disagree.
    /// Treating them as the same puts C1 control bytes on the page where
    /// punctuation belongs, and text extractors return mojibake for it.
    #[test]
    fn cp1252_punctuation_is_not_treated_as_latin1() {
        assert_eq!(winansi_byte('€'), Some(0x80));
        assert_eq!(winansi_byte('\u{2019}'), Some(0x92)); // right single quote
        assert_eq!(winansi_byte('\u{201C}'), Some(0x93)); // left double quote
        assert_eq!(winansi_byte('\u{201D}'), Some(0x94)); // right double quote
        assert_eq!(winansi_byte('\u{2022}'), Some(0x95)); // bullet
        assert_eq!(winansi_byte('\u{2013}'), Some(0x96)); // en dash
        assert_eq!(winansi_byte('\u{2014}'), Some(0x97)); // em dash
        assert_eq!(winansi_byte('\u{2026}'), Some(0x85)); // ellipsis
    }

    /// The middot separates stack items and role headings on the page.
    #[test]
    fn the_middot_encodes() {
        assert_eq!(winansi_byte('·'), Some(0xB7));
    }

    /// Reporting the gap rather than substituting '?' is what lets a test
    /// assert the real content/cv.toml is representable before it ships.
    #[test]
    fn characters_outside_winansi_are_reported_not_guessed() {
        assert_eq!(winansi_byte('\u{2192}'), None); // rightwards arrow
        assert_eq!(winansi_byte('日'), None);
        assert_eq!(winansi_byte('\u{0100}'), None); // A with macron
    }
}
```

- [ ] **Step 3: Run the tests to verify they fail**

Add `mod pdf;` and `mod pdf_metrics;` to `src/main.rs` alongside the existing `mod cv;` at line 8, keeping the list alphabetical:

```rust
mod content;
mod cv;
mod djot;
mod highlight;
mod html;
mod math;
mod pdf;
mod pdf_metrics;
```

Run: `cargo test pdf`
Expected: compile errors — `Font`, `winansi_byte` not found.

- [ ] **Step 4: Implement the encoding and metrics**

Prepend to `src/pdf.rs`:

```rust
//! A minimal PDF writer: absolute-positioned text in the two Helvetica faces,
//! serialized to bytes. It knows nothing about what it is typesetting -- see
//! `cv_pdf` for the layout that drives it.

use crate::pdf_metrics::{HELVETICA, HELVETICA_BOLD};

pub const POINTS_PER_MM: f32 = 72.0 / 25.4;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Font {
    Helvetica,
    HelveticaBold,
}

impl Font {
    fn widths(self) -> &'static [u16; 224] {
        match self {
            Font::Helvetica => &HELVETICA,
            Font::HelveticaBold => &HELVETICA_BOLD,
        }
    }

    /// The width of `text` in points at `size_pt`. Characters with no WinAnsi
    /// representation are measured as the '?' they will be rendered as, so the
    /// measurement never disagrees with what lands on the page.
    pub fn width(self, text: &str, size_pt: f32) -> f32 {
        let widths = self.widths();
        let thousandths: u32 = text
            .chars()
            .map(|c| {
                let byte = winansi_byte(c).unwrap_or(b'?');
                u32::from(widths[byte as usize - 32])
            })
            .sum();
        thousandths as f32 / 1000.0 * size_pt
    }
}

/// The WinAnsiEncoding byte for `c`, or `None` if WinAnsi cannot represent it.
///
/// WinAnsi is CP-1252, not ISO-8859-1: the two agree everywhere except
/// 0x80-0x9F, where Latin-1 has C1 controls and CP-1252 has the punctuation
/// people actually type (curly quotes, dashes, the bullet). Mapping that range
/// by code point -- the obvious `c as u8` shortcut -- puts control bytes on the
/// page and makes text extractors return mojibake, which matters because
/// recruiters' tooling parses this file.
///
/// Returning `None` rather than substituting '?' is deliberate: it lets a test
/// prove the real content/cv.toml is representable, instead of discovering the
/// substitution in a PDF already sent to an employer.
pub fn winansi_byte(c: char) -> Option<u8> {
    let byte = match c {
        '\u{20AC}' => 0x80, // euro
        '\u{201A}' => 0x82, // single low quote
        '\u{0192}' => 0x83, // florin
        '\u{201E}' => 0x84, // double low quote
        '\u{2026}' => 0x85, // ellipsis
        '\u{2020}' => 0x86, // dagger
        '\u{2021}' => 0x87, // double dagger
        '\u{02C6}' => 0x88, // circumflex
        '\u{2030}' => 0x89, // per mille
        '\u{0160}' => 0x8A, // S caron
        '\u{2039}' => 0x8B, // single left guillemet
        '\u{0152}' => 0x8C, // OE
        '\u{017D}' => 0x8E, // Z caron
        '\u{2018}' => 0x91, // left single quote
        '\u{2019}' => 0x92, // right single quote
        '\u{201C}' => 0x93, // left double quote
        '\u{201D}' => 0x94, // right double quote
        '\u{2022}' => 0x95, // bullet
        '\u{2013}' => 0x96, // en dash
        '\u{2014}' => 0x97, // em dash
        '\u{02DC}' => 0x98, // small tilde
        '\u{2122}' => 0x99, // trademark
        '\u{0161}' => 0x9A, // s caron
        '\u{203A}' => 0x9B, // single right guillemet
        '\u{0153}' => 0x9C, // oe
        '\u{017E}' => 0x9E, // z caron
        '\u{0178}' => 0x9F, // Y diaeresis
        // Below 0x80 and from 0xA0 up, WinAnsi and Latin-1 agree. The C1 range
        // and DEL have no glyph, so a raw control character is not representable.
        c if (c as u32) >= 0x20 && (c as u32) < 0x7F => c as u8,
        c if (c as u32) >= 0xA0 && (c as u32) < 0x100 => c as u8,
        _ => return None,
    };
    Some(byte)
}
```

- [ ] **Step 5: Run the tests to verify they pass**

Run: `cargo test pdf`
Expected: all tests in the `pdf` and `pdf_metrics` modules PASS.

Then run: `cargo fmt --check && cargo clippy -- -D warnings`
Expected: clean. `clippy` will object to `&self` on `Font::widths` taking a `Copy` type by reference — take `self` by value if it does.

- [ ] **Step 6: Commit**

```bash
git add src/pdf.rs src/pdf_metrics.rs src/main.rs
git commit -m "feat(pdf): WinAnsi encoding and Helvetica metrics"
```

---

### Task 2: The PDF writer

Turns positioned strings into a valid file. Structural correctness only — no layout decisions live here.

**Files:**
- Modify: `src/pdf.rs`
- Reference: `scratchpad/pdf_sample.rs` (the starting point)

**Interfaces:**
- Consumes: `Font`, `winansi_byte`, `POINTS_PER_MM` from Task 1.
- Produces:
  - `pdf::Placement { x_mm: f32, y_mm: f32, size_pt: f32, font: Font, text: String }`
  - `pdf::Page { placements: Vec<Placement> }`, deriving `Default`
  - `pdf::write_pdf(title: &str, width_mm: f32, height_mm: f32, pages: &[Page]) -> Vec<u8>`

**Defects to fix while porting.** The sample is structurally sound — its xref
table, free list, 20-byte entry format, `/Length` accounting, paren escaping,
binary header comment and `BT…Tf…Tm…Tj…ET` sequence were all verified correct
against ISO 32000-1, and `/ProcSet` is correctly *absent* (the spec marks it
obsolescent). Do not "fix" any of those. These six are real:

| # | Defect | Consequence |
|---|---|---|
| 1 | `{:.2}` on a non-finite float emits `NaN` / `inf`, which are not valid PDF numbers (7.3.3) | A NaN in `/MediaBox` is a **hard open failure** — pypdf raises `PdfReadError`, pdfminer raises `TypeError`. A NaN in `Tf` selects no font and the page extracts as replacement characters. |
| 2 | `offsets` is sized `objects.len() + 1` and indexed by object *number*; `/Size` is `objects.len() + 1` | Correct today **only** because numbering happens to be dense, 1-based and gapless. Nothing asserts it. Add one object out of sequence and this panics with index-out-of-bounds or writes a wrong `/Size`. |
| 3 | `escape_text` filters `/Title` on `c.is_ascii()` | `"Jonas Hansen — CV"` ships as `"Jonas Hansen  CV"`. The em dash is dropped from the browser tab and from ATS metadata. 7.9.2.2 provides for this: UTF-16BE with a U+FEFF BOM. |
| 4 | No trailer `/ID` | Table 15: *"strongly recommended"*. Tolerated by every viewer; matters to document-management intake. |
| 5 | No `/Lang` on the Catalog | Acrobat's accessibility checker reports "document language not set". One token. |
| 6 | `%%EOF` has no trailing EOL, and zero pages yields `/Kids []` | 7.5.5 wants `%%EOF` on a line of its own. Acrobat refuses a page-less document outright. |

Not acted on, recorded so it is a decision rather than an oversight: `/Widths`
is omitted, which the standard-14 exemption permits in PDF 1.x but which PDF
2.0 deprecates. Every target viewer ships Helvetica metrics. Revisit if the
document ever needs a non-standard face.

- [ ] **Step 1: Write the failing tests**

Add to the `tests` module in `src/pdf.rs`:

```rust
fn one_page_document() -> Vec<u8> {
    let pages = vec![Page {
        placements: vec![Placement {
            x_mm: 16.0,
            y_mm: 279.0,
            size_pt: 12.0,
            font: Font::Helvetica,
            text: "Sælger & (Søn) ApS \\ æøå".into(),
        }],
    }];
    write_pdf("Jonas Hansen - CV", 210.0, 297.0, &pages)
}

#[test]
fn a_minimal_document_is_structurally_sound() {
    let bytes = one_page_document();
    let text = String::from_utf8_lossy(&bytes);

    assert!(bytes.starts_with(b"%PDF-1.4"));
    assert!(bytes.ends_with(b"%%EOF"));
    assert!(text.contains("/Type /Catalog"));
    assert!(text.contains("/BaseFont /Helvetica"));
    assert!(text.contains("/WinAnsiEncoding"));
}

/// The content stream is latin-1 bytes, not UTF-8, so this asserts on raw
/// bytes: \(S<F8>n\) with the parens escaped and ø as a single 0xF8.
#[test]
fn parens_are_escaped_and_high_bytes_are_single_byte() {
    let bytes = one_page_document();
    assert!(
        bytes
            .windows(8)
            .any(|w| w == [b'\\', b'(', b'S', 0xF8, b'n', b'\\', b')', b' ']),
        "expected an escaped, latin-1 encoded (Søn)"
    );
}

/// A viewer locates the objects through this offset. If it is off by even one
/// byte -- which is what happens the moment anything is written between the
/// header and the first object -- the file opens as blank or damaged.
#[test]
fn the_trailer_xref_offset_points_at_the_xref_keyword() {
    let bytes = one_page_document();
    let text = String::from_utf8_lossy(&bytes);
    let start: usize = text
        .rsplit("startxref\n")
        .next()
        .and_then(|tail| tail.split('\n').next())
        .and_then(|line| line.trim().parse().ok())
        .expect("a startxref offset");
    assert_eq!(&bytes[start..start + 4], b"xref");
}

/// /Length must equal exactly the bytes between the newline after `stream` and
/// the newline before `endstream`. Poppler and PDFBox trust it; when it is
/// wrong by the stray newline this writer emits, text extraction truncates.
#[test]
fn the_declared_stream_length_matches_the_actual_stream_bytes() {
    let bytes = one_page_document();
    let text = String::from_utf8_lossy(&bytes);

    let declared: usize = text
        .split("<< /Length ")
        .nth(1)
        .and_then(|tail| tail.split(' ').next())
        .and_then(|n| n.trim().parse().ok())
        .expect("a /Length");

    let begin = find(&bytes, b"stream\n").expect("a stream keyword") + b"stream\n".len();
    let end = find(&bytes, b"\nendstream").expect("an endstream keyword");
    assert_eq!(declared, end - begin, "/Length disagrees with the stream");
}

fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|w| w == needle)
}

#[test]
fn multi_page_documents_reference_every_page() {
    let page = || Page {
        placements: vec![Placement {
            x_mm: 16.0,
            y_mm: 100.0,
            size_pt: 9.0,
            font: Font::HelveticaBold,
            text: "side".into(),
        }],
    };
    let bytes = write_pdf("x", 210.0, 297.0, &[page(), page(), Page::default()]);
    let text = String::from_utf8_lossy(&bytes);
    assert!(text.contains("/Count 3"));
    assert_eq!(text.matches("/Type /Page ").count(), 3);
}

/// The xref /Size must cover every object number in the file. Getting this
/// wrong is invisible in permissive viewers and fatal in strict ones.
#[test]
fn the_trailer_size_covers_every_object() {
    let bytes = write_pdf("x", 210.0, 297.0, &[Page::default(), Page::default()]);
    let text = String::from_utf8_lossy(&bytes);
    let size: usize = text
        .rsplit("/Size ")
        .next()
        .and_then(|tail| tail.split(|c: char| !c.is_ascii_digit()).next())
        .and_then(|n| n.parse().ok())
        .expect("a /Size");
    let highest = text
        .match_indices(" 0 obj")
        .filter_map(|(at, _)| {
            text[..at]
                .rsplit('\n')
                .next()
                .and_then(|line| line.trim().parse::<usize>().ok())
        })
        .max()
        .expect("at least one object");
    assert_eq!(size, highest + 1);
}

/// The site build must be reproducible: the same content/cv.toml has to give
/// the same bytes, or every deploy rsyncs a "changed" PDF that is identical.
#[test]
fn identical_input_produces_identical_bytes() {
    assert_eq!(one_page_document(), one_page_document());
}

/// Acrobat refuses a document with no pages outright. A caller asking for
/// nothing gets one blank page rather than a file that will not open.
#[test]
fn a_document_with_no_pages_still_has_one() {
    let bytes = write_pdf("empty", 210.0, 297.0, &[]);
    let text = String::from_utf8_lossy(&bytes);
    assert!(text.contains("/Count 1"));
    assert_eq!(text.matches("/Type /Page ").count(), 1);
}

/// `{:.2}` on a non-finite f32 emits `NaN` or `inf`, neither of which is a
/// valid PDF number (7.3.3). A NaN reaching /MediaBox is not a cosmetic fault:
/// the document fails to open at all. Cheap insurance against a layout bug
/// that divides by a zero column width.
#[test]
fn non_finite_coordinates_do_not_reach_the_output() {
    let pages = vec![Page {
        placements: vec![Placement {
            x_mm: f32::NAN,
            y_mm: f32::INFINITY,
            size_pt: f32::NAN,
            font: Font::Helvetica,
            text: "hello".into(),
        }],
    }];
    let bytes = write_pdf("x", f32::NAN, 297.0, &pages);
    let text = String::from_utf8_lossy(&bytes);
    for token in ["NaN", "inf", "NAN", "Infinity"] {
        assert!(!text.contains(token), "{token} reached the output");
    }
}

/// 7.9.2.2: a text string is PDFDocEncoded, or UTF-16BE behind a U+FEFF BOM.
/// Filtering to ASCII instead -- which the reference writer did -- silently
/// drops the em dash from "Jonas Hansen — CV" in the browser tab and in the
/// metadata an applicant tracking system reads.
#[test]
fn a_non_ascii_title_survives_as_utf16() {
    let bytes = write_pdf("Jonas Hansen \u{2014} CV", 210.0, 297.0, &[Page::default()]);
    let text = String::from_utf8_lossy(&bytes);
    assert!(text.contains("/Title <FEFF"), "title is not UTF-16BE: {text:.400}");
    // "CV" as UTF-16BE big-endian hex.
    assert!(text.contains("00430056"));
}

#[test]
fn an_ascii_title_stays_a_readable_literal_string() {
    let bytes = write_pdf("Jonas Hansen CV", 210.0, 297.0, &[Page::default()]);
    assert!(String::from_utf8_lossy(&bytes).contains("/Title (Jonas Hansen CV)"));
}

#[test]
fn the_document_declares_its_language_and_an_id() {
    let bytes = one_page_document();
    let text = String::from_utf8_lossy(&bytes);
    assert!(text.contains("/Lang (en)"));
    assert!(text.contains("/ID ["));
}

/// 7.5.5 wants %%EOF on a line of its own.
#[test]
fn the_file_ends_with_a_terminated_eof_marker() {
    assert!(one_page_document().ends_with(b"%%EOF\n"));
}
```

Note the `a_minimal_document_is_structurally_sound` test above asserts
`bytes.ends_with(b"%%EOF")` — update it to `b"%%EOF\n"` to match.

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test pdf::`
Expected: compile errors — `Placement`, `Page`, `write_pdf` not found.

- [ ] **Step 3: Implement the writer**

Port `scratchpad/pdf_sample.rs` into `src/pdf.rs`, with these changes:

1. Drop the sample's own `Font`, `winansi`, and `POINTS_PER_MM` — Task 1 owns those. The content-stream encoder becomes a thin wrapper over `winansi_byte`:

```rust
/// WinAnsi bytes with PDF literal-string escaping. A character with no WinAnsi
/// form degrades to '?' rather than aborting the build; the test over the real
/// content/cv.toml is what stops one reaching a shipped PDF.
fn encode(value: &str) -> Vec<u8> {
    let mut out = Vec::with_capacity(value.len());
    for c in value.chars() {
        let byte = winansi_byte(c).unwrap_or(b'?');
        if matches!(byte, b'(' | b')' | b'\\') {
            out.push(b'\\');
        }
        out.push(byte);
    }
    out
}
```

   Note this fixes a fall-through bug in the sample, where the escape arm pushed the backslash and then pushed `c as u8` from the outer match — correct only because those three characters are ASCII.

2. Set `/Producer (sasin91.xyz)` in the Info dictionary, not the sample's `123platform`.

3. Clamp every float before it is formatted, fixing defect 1:

```rust
/// PDF numbers have no NaN and no infinity (7.3.3). A non-finite value reaching
/// /MediaBox does not degrade the page, it stops the document opening: pypdf
/// raises PdfReadError and pdfminer raises TypeError on the bare `NaN` token.
/// Substituting zero keeps a layout bug survivable and visible.
fn num(value: f32) -> f32 {
    if value.is_finite() { value } else { 0.0 }
}
```

   Apply it to `width`, `height`, and each placement's `x_mm`, `y_mm` and `size_pt`. Format `size_pt` with `{:.2}` as well, so no path uses bare `{}`.

4. Derive the xref size from the objects rather than their count, fixing defect 2:

```rust
// Sized from the highest object number actually present, not from how many
// objects there happen to be. Those agree only while numbering is dense,
// 1-based and gapless -- which nothing enforces, and which the next object
// anyone adds may quietly break.
let size = objects.iter().map(|(number, _)| *number).max().unwrap_or(0) + 1;
let mut offsets = vec![0usize; size];
```

   Then use `size` for the `xref` subsection header and for `/Size` in the trailer.

5. Emit `/Title` as UTF-16BE when it is not pure ASCII, fixing defect 3:

```rust
/// A PDF text string: a readable literal when it is ASCII, UTF-16BE behind a
/// U+FEFF byte order mark when it is not (7.9.2.2). The reference writer
/// filtered non-ASCII away instead, which drops the em dash out of
/// "Jonas Hansen — CV" everywhere the title is displayed.
fn text_string(value: &str) -> String {
    if value.is_ascii() {
        let escaped: String = value
            .chars()
            .filter(|c| !c.is_control())
            .flat_map(|c| {
                match c {
                    '(' | ')' | '\\' => Some('\\'),
                    _ => None,
                }
                .into_iter()
                .chain(std::iter::once(c))
            })
            .collect();
        format!("({escaped})")
    } else {
        let mut hex = String::from("<FEFF");
        for unit in value.encode_utf16() {
            hex.push_str(&format!("{unit:04X}"));
        }
        hex.push('>');
        hex
    }
}
```

   The Info object becomes `format!("<< /Title {} /Producer (sasin91.xyz) >>", text_string(title))`. `escape_text` from the sample is deleted — `text_string` replaces it.

6. Add `/Lang (en)` to the Catalog (defect 5) — the site declares `<html lang="en">` at `templates/base.html:2`, and the CV's content is English:

```rust
(1, b"<< /Type /Catalog /Pages 2 0 R /Lang (en) >>".to_vec()),
```

7. Add a trailer `/ID` (defect 4). It must be deterministic or the reproducibility test fails, so hash the document body rather than using a timestamp:

```rust
/// FNV-1a over the object section. /ID is "strongly recommended" (Table 15) and
/// is what document-management systems use to tell two revisions of a file
/// apart. Derived from the content so the same CV always produces the same ID
/// -- a random or time-based one would make every build a changed file.
fn fnv1a(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}
```

   Compute it over `out` as it stands just before the `xref` keyword is written, and emit `/ID [<{hash:016X}> <{hash:016X}>]` in the trailer.

8. Fix defect 6: substitute a single blank page when `pages` is empty, and terminate the file with `%%EOF\n`:

```rust
// Acrobat rejects a page-less document outright, so an empty request yields
// one blank page rather than a file that will not open.
let blank = [Page { placements: Vec::new() }];
let pages = if pages.is_empty() { &blank[..] } else { pages };
```

Keep the sample's object-numbering scheme, byte-accurate xref construction, and `{:.2}` coordinate formatting — Rust's `core::fmt` has no locale support, so the output stays byte-identical across machines.

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test pdf::`
Expected: PASS.

Then: `cargo fmt --check && cargo clippy -- -D warnings`

- [ ] **Step 5: Commit**

```bash
git add src/pdf.rs
git commit -m "feat(pdf): serialize positioned text into a PDF file"
```

---

### Task 3: Greedy line wrapping

**Files:**
- Create: `src/cv_pdf.rs`
- Modify: `src/main.rs:7-12` (add `mod cv_pdf;`)

**Interfaces:**
- Consumes: `pdf::Font::width`, `pdf::POINTS_PER_MM`.
- Produces: `fn wrap(text: &str, font: Font, size_pt: f32, width_mm: f32) -> Vec<String>` (module-private).

- [ ] **Step 1: Write the failing tests**

Create `src/cv_pdf.rs`:

```rust
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
        assert_eq!(lines.join(" "), LONGEST_ACHIEVEMENT.split_whitespace()
            .collect::<Vec<_>>().join(" "));
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
```

Add `mod cv_pdf;` to `src/main.rs` before `mod djot;`, keeping the list alphabetical.

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test cv_pdf`
Expected: compile error — `wrap` not found.

- [ ] **Step 3: Implement wrapping**

Prepend to `src/cv_pdf.rs`:

```rust
//! Lays a `Cv` out on A4 and hands the placements to `pdf`. The geometry here
//! mirrors the `@page` rule in static/site.css so the printed measure matches
//! what /cv/ prints to, even though the two do not share a layout.

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
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test cv_pdf`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/cv_pdf.rs src/main.rs
git commit -m "feat(cv-pdf): greedy line wrapping against Helvetica metrics"
```

---

### Task 4: The layout cursor and pagination

Where `break-inside: avoid` and `break-after: avoid` from the print stylesheet become arithmetic.

**Files:**
- Modify: `src/cv_pdf.rs`

**Interfaces:**
- Consumes: `wrap` from Task 3.
- Produces (all module-private):
  - `const PAGE_W_MM: f32 = 210.0`, `PAGE_H_MM: f32 = 297.0`, `MARGIN_X_MM: f32 = 16.0`, `MARGIN_Y_MM: f32 = 18.0`, `COLUMN_MM: f32 = 178.0`
  - `struct Line { text: String, font: Font, size_pt: f32, leading_mm: f32, indent_mm: f32 }`
  - `struct Cursor { pages: Vec<Page>, y_mm: f32 }` with:
    - `fn new() -> Cursor`
    - `fn remaining_mm(&self) -> f32`
    - `fn place(&mut self, lines: &[Line])` — emits, breaking pages between lines
    - `fn place_together(&mut self, lines: &[Line])` — starts a new page first if the block will not fit whole
    - `fn gap(&mut self, mm: f32)`
    - `fn finish(self) -> Vec<Page>`

- [ ] **Step 1: Write the failing tests**

Add to the `tests` module in `src/cv_pdf.rs`:

```rust
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
    cursor.place(&(0..filler_count).map(|i| body(&format!("f{i}"))).collect::<Vec<_>>());

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
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test cv_pdf`
Expected: compile errors — `Cursor`, `Line`, the geometry constants not found.

- [ ] **Step 3: Implement the cursor**

Extend the `use` at the top of `src/cv_pdf.rs` to bring in the types this task
introduces:

```rust
use crate::pdf::{Font, POINTS_PER_MM, Page, Placement};
```

Then add:

```rust
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
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test cv_pdf`
Expected: PASS.

Then: `cargo fmt --check && cargo clippy -- -D warnings`

- [ ] **Step 5: Commit**

```bash
git add src/cv_pdf.rs
git commit -m "feat(cv-pdf): page flow with keep-together blocks"
```

---

### Task 5: Render the CV

**Files:**
- Modify: `src/cv_pdf.rs`

**Interfaces:**
- Consumes: `wrap`, `Cursor`, `Line` from Tasks 3 and 4; `Cv`, `Role`, `Education`, `Skill` from `src/cv.rs`.
- Produces: `pub fn render(cv: &Cv) -> Vec<u8>`

- [ ] **Step 1: Write the failing tests**

Add to the `tests` module in `src/cv_pdf.rs`:

```rust
/// The real content, not a fixture. These tests are the replacement for the
/// pdftotext guards the deploy workflow used to run, and they are only worth
/// anything if they check what actually ships.
fn real_cv() -> Cv {
    let src = std::fs::read_to_string("content/cv.toml").expect("content/cv.toml");
    let cv: Cv = toml::from_str(&src).expect("content/cv.toml must parse");
    cv.validate().expect("content/cv.toml must have valid dates");
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
        assert!(text.contains(&role.company), "missing company: {}", role.company);
        assert!(text.contains(&role.title), "missing title: {}", role.title);
        for achievement in &role.achievements {
            let opening: String = achievement.split_whitespace().take(4).collect::<Vec<_>>().join(" ");
            assert!(text.contains(&opening), "missing achievement: {opening}");
        }
    }
    for education in &cv.education {
        assert!(text.contains(&education.school), "missing school: {}", education.school);
    }
    for skill in &cv.skills {
        let opening: String = skill.name.split_whitespace().take(3).collect::<Vec<_>>().join(" ");
        assert!(text.contains(&opening), "missing skill: {opening}");
    }
}

/// A CV is one to two pages. Three is the tolerance; anything beyond it means
/// the layout is broken, not that the content grew.
#[test]
fn the_cv_fits_in_a_sane_number_of_pages() {
    let bytes = render(&real_cv());
    let text = String::from_utf8_lossy(&bytes);
    let pages = text.matches("/Type /Page ").count();
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
    let mut strings: Vec<&str> = vec![&cv.site.name, &cv.site.title, &cv.contact.town,
        &cv.contact.postcode, &cv.contact.phone, &cv.contact.email];
    strings.extend(cv.intro.iter().map(String::as_str));
    for role in &cv.roles {
        strings.extend([role.title.as_str(), role.company.as_str(),
            role.location.as_str(), role.summary.as_str()]);
        strings.extend(role.achievements.iter().map(String::as_str));
    }
    for education in &cv.education {
        strings.extend([education.title.as_str(), education.school.as_str(),
            education.location.as_str()]);
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
```

Note: the `about` field is deliberately not checked — it is prose for `/about/` only and does not appear on the CV.

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test cv_pdf`
Expected: compile error — `render` not found.

- [ ] **Step 3: Implement the render**

Extend the `use` at the top of `src/cv_pdf.rs` a final time:

```rust
use crate::cv::Cv;
use crate::pdf::{Font, POINTS_PER_MM, Page, Placement, write_pdf};
```

Then add the following. The helpers turn the type scale from Global Constraints into `Line`s; `leading_mm` converts points to millimetres so the cursor works in one unit.

```rust
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

/// Render `cv` as a complete PDF.
///
/// Infallible by construction: `Cv::validate` has already run by the time the
/// build reaches here, which is the same guarantee `Role::start_label()` leans
/// on when it unwraps.
///
/// This takes a `&Cv`, not a URL. That is the whole point -- the previous
/// implementation fetched /cv/ over HTTP and shipped the homepage as cv.pdf
/// for several deploys, a failure that is not expressible here.
pub fn render(cv: &Cv) -> Vec<u8> {
    let mut cursor = Cursor::new();

    cursor.place(&lines(&cv.site.name, Font::HelveticaBold, 20.0, 24.0, 0.0, 0.0));
    cursor.place(&lines(&cv.site.title, Font::Helvetica, 11.0, 15.0, 0.0, 0.0));

    let contact = format!(
        "{}, {} \u{b7} {} \u{b7} {}",
        cv.contact.town, cv.contact.postcode, cv.contact.phone, cv.contact.email
    );
    cursor.place(&lines(&contact, Font::Helvetica, 9.0, 13.0, 0.0, 0.0));
    let links = format!("{} \u{b7} {}", cv.site.links.github,
        cv.site.links.linkedin);
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

    let title = format!("{} \u{2014} CV", cv.site.name);
    write_pdf(&title, PAGE_W_MM, PAGE_H_MM, &cursor.finish())
}

/// A section heading. `break-after: avoid` from the print stylesheet, done by
/// reserving room for the heading plus a first line of content: a heading
/// stranded alone at the foot of a page reads as a section with nothing in it.
fn section(cursor: &mut Cursor, title: &str) {
    cursor.gap(pt(10.0));
    let heading = lines(title, Font::HelveticaBold, 11.0, 16.0, 0.0, 0.0);
    let needed: f32 = heading.iter().map(|l| l.leading_mm).sum::<f32>() + pt(14.0) * 2.0;
    if cursor.remaining_mm() < needed {
        cursor.place_together(&heading);
    } else {
        cursor.place(&heading);
    }
    cursor.gap(pt(3.0));
}
```

The `section` helper needs `place_together` to force the break; because the heading alone is short it will always fit on a fresh page, so this is safe.

- [ ] **Step 4: Split `layout` out of `render`, and add the no-orphan-heading test**

The orphan-heading check needs to see `Vec<Page>`, not serialized bytes — digging placements back out of a content stream is a string parse that will break the first time anything about the writer changes. So split the layout from the serialization first. Change the signature of the function written in Step 3 and add a two-line `render` over it:

```rust
/// Lay `cv` out into pages. Separate from `render` so tests can assert on
/// placements rather than on parsed PDF bytes.
fn layout(cv: &Cv) -> Vec<Page> {
    // ... the body written in Step 3, ending with `cursor.finish()` instead of
    // the `write_pdf(...)` call.
}

/// Render `cv` as a complete PDF.
pub fn render(cv: &Cv) -> Vec<u8> {
    let title = format!("{} \u{2014} CV", cv.site.name);
    write_pdf(&title, PAGE_W_MM, PAGE_H_MM, &layout(cv))
}
```

Move the doc comment written in Step 3 (the one explaining why this takes a `&Cv` and not a URL) onto `layout`, since that is where the reasoning belongs.

Then add the test:

```rust
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
```

The page-count test from Step 1 can now also drop its string parse:

```rust
#[test]
fn the_cv_fits_in_a_sane_number_of_pages() {
    let pages = layout(&real_cv()).len();
    assert!((1..=3).contains(&pages), "rendered {pages} pages");
}
```

Replace the earlier version rather than keeping both.

Add one more, which guards a trap this layout currently avoids by accident:

```rust
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
```

- [ ] **Step 5: Run the tests to verify they pass**

Run: `cargo test`
Expected: PASS, including the pre-existing `cv` module tests.

Then: `cargo fmt --check && cargo clippy -- -D warnings`

- [ ] **Step 6: Commit**

```bash
git add src/cv_pdf.rs
git commit -m "feat(cv-pdf): lay out the CV on A4"
```

---

### Task 6: Write the file during the build

**Files:**
- Modify: `src/main.rs` (module list at lines 7-12; the write block around lines 182-185)

**Interfaces:**
- Consumes: `cv_pdf::render`.

- [ ] **Step 1: Write the failing test**

There is no test harness for `main`; this step is a manual assertion instead. Confirm the file does not yet exist:

```powershell
Test-Path public/cv.pdf
```
Expected: `False` (or the directory is absent).

- [ ] **Step 2: Add the write**

In `src/main.rs`, immediately after the existing block that writes the CV page:

```rust
    write(
        format!("{OUT}/cv/index.html"),
        &CvPage { cv: &cv, year }.render()?,
    )?;
```

add:

```rust
    // Generated from the same `Cv` as the page above, so the two cannot carry
    // different content. This used to be a CI step that pointed headless Chrome
    // at a local server; that server once resolved /cv/ to the homepage and the
    // site shipped the landing page as cv.pdf for several deploys. There is no
    // URL to get wrong here.
    fs::write(format!("{OUT}/cv.pdf"), cv_pdf::render(&cv))
        .context("writing public/cv.pdf")?;
```

`write` is the module's string helper, so this uses `fs::write` directly for bytes. Confirm `fs` and `Context` are already imported at `src/main.rs:14-19` — they are.

- [ ] **Step 3: Build and verify**

Run: `cargo run --release`
Expected: the build succeeds and reports its usual `built N posts` line.

```powershell
Test-Path public/cv.pdf
(Get-Item public/cv.pdf).Length
```
Expected: `True`, and a length over 3000 bytes.

- [ ] **Step 4: Confirm the file opens**

Open `public/cv.pdf` in a PDF viewer. It must render as a CV, with text that can be selected and copied. If it opens damaged or blank, stop — that is a Task 2 defect, not a layout one, and it belongs back in the writer's tests.

- [ ] **Step 5: Commit**

```bash
git add src/main.rs
git commit -m "feat: write public/cv.pdf during the build"
```

---

### Task 7: Comparison gate — human review

**This task writes no code. Do not proceed to Task 8 without an explicit go-ahead.**

The spec commits to this: *"The new layout is unproven until seen."*

- [ ] **Step 1: Fetch the current live PDF**

```powershell
Invoke-WebRequest https://sasin91.xyz/cv.pdf -OutFile $env:TEMP\cv-chrome.pdf
```

This is the headless-Chrome output currently in production.

- [ ] **Step 2: Present both**

Send the user `public/cv.pdf` (new) and the fetched `cv-chrome.pdf` (current), and ask which holds up. Name the differences you already know about rather than letting them hunt: the typeface changes from whatever `ubuntu-latest` resolved `system-ui` to, over to Helvetica; the page geometry is unchanged at 18mm/16mm margins; links are plain text rather than clickable annotations.

- [ ] **Step 3: Record the verdict**

If approved, continue to Task 8.

If rejected, the layout is adjustable — the type scale in Global Constraints and the gaps in `render` are the knobs, and no other task depends on their values. If rejected outright, `git checkout main` discards the branch and nothing shipped.

---

### Task 8: Remove the browser from the deploy workflow

**Only after Task 7 is approved.**

**Files:**
- Modify: `.github/workflows/deploy.yml:59-83` (delete), `:129-136` (extend)

- [ ] **Step 1: Delete the render step**

Remove the entire `Render the CV to PDF` step — the step name, its comment block at lines 48-58, and its whole `run:` body. That deletes `apt-get install poppler-utils`, `python3 -m http.server`, the curl warm-up loop, the `grep -q 'Experience'` pre-check, the `google-chrome --headless` invocation, and the four `pdftotext` assertions.

Leave the `Fail if unexpected JavaScript slipped in` step untouched. It guards a different property and its `|| true` fixes are unrelated to this work.

- [ ] **Step 2: Add cv.pdf to the required-files check**

In the `Fail if a required URL is missing` step, the loop tests `public/$u.html`, so `cv.pdf` cannot join that list. Add a line after the loop:

```yaml
          test -f public/cv.pdf || { echo "missing: cv.pdf"; exit 1; }
          echo "all required URLs present"
```

replacing the existing bare `echo "all required URLs present"`.

- [ ] **Step 3: Verify the workflow still parses**

Run: `gh workflow view deploy --repo sasin91/sasin91.xyz` if the repo is on GitHub and `gh` is authenticated. Otherwise inspect the YAML by eye for indentation damage where the step was removed — a deleted step commonly leaves an orphaned `run: |` body.

- [ ] **Step 4: Confirm the full check suite passes locally**

Run: `cargo fmt --check && cargo clippy -- -D warnings && cargo test`
Expected: all PASS. This is exactly what the `check` job runs.

- [ ] **Step 5: Commit**

```bash
git add .github/workflows/deploy.yml
git commit -m "ci: drop headless Chrome, the web server and poppler from deploy"
```

- [ ] **Step 6: Merge**

Use `superpowers:finishing-a-development-branch` to decide how this lands on `main`. The first push to `main` runs the real deploy; the `Verify the deployed site` step already curls `/cv.pdf` and will fail the run if the file is missing or non-200.

---

## Notes for the implementer

**Why the tests are where they are.** The deploy workflow used to prove things about `cv.pdf` by installing `poppler-utils` and grepping `pdftotext` output. Those assertions moved into Task 5 unchanged in intent: the same properties, checked at `cargo test` time, on the real `content/cv.toml`. If you find yourself weakening one of them to make it pass, you are re-introducing the bug it was written for.

**The one thing that cannot be unit-tested** is whether the page looks right. That is Task 7, and it is a human's call.
