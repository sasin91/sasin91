//! A minimal PDF writer: absolute-positioned text in the two Helvetica faces,
//! serialized to bytes. It knows nothing about what it is typesetting -- see
//! `cv_pdf` for the layout that drives it.

// Nothing in `main.rs` calls this yet: Task 2 consumes `Font::width` and
// `winansi_byte` for layout/wrapping, and Task 5 consumes them for byte
// serialization. Until then `-D warnings` would fail the build on dead code
// that later tasks give a caller, so the allow stays until Task 5 lands.
#![allow(dead_code)]

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

/// PDF numbers have no NaN and no infinity (7.3.3). A non-finite value reaching
/// /MediaBox does not degrade the page, it stops the document opening: pypdf
/// raises PdfReadError and pdfminer raises TypeError on the bare `NaN` token.
/// Substituting zero keeps a layout bug survivable and visible.
fn num(value: f32) -> f32 {
    if value.is_finite() { value } else { 0.0 }
}

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

/// One absolutely-positioned run of text on a page.
pub struct Placement {
    pub x_mm: f32,
    pub y_mm: f32,
    pub size_pt: f32,
    pub font: Font,
    pub text: String,
}

/// One page of absolute-positioned text placements.
#[derive(Default)]
pub struct Page {
    pub placements: Vec<Placement>,
}

/// Serialize `pages` into a complete PDF file. Structural correctness only --
/// callers decide layout, this only decides how to make that layout a valid
/// PDF 1.4 byte stream.
pub fn write_pdf(title: &str, width_mm: f32, height_mm: f32, pages: &[Page]) -> Vec<u8> {
    // Acrobat rejects a page-less document outright, so an empty request
    // yields one blank page rather than a file that will not open.
    let blank = [Page {
        placements: Vec::new(),
    }];
    let pages = if pages.is_empty() { &blank[..] } else { pages };

    let width = num(width_mm) * POINTS_PER_MM;
    let height = num(height_mm) * POINTS_PER_MM;

    // object numbering: 1 Catalog, 2 Pages, 3 Info, 4 F1, 5 F2, then Page+Contents
    // pairs (first page object = 6)
    let first_page_object = 6usize;
    let page_object = |index: usize| first_page_object + index * 2;

    let kids: Vec<String> = (0..pages.len())
        .map(|index| format!("{} 0 R", page_object(index)))
        .collect();

    let mut objects: Vec<(usize, Vec<u8>)> = vec![
        (1, b"<< /Type /Catalog /Pages 2 0 R /Lang (en) >>".to_vec()),
        (
            2,
            format!(
                "<< /Type /Pages /Kids [{}] /Count {} >>",
                kids.join(" "),
                pages.len()
            )
            .into_bytes(),
        ),
        (
            3,
            format!(
                "<< /Title {} /Producer (sasin91.xyz) >>",
                text_string(title)
            )
            .into_bytes(),
        ),
        (
            4,
            b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>"
                .to_vec(),
        ),
        (
            5,
            b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica-Bold /Encoding /WinAnsiEncoding >>"
                .to_vec(),
        ),
    ];

    for (index, page) in pages.iter().enumerate() {
        let mut content = Vec::new();
        for placement in &page.placements {
            let font = match placement.font {
                Font::Helvetica => "/F1",
                Font::HelveticaBold => "/F2",
            };
            content.extend_from_slice(
                format!(
                    "BT {font} {size:.2} Tf 1 0 0 1 {x:.2} {y:.2} Tm (",
                    size = num(placement.size_pt),
                    x = num(placement.x_mm) * POINTS_PER_MM,
                    y = num(placement.y_mm) * POINTS_PER_MM,
                )
                .as_bytes(),
            );
            content.extend_from_slice(&encode(&placement.text));
            content.extend_from_slice(b") Tj ET\n");
        }

        let contents_object = page_object(index) + 1;
        objects.push((
            page_object(index),
            format!(
                "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {width:.2} {height:.2}] \
                 /Resources << /Font << /F1 4 0 R /F2 5 0 R >> >> \
                 /Contents {contents_object} 0 R >>"
            )
            .into_bytes(),
        ));
        let mut stream = format!("<< /Length {} >>\nstream\n", content.len()).into_bytes();
        stream.extend_from_slice(&content);
        stream.extend_from_slice(b"\nendstream");
        objects.push((contents_object, stream));
    }

    // assemble with byte-accurate xref offsets
    let mut out = b"%PDF-1.4\n%\xE6\xF8\xE5\xB5\n".to_vec();
    // Sized from the highest object number actually present, not from how many
    // objects there happen to be. Those agree only while numbering is dense,
    // 1-based and gapless -- which nothing enforces, and which the next object
    // anyone adds may quietly break.
    let size = objects.iter().map(|(number, _)| *number).max().unwrap_or(0) + 1;
    let mut offsets = vec![0usize; size];
    for (number, body) in &objects {
        offsets[*number] = out.len();
        out.extend_from_slice(format!("{number} 0 obj\n").as_bytes());
        out.extend_from_slice(body);
        out.extend_from_slice(b"\nendobj\n");
    }
    let xref_start = out.len();
    out.extend_from_slice(format!("xref\n0 {size}\n").as_bytes());
    out.extend_from_slice(b"0000000000 65535 f \n");
    for offset in offsets.iter().skip(1) {
        out.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
    }
    // Computed over the object section as it stands right now -- before the
    // xref keyword is written -- so the ID stays a pure function of the
    // document body and `render` produces byte-identical output across runs.
    let id = fnv1a(&out);
    out.extend_from_slice(
        format!(
            "trailer\n<< /Size {size} /Root 1 0 R /Info 3 0 R /ID [<{id:016X}> <{id:016X}>] >>\nstartxref\n{xref_start}\n%%EOF\n",
        )
        .as_bytes(),
    );
    out
}

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
                crate::pdf_metrics::HELVETICA[index],
                0,
                "Helvetica has no width for 0x{code:02X}"
            );
            assert_ne!(
                crate::pdf_metrics::HELVETICA_BOLD[index],
                0,
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
        assert!(bytes.ends_with(b"%%EOF\n"));
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
        assert!(
            text.contains("/Title <FEFF"),
            "title is not UTF-16BE: {text:.400}"
        );
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
}
