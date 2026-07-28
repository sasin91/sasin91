//! HTML escaping, shared by the Djot and math renderers so the two cannot
//! drift apart.

/// Escape text for insertion into HTML, including inside a double-quoted
/// attribute value.
///
/// Ampersand is replaced first, so the entities produced by the later
/// replacements are not themselves re-escaped.
pub fn escape(raw: &str) -> String {
    raw.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes_the_four_dangerous_characters() {
        assert_eq!(
            escape(r#"<a href="x">&</a>"#),
            "&lt;a href=&quot;x&quot;&gt;&amp;&lt;/a&gt;"
        );
    }

    #[test]
    fn escapes_ampersands_before_the_entities_it_creates() {
        // A naive implementation that replaces < before & yields "&amp;lt;".
        assert_eq!(escape("&lt;"), "&amp;lt;");
    }

    #[test]
    fn leaves_ordinary_text_alone() {
        assert_eq!(escape("4-remaster.sh"), "4-remaster.sh");
    }
}
