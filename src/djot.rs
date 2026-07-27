//! Djot to HTML.
//!
//! Djot gives natively what CommonMark needed hand-rolled parsing for:
//! `::: warning` divs, `{key="value"}` attributes on any block, and math.
//! So this module intercepts only the two things that need a build step:
//!
//!   * code blocks -> syntax highlighting, plus a filename bar from `title=`
//!   * math        -> MathML, rendered here rather than by KaTeX in the browser

use anyhow::Result;
use jotdown::{Container, Event, Parser, Render};

use crate::highlight::Highlighter;
use crate::html::escape;
use crate::math;

pub fn render(source: &str, hl: &Highlighter) -> Result<String> {
    let mut events: Vec<Event> = Vec::new();

    // What we are currently accumulating text into, if anything.
    let mut code: Option<(String, Option<String>)> = None;
    let mut display_math: Option<bool> = None;
    let mut buffer = String::new();

    for event in Parser::new(source) {
        match event {
            Event::Start(Container::CodeBlock { language }, attrs) => {
                let title = attrs
                    .get_value("title")
                    .map(|v| v.to_string().trim_matches('"').to_string());
                code = Some((language.to_string(), title));
                buffer.clear();
            }
            Event::End(Container::CodeBlock { .. }) => {
                let (language, title) = code.take().unwrap_or_default();
                events.extend(raw_block(code_html(&language, title.as_deref(), &buffer, hl)?));
                buffer.clear();
            }

            Event::Start(Container::Math { display }, _) => {
                display_math = Some(display);
                buffer.clear();
            }
            Event::End(Container::Math { .. }) => {
                let display = display_math.take().unwrap_or(false);
                events.extend(raw_inline(math::to_mathml(&buffer, display)));
                buffer.clear();
            }

            // Text belonging to a block we are capturing, rather than prose.
            Event::Str(text) if code.is_some() || display_math.is_some() => {
                buffer.push_str(&text)
            }

            other => events.push(other),
        }
    }

    let mut html = String::new();
    jotdown::html::Renderer::default().push_events(events.into_iter(), &mut html)?;
    Ok(html)
}

/// Splice pre-rendered HTML back into the event stream.
fn raw_block(html: String) -> [Event<'static>; 3] {
    [
        Event::Start(
            Container::RawBlock { format: "html".into() },
            Default::default(),
        ),
        Event::Str(html.into()),
        Event::End(Container::RawBlock { format: "html".into() }),
    ]
}

fn raw_inline(html: String) -> [Event<'static>; 3] {
    [
        Event::Start(
            Container::RawInline { format: "html".into() },
            Default::default(),
        ),
        Event::Str(html.into()),
        Event::End(Container::RawInline { format: "html".into() }),
    ]
}

fn code_html(language: &str, title: Option<&str>, code: &str, hl: &Highlighter) -> Result<String> {
    let highlighted = hl.to_html(code, language)?;

    Ok(match title {
        Some(name) => format!(
            "<div class=\"codeblock\"><div class=\"filename\">{}</div>{highlighted}</div>",
            escape(name)
        ),
        None => highlighted,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::highlight::Highlighter;

    fn render_str(source: &str) -> String {
        render(source, &Highlighter::new()).unwrap()
    }

    #[test]
    fn renders_a_div_as_a_callout() {
        let html = render_str("::: warning\n## What went wrong\n\nIt broke.\n:::\n");
        assert!(html.contains("<div class=\"warning\">"), "got: {html}");
        assert!(html.contains("What went wrong"));
    }

    #[test]
    fn frames_a_code_block_that_has_a_title() {
        let html = render_str("{title=\"4-remaster.sh\"}\n```bash\necho hi\n```\n");
        assert!(html.contains("<div class=\"codeblock\">"), "got: {html}");
        assert!(html.contains("<div class=\"filename\">4-remaster.sh</div>"));
        assert!(html.contains("hl-code"));
    }

    #[test]
    fn leaves_an_untitled_code_block_unframed() {
        let html = render_str("```bash\necho hi\n```\n");
        assert!(!html.contains("codeblock"), "got: {html}");
        assert!(html.contains("hl-code"));
    }

    #[test]
    fn renders_math_to_mathml_rather_than_deferring_to_javascript() {
        let html = render_str("Epley is $`e = w(1 + r/30)`.\n");
        assert!(html.contains("<math"), "got: {html}");
        // jotdown's default would emit \( ... \) for KaTeX to pick up.
        assert!(!html.contains(r"\("), "must not defer to KaTeX: {html}");
    }

    #[test]
    fn passes_raw_html_through() {
        let html = render_str("``` =html\n<p class=\"colophon\">Measured.</p>\n```\n");
        assert!(html.contains("<p class=\"colophon\">Measured.</p>"), "got: {html}");
    }

    #[test]
    fn escapes_a_title_that_contains_markup() {
        let html = render_str("{title=\"<script>\"}\n```bash\nx\n```\n");
        assert!(!html.contains("<div class=\"filename\"><script>"), "got: {html}");
    }
}
