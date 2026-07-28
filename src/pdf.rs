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
}
