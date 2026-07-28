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

            // A local `.svg` image: capture its alt text (the `Str` events
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
                events.extend(raw_block(format!(
                    "<figure class=\"diagram\" role=\"img\" aria-label=\"{}\">{}</figure>",
                    escape(&buffer),
                    svg
                )));
                buffer.clear();
            }

            // Text belonging to a block we are capturing, rather than prose.
            Event::Str(text) if code.is_some() || display_math.is_some() || inlining_svg => {
                buffer.push_str(&text);
            }

            other => events.push(other),
        }
    }

    let mut html = String::new();
    jotdown::html::Renderer::default().push_events(events.into_iter(), &mut html)?;
    Ok(html)
}

/// True for images that should be inlined: a root-relative path ending in
/// `.svg`. Remote URLs and non-SVG images are left to jotdown's normal
/// `<img>` rendering.
fn is_local_svg(dst: &str) -> bool {
    dst.starts_with('/') && dst.ends_with(".svg")
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
}
