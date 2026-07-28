//! LaTeX to MathML at build time. Browsers render MathML natively, so a
//! formula costs the reader no JavaScript.

use latex2mathml::{DisplayStyle, latex_to_mathml};

use crate::html::escape;

pub fn to_mathml(tex: &str, display: bool) -> String {
    let style = if display {
        DisplayStyle::Block
    } else {
        DisplayStyle::Inline
    };

    match latex_to_mathml(tex, style) {
        Ok(mathml) => mathml,
        // A broken formula should be visible in the page, not silently dropped.
        Err(error) => format!(
            "<code class=\"math-error\" title=\"{}\">{}</code>",
            escape(&error.to_string()),
            escape(tex)
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_inline_math_to_mathml() {
        let html = to_mathml(r"e_1 = w\left(1 + \frac{r}{30}\right)", false);
        assert!(html.contains("<math"));
        assert!(html.contains("display=\"inline\""));
    }

    #[test]
    fn renders_display_math_as_a_block() {
        let html = to_mathml(r"b_1 = \frac{w}{1.0278 - 0.0278 r}", true);
        assert!(html.contains("<math"));
        assert!(html.contains("display=\"block\""));
    }

    #[test]
    fn renders_broken_math_visibly_instead_of_dropping_it() {
        let html = to_mathml(r"\frac{", false);
        assert!(html.contains("math-error"), "got: {html}");
        // The offending source must still reach the page.
        assert!(html.contains("frac"), "got: {html}");
    }

    #[test]
    fn escapes_markup_in_the_error_path() {
        let html = to_mathml(r"\frac{<script>", false);
        // The error branch must actually be taken, or this test is vacuous.
        assert!(html.contains("math-error"), "got: {html}");
        assert!(!html.contains("<script>"), "got: {html}");
    }
}
