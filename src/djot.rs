//! Djot to HTML.
//!
//! Djot gives natively what CommonMark needed hand-rolled parsing for:
//! `::: warning` divs, `{key="value"}` attributes on any block, and math.
//! So this module intercepts only the two things that need a build step:
//!
//!   * code blocks -> syntax highlighting, plus a filename bar from `title=`
//!   * math        -> MathML, rendered here rather than by KaTeX in the browser

use std::path::Path;

use anyhow::{Context, Result};
use jotdown::{Container, Event, Parser, Render};

use crate::highlight::Highlighter;
use crate::html::escape;
use crate::math;

/// Render with the default asset root (`static`), relative to the process's
/// working directory, matching where the rest of the build reads assets from.
pub fn render(source: &str, hl: &Highlighter) -> Result<String> {
    render_with_assets(source, hl, Path::new("static"))
}

/// Render, inlining local `.svg` images found under `assets` so they become
/// part of the page document and inherit its `data-theme`, rather than
/// staying a separate `<img>` document that can only see `prefers-color-scheme`.
pub fn render_with_assets(source: &str, hl: &Highlighter, assets: &Path) -> Result<String> {
    let mut events: Vec<Event> = Vec::new();

    // What we are currently accumulating text into, if anything.
    let mut code: Option<(String, Option<String>)> = None;
    let mut display_math: Option<bool> = None;
    let mut inlining_svg = false;
    let mut buffer = String::new();

    for event in Parser::new(source) {
        match event {
            Event::Start(Container::CodeBlock { language }, attrs) => {
                let title = attrs.get_value("title").map(|v| v.to_string());
                code = Some((language.to_string(), title));
                buffer.clear();
            }
            Event::End(Container::CodeBlock { .. }) => {
                let (language, title) = code.take().unwrap_or_default();
                events.extend(raw_block(code_html(
                    &language,
                    title.as_deref(),
                    &buffer,
                    hl,
                )?));
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

            // A local `.svg` image: capture its alt text (every event
            // between here and the matching `End`) instead of emitting the
            // usual `<img>` Start/End pair.
            Event::Start(Container::Image(ref dst, _), _) if is_local_svg(dst) => {
                inlining_svg = true;
                buffer.clear();
            }
            Event::End(Container::Image(dst, _)) if inlining_svg => {
                inlining_svg = false;
                let path = assets.join(dst.trim_start_matches('/'));
                let svg = std::fs::read_to_string(&path)
                    .with_context(|| format!("missing SVG referenced by post: {dst}"))?;
                events.extend(raw_block(svg_figure(&buffer, &svg)));
                buffer.clear();
            }

            // Everything else while we are inside an image being inlined
            // belongs to its alt text and must be captured or discarded here
            // -- never emitted -- or smart-typography atoms (quotes, dashes,
            // ellipsis, ...), which jotdown represents as their own event
            // variants rather than `Str`, leak as loose characters into the
            // page and vanish from the accessible name.
            event if inlining_svg => push_alt_text(&mut buffer, event),

            // Text belonging to a block we are capturing, rather than prose.
            Event::Str(text) if code.is_some() || display_math.is_some() => {
                buffer.push_str(&text);
            }

            other => events.push(other),
        }
    }

    let mut html = String::new();
    jotdown::html::Renderer::default().push_events(events.into_iter(), &mut html)?;
    Ok(html)
}

/// The heading a passage sits under: the anchor jotdown gives its section
/// (the same `id` as in the rendered HTML) and the heading's text.
#[derive(Debug, Clone, PartialEq)]
pub struct Section {
    pub id: String,
    pub heading: String,
}

/// One searchable passage of a document.
#[derive(Debug, Clone, PartialEq)]
pub struct Passage {
    pub text: String,
    /// The innermost heading above it, if any, so a search result can link
    /// to that part of the page rather than its top.
    pub section: Option<Section>,
}

/// The prose of a document as plain-text passages, one per top-level block
/// (a paragraph, a whole list, a block quote), for embedding and searching.
///
/// Code blocks, raw HTML and math are left out: they are what a reader
/// recognises on the page, not what a question is phrased in. A paragraph
/// ending in a colon introduces whatever follows it, so it is joined to the
/// next passage in the same section rather than embedded on its own.
pub fn passages(source: &str) -> Vec<Passage> {
    let mut passages: Vec<Passage> = Vec::new();
    let mut current = String::new();
    let mut sections: Vec<Section> = Vec::new();
    let mut in_heading = false;
    let mut depth = 0usize;
    let mut skipping = 0usize;

    let flush = |text: &mut String, section: Option<&Section>, passages: &mut Vec<Passage>| {
        let text = std::mem::take(text);
        let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
        if text.is_empty() {
            return;
        }
        let section = section.cloned();
        match passages.last_mut() {
            Some(previous) if previous.text.ends_with(':') && previous.section == section => {
                previous.text.push(' ');
                previous.text.push_str(&text);
            }
            _ => passages.push(Passage { text, section }),
        }
    };

    for event in Parser::new(source) {
        match event {
            Event::Start(Container::Section { id }, _) => sections.push(Section {
                id: id.to_string(),
                heading: String::new(),
            }),
            Event::End(Container::Section { .. }) => {
                sections.pop();
            }
            // Headings label the passages after them rather than being
            // passages of their own.
            Event::Start(Container::Heading { .. }, _) => in_heading = true,
            Event::End(Container::Heading { .. }) => in_heading = false,
            event if in_heading => {
                if let Some(section) = sections.last_mut() {
                    push_alt_text(&mut section.heading, event);
                }
            }

            // Wrappers with no prose of their own: their children are the
            // top-level blocks.
            Event::Start(Container::Document | Container::Div { .. }, _)
            | Event::End(Container::Document | Container::Div { .. }) => {}

            Event::Start(container, _) => {
                if is_unsearchable(&container) {
                    skipping += 1;
                }
                if container.is_block() {
                    depth += 1;
                }
            }
            Event::End(container) => {
                if is_unsearchable(&container) {
                    skipping -= 1;
                }
                if container.is_block() {
                    depth -= 1;
                    if depth == 0 {
                        flush(&mut current, sections.last(), &mut passages);
                    } else {
                        current.push(' ');
                    }
                }
            }

            event if skipping == 0 && depth > 0 => push_alt_text(&mut current, event),
            _ => {}
        }
    }

    passages
}

fn is_unsearchable(container: &Container) -> bool {
    matches!(
        container,
        Container::CodeBlock { .. }
            | Container::RawBlock { .. }
            | Container::RawInline { .. }
            | Container::Math { .. }
            | Container::LinkDefinition { .. }
            | Container::Footnote { .. }
    )
}

/// Whether a document contains any heading. Knowledge nodes may not: the
/// node's own title is its heading, and jotdown would give an inner heading
/// a generated id that could collide with a node's anchor.
pub fn has_heading(source: &str) -> bool {
    Parser::new(source).any(|event| matches!(event, Event::Start(Container::Heading { .. }, _)))
}

/// True for images that should be inlined: a root-relative path ending in
/// `.svg`. Remote URLs and non-SVG images are left to jotdown's normal
/// `<img>` rendering. `pub(crate)` so `main.rs` can apply the same rule to a
/// post's hero, which bypasses this module entirely (it comes straight from
/// frontmatter, never through `Parser`).
pub(crate) fn is_local_svg(dst: &str) -> bool {
    dst.starts_with('/') && dst.ends_with(".svg")
}

/// Wrap raw SVG markup in the `<figure>` every inlined diagram uses, carrying
/// `alt` as the accessible name (a `role="img"` on the figure plus an
/// `aria-label`, since the SVG's own internal text is not a substitute -- a
/// screen reader should not have to read a diagram's axis labels to learn
/// what it depicts). `pub(crate)` so `main.rs` can build the same markup for
/// a post's hero image, which never passes through `render_with_assets` --
/// it comes straight from frontmatter to an `<img>` in the template.
pub(crate) fn svg_figure(alt: &str, svg: &str) -> String {
    format!(
        "<figure class=\"diagram\" role=\"img\" aria-label=\"{}\">{}</figure>",
        escape(alt),
        svg
    )
}

/// Flatten one event from inside an inlined image's alt text into plain
/// text. jotdown represents smart-typography output (curly quotes, dashes,
/// ellipsis, non-breaking space) as their own zero-payload event variants
/// rather than `Event::Str`, so they need mapping to the literal characters
/// jotdown's own HTML renderer would produce for them (verified against the
/// examples in jotdown's doc comments). Nested containers (emphasis, spans,
/// ...) contribute no text of their own -- their contents arrive as
/// separate `Str` events -- so their `Start`/`End` are dropped here, along
/// with anything else with no sensible flat-text meaning. Every variant is
/// matched explicitly, plus a defensive wildcard, so a future jotdown
/// variant this code does not know about is dropped rather than leaked into
/// the page as an unescaped `other` event.
fn push_alt_text(buffer: &mut String, event: Event) {
    match event {
        Event::Str(text) => buffer.push_str(&text),
        Event::Symbol(name) => {
            buffer.push(':');
            buffer.push_str(&name);
            buffer.push(':');
        }
        Event::LeftSingleQuote => buffer.push('\u{2018}'),
        Event::RightSingleQuote => buffer.push('\u{2019}'),
        Event::LeftDoubleQuote => buffer.push('\u{201c}'),
        Event::RightDoubleQuote => buffer.push('\u{201d}'),
        Event::Ellipsis => buffer.push('\u{2026}'),
        Event::EnDash => buffer.push('\u{2013}'),
        Event::EmDash => buffer.push('\u{2014}'),
        Event::NonBreakingSpace => buffer.push('\u{00a0}'),
        Event::Softbreak | Event::Hardbreak => buffer.push(' '),
        // Zero-width, structural, or otherwise contributing no flat text.
        Event::Escape
        | Event::Start(..)
        | Event::End(..)
        | Event::FootnoteReference(_)
        | Event::Blankline
        | Event::ThematicBreak(_)
        | Event::Attributes(_) => {}
        // Defensive: a jotdown variant added later that this match has not
        // been taught about must be dropped, not leaked into the page.
        #[allow(unreachable_patterns)]
        _ => {}
    }
}

/// Splice pre-rendered HTML back into the event stream.
fn raw_block(html: String) -> [Event<'static>; 3] {
    [
        Event::Start(
            Container::RawBlock {
                format: "html".into(),
            },
            Default::default(),
        ),
        Event::Str(html.into()),
        Event::End(Container::RawBlock {
            format: "html".into(),
        }),
    ]
}

fn raw_inline(html: String) -> [Event<'static>; 3] {
    [
        Event::Start(
            Container::RawInline {
                format: "html".into(),
            },
            Default::default(),
        ),
        Event::Str(html.into()),
        Event::End(Container::RawInline {
            format: "html".into(),
        }),
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
        assert!(
            html.contains("<p class=\"colophon\">Measured.</p>"),
            "got: {html}"
        );
    }

    #[test]
    fn escapes_a_title_that_contains_markup() {
        let html = render_str("{title=\"<script>\"}\n```bash\nx\n```\n");
        assert!(
            !html.contains("<div class=\"filename\"><script>"),
            "got: {html}"
        );
    }

    #[test]
    fn keeps_escaped_quotes_in_a_title_intact() {
        let html = render_str("{title=\"\\\"weird\\\".sh\"}\n```bash\nx\n```\n");
        assert!(
            html.contains("<div class=\"filename\">&quot;weird&quot;.sh</div>"),
            "got: {html}"
        );
    }

    #[test]
    fn renders_display_math_to_mathml_rather_than_deferring_to_javascript() {
        let html = render_str("$$`e = w(1 + r/30)`\n");
        assert!(
            html.contains("<math") && html.contains(r#"display="block""#),
            "got: {html}"
        );
        // jotdown's default would emit \[ ... \] for KaTeX to pick up.
        assert!(
            !html.contains(r"\(") && !html.contains(r"\["),
            "must not defer to KaTeX: {html}"
        );
    }

    #[test]
    fn inlines_a_local_svg_so_it_can_inherit_the_page_theme() {
        let dir = std::env::temp_dir().join(format!("djot-svg-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("images")).unwrap();
        std::fs::write(
            dir.join("images/diagram.svg"),
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 10 10"><rect width="10" height="10"/></svg>"#,
        )
        .unwrap();

        let html = render_with_assets(
            "![A diagram](/images/diagram.svg)\n",
            &Highlighter::new(),
            &dir,
        )
        .unwrap();

        assert!(html.contains("<svg"), "should be inlined: {html}");
        assert!(!html.contains("<img"), "should not remain an img: {html}");
        assert!(html.contains(r#"role="img""#), "needs a role: {html}");
        assert!(
            html.contains("A diagram"),
            "alt must survive as a label: {html}"
        );

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn leaves_remote_and_raster_images_alone() {
        let dir = std::env::temp_dir().join(format!("djot-svg-skip-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();

        let html = render_with_assets(
            "![remote](https://example.com/x.svg)\n![raster](/images/y.png)\n",
            &Highlighter::new(),
            &dir,
        )
        .unwrap();

        assert_eq!(html.matches("<img").count(), 2, "both stay images: {html}");

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn a_missing_svg_fails_the_build_rather_than_rendering_nothing() {
        let dir = std::env::temp_dir().join(format!("djot-svg-missing-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();

        let result = render_with_assets("![gone](/images/nope.svg)\n", &Highlighter::new(), &dir);
        assert!(result.is_err(), "a broken diagram must not ship silently");

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn passages_are_top_level_blocks_as_plain_text() {
        let source = "First *paragraph* with [a link](#x) and `code`.\n\n\
                      Second paragraph,\nwrapped.\n\n\
                      > A quote.\n";
        assert_eq!(
            texts(source),
            [
                "First paragraph with a link and code.",
                "Second paragraph, wrapped.",
                "A quote."
            ]
        );
    }

    #[test]
    fn passages_leave_out_code_and_math() {
        let source = "Before $`x^2` after.\n\n```rust\nfn main() {}\n```\n\n$$`e = mc^2`\n";
        assert_eq!(texts(source), ["Before after."]);
    }

    #[test]
    fn a_lead_in_ending_in_a_colon_joins_the_list_it_introduces() {
        let source = "Three things:\n\n- one\n- two\n- three\n\nAfter.\n";
        assert_eq!(texts(source), ["Three things: one two three", "After."]);
    }

    #[test]
    fn passages_look_inside_divs() {
        assert_eq!(texts("::: note\nInside.\n:::\n"), ["Inside."]);
    }

    fn texts(source: &str) -> Vec<String> {
        passages(source).into_iter().map(|p| p.text).collect()
    }

    /// The section id must be the one jotdown puts on the rendered
    /// `<section>`, or a search result would link to an anchor that does not
    /// exist on the page.
    #[test]
    fn passages_know_the_heading_they_sit_under() {
        let source = "Intro.\n\n## What it costs\n\nMoney.\n\nMore money:\n\n- lots\n\n## Where *k3s* wins\n\nHere.\n";
        let found = passages(source);
        assert_eq!(found[0].section, None);
        assert_eq!(found[0].text, "Intro.");

        let costs = found[1].section.clone().unwrap();
        assert_eq!(costs.heading, "What it costs");
        assert_eq!(found[2].section, found[1].section);
        assert_eq!(found[2].text, "More money: lots");

        let wins = found[3].section.clone().unwrap();
        assert_eq!(wins.heading, "Where k3s wins");

        let html = render_str(source);
        for section in [costs, wins] {
            assert!(
                html.contains(&format!("<section id=\"{}\">", section.id)),
                "{} not in {html}",
                section.id
            );
        }
        assert_eq!(found.len(), 4, "headings are labels, not passages");
    }

    #[test]
    fn passages_keep_smart_punctuation_as_text() {
        assert_eq!(
            texts("It's \"quoted\" -- and...\n"),
            ["It\u{2019}s \u{201c}quoted\u{201d} \u{2013} and\u{2026}"]
        );
    }

    #[test]
    fn detects_headings() {
        assert!(has_heading("Text.\n\n## Heading\n"));
        assert!(!has_heading("Text with a # hash.\n"));
    }

    fn write_test_svg(dir: &std::path::Path) {
        std::fs::create_dir_all(dir.join("images")).unwrap();
        std::fs::write(
            dir.join("images/diagram.svg"),
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 10 10"><rect width="10" height="10"/></svg>"#,
        )
        .unwrap();
    }

    #[test]
    fn preserves_an_apostrophe_in_alt_text_without_leaking_a_stray_quote() {
        let dir = std::env::temp_dir().join(format!("djot-svg-apos-{}", std::process::id()));
        write_test_svg(&dir);

        let html = render_with_assets(
            "![the deploy's timeline](/images/diagram.svg)\n",
            &Highlighter::new(),
            &dir,
        )
        .unwrap();

        let expected_label = "aria-label=\"the deploy\u{2019}s timeline\"";
        assert!(
            html.contains(expected_label),
            "apostrophe must survive as part of the accessible name: {html}"
        );

        let outside_label = html.replacen(expected_label, "", 1);
        assert!(
            !outside_label.contains('\u{2019}'),
            "no stray quote glyph should leak outside the aria-label: {html}"
        );

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn preserves_double_quotes_in_alt_text_without_leaking_stray_quotes() {
        let dir = std::env::temp_dir().join(format!("djot-svg-quotes-{}", std::process::id()));
        write_test_svg(&dir);

        let html = render_with_assets(
            "![just \"quoted\" text](/images/diagram.svg)\n",
            &Highlighter::new(),
            &dir,
        )
        .unwrap();

        let expected_label = "aria-label=\"just \u{201c}quoted\u{201d} text\"";
        assert!(
            html.contains(expected_label),
            "smart double quotes must survive as part of the accessible name: {html}"
        );

        let outside_label = html.replacen(expected_label, "", 1);
        assert!(
            !outside_label.contains('\u{201c}') && !outside_label.contains('\u{201d}'),
            "no stray quote glyphs should leak outside the aria-label: {html}"
        );

        std::fs::remove_dir_all(&dir).ok();
    }
}
