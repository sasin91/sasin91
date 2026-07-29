//! Loading posts from disk: frontmatter, the `Post` type, and the walk.

use anyhow::{Context, Result};
use chrono::NaiveDate;
use serde::Deserialize;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

/// TOML has a native date type, so `date = 2026-07-26` arrives as a structured
/// value rather than a string. Accept it and hand back a chrono date.
fn toml_date<'de, D>(de: D) -> Result<NaiveDate, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error as _;

    let raw = toml::value::Datetime::deserialize(de)?;
    let date = raw
        .date
        .ok_or_else(|| D::Error::custom("expected a date"))?;

    NaiveDate::from_ymd_opt(date.year as i32, date.month as u32, date.day as u32)
        .ok_or_else(|| D::Error::custom(format!("not a real date: {raw}")))
}

#[derive(Debug, Deserialize)]
pub struct FrontMatter {
    /// The URL path, without leading or trailing slash, e.g.
    /// `blog/trongate/mx-transition`. Deliberately explicit rather than
    /// derived from the filename, so nested URLs survive.
    pub path: String,
    pub title: String,
    #[serde(deserialize_with = "toml_date")]
    pub date: NaiveDate,
    pub description: String,
    #[serde(default)]
    pub hero: Option<String>,
    #[serde(default)]
    pub hero_alt: Option<String>,
    /// A raster image used only as the link-share card (`og:image`), never
    /// rendered on the page itself.
    ///
    /// Separate from `hero` because the two have incompatible requirements
    /// and `freebsd-on-hetzner` is the post where that stopped being
    /// theoretical: its hero is `/images/freebsd-on-hetzner/header.svg`,
    /// which is the right choice on the page — `main.rs` inlines it, so it
    /// follows the theme toggle instead of being stuck on
    /// `prefers-color-scheme` — and simultaneously useless to every crawler,
    /// because Facebook, LinkedIn and Twitter/X all refuse to render an SVG
    /// as a share image (see `main::OG_IMAGE_EXTENSIONS`). Without this
    /// field the post either loses its theme-aware hero or shares as a bare
    /// link with no image at all. With it, the page keeps the SVG and the
    /// crawler gets a PNG.
    ///
    /// Must itself be a raster file; a `card` pointing at an SVG is the same
    /// bug wearing a different name, and `main::og_image` refuses to build
    /// rather than emit it.
    #[serde(default)]
    pub card: Option<String>,
    /// Alt text for `card`, and never reused from `hero_alt`. The two images
    /// show different things -- on `freebsd-on-hetzner` the hero is a disk
    /// layout and the card is a latency chart -- so describing one with the
    /// other's words is wrong in the place it is least likely to be noticed,
    /// since `og:image:alt` is only ever read aloud by someone else's client.
    #[serde(default)]
    pub card_alt: Option<String>,
}

#[derive(Debug)]
pub struct Post {
    pub path: String,
    pub title: String,
    pub date: NaiveDate,
    pub description: String,
    pub hero: Option<String>,
    pub hero_alt: Option<String>,
    /// The link-share card, if the post carries one. See
    /// [`FrontMatter::card`] for why this is not just `hero`.
    pub card: Option<String>,
    /// Alt text for `card`. See [`FrontMatter::card_alt`].
    pub card_alt: Option<String>,
    /// Rendered HTML, not source.
    pub body: String,
    /// Set by `main.rs`, after loading, when `hero` points at a local
    /// `.svg`: the pre-rendered `<figure>` markup produced the same way the
    /// body inlines a diagram, so the hero inherits the page theme too. A
    /// template cannot read the file itself, hence this is filled in at load
    /// time rather than computed lazily by the template. `None` for a raster
    /// hero (or no hero at all), in which case the template falls back to a
    /// plain `<img>`.
    pub hero_html: Option<String>,
}

impl Post {
    pub fn url(&self) -> String {
        format!("/{}/", self.path)
    }

    /// "March 3, 2025"
    pub fn date_long(&self) -> String {
        self.date.format("%B %e, %Y").to_string().replace("  ", " ")
    }

    pub fn date_iso(&self) -> String {
        self.date.format("%Y-%m-%d").to_string()
    }

    pub fn date_rfc2822(&self) -> String {
        self.date
            .and_hms_opt(0, 0, 0)
            .map(|dt| dt.and_utc().to_rfc2822())
            .unwrap_or_default()
    }

    pub fn alt(&self) -> &str {
        self.hero_alt.as_deref().unwrap_or_default()
    }

    /// Whether this post renders at least one inlined diagram, and so needs
    /// the lightbox dialog and its script. A post without one ships neither:
    /// the script would bind nothing and return immediately, but it would
    /// still be bytes every reader downloads for no reason.
    ///
    /// Matches the opening tag [`crate::djot::svg_figure`] emits, which is the
    /// only thing that produces this class.
    pub fn has_diagrams(&self) -> bool {
        const FIGURE: &str = "<figure class=\"diagram\"";
        self.hero_html
            .as_deref()
            .is_some_and(|h| h.contains(FIGURE))
            || self.body.contains(FIGURE)
    }

    /// Whether this post needs `syntax.css`. Matches the exact wrapper
    /// [`crate::highlight::Highlighter::to_html`] emits for every fenced code
    /// block it highlights, regardless of language (even an unrecognised one
    /// still gets this wrapper via the plain-text fallback — see
    /// `highlight::tests::falls_back_to_plain_text_for_an_unknown_language`).
    /// An inline `` `code` `` span never reaches the highlighter — djot.rs
    /// only intercepts `Container::CodeBlock`, not `Container::Verbatim` — so
    /// it renders as plain jotdown `<code>` and never sets this true. Without
    /// this check every page would pay for a render-blocking stylesheet it
    /// has no highlighted code to use.
    pub fn has_syntax(&self) -> bool {
        const HIGHLIGHTED: &str = "<pre class=\"hl-code\">";
        self.body.contains(HIGHLIGHTED)
    }
}

/// `+++ toml +++` frontmatter, then the body.
pub fn split_frontmatter(source: &str) -> Result<(FrontMatter, &str)> {
    let rest = source
        .strip_prefix("+++")
        .context("post must start with a +++ frontmatter block")?;
    let (raw, body) = rest
        .split_once("+++")
        .context("frontmatter block is never closed")?;

    Ok((toml::from_str(raw).context("invalid frontmatter")?, body))
}

/// Load every `.dj` file under `dir`, newest first.
pub fn load_posts(dir: &Path, render: impl Fn(&str) -> Result<String>) -> Result<Vec<Post>> {
    let mut posts = Vec::new();

    for entry in WalkDir::new(dir) {
        let entry = entry.with_context(|| format!("walking {}", dir.display()))?;
        if entry.path().extension().is_none_or(|e| e != "dj") {
            continue;
        }

        let file = entry.path();
        let source =
            fs::read_to_string(file).with_context(|| format!("reading {}", file.display()))?;
        let (front, body) =
            split_frontmatter(&source).with_context(|| format!("parsing {}", file.display()))?;

        posts.push(Post {
            path: front.path,
            title: front.title,
            date: front.date,
            description: front.description,
            hero: front.hero,
            hero_alt: front.hero_alt,
            card: front.card,
            card_alt: front.card_alt,
            body: render(body).with_context(|| format!("rendering {}", file.display()))?,
            // Filled in by `main.rs` after `load_posts` returns; it needs
            // the SVG-inlining helper `djot.rs` owns, and this function has
            // no reason to depend on that module.
            hero_html: None,
        });
    }

    // Newest first; ties (two posts sharing a date, e.g. published the same
    // day) break on `path` so ordering is stable across machines instead of
    // depending on unspecified WalkDir order.
    posts.sort_by(|a, b| b.date.cmp(&a.date).then_with(|| a.path.cmp(&b.path)));
    Ok(posts)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"+++
path = "blog/trongate/mx-transition"
title = "Trongate mx-transition attribute"
date = 2025-03-03
description = "MX transition provides an easy way to add animations."
+++

Body text here.
"#;

    #[test]
    fn parses_frontmatter_and_returns_the_body() {
        let (front, body) = split_frontmatter(SAMPLE).unwrap();
        assert_eq!(front.path, "blog/trongate/mx-transition");
        assert_eq!(front.title, "Trongate mx-transition attribute");
        assert_eq!(front.date.to_string(), "2025-03-03");
        assert_eq!(body.trim(), "Body text here.");
    }

    #[test]
    fn keeps_nested_paths_intact() {
        let (front, _) = split_frontmatter(SAMPLE).unwrap();
        let post = Post {
            path: front.path,
            title: front.title,
            date: front.date,
            description: front.description,
            hero: None,
            hero_alt: None,
            card: None,
            card_alt: None,
            body: String::new(),
            hero_html: None,
        };
        // The bug this guards: deriving the slug from a filename would
        // flatten this to /blog/mx-transition and break a live URL.
        assert_eq!(post.url(), "/blog/trongate/mx-transition/");
    }

    /// A `Post` with the diagram-bearing fields set as given, everything else
    /// empty. Only `hero_html` and `body` matter to `has_diagrams`.
    fn post_with(hero_html: Option<&str>, body: &str) -> Post {
        Post {
            path: "blog/x".into(),
            title: "x".into(),
            date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            description: "x".into(),
            hero: None,
            hero_alt: None,
            card: None,
            card_alt: None,
            body: body.into(),
            hero_html: hero_html.map(Into::into),
        }
    }

    #[test]
    fn detects_a_diagram_in_the_hero_or_the_body() {
        // The exact markup svg_figure emits, so the two stay in step.
        let figure = crate::djot::svg_figure("a diagram", "<svg></svg>");

        assert!(post_with(Some(&figure), "").has_diagrams());
        assert!(post_with(None, &format!("<p>text</p>{figure}")).has_diagrams());
        assert!(post_with(Some(&figure), &figure).has_diagrams());
    }

    #[test]
    fn a_post_without_a_diagram_ships_no_lightbox() {
        // A raster hero renders as a plain <img> and gets no figure, so these
        // posts must not pull in the lightbox dialog or its script.
        assert!(!post_with(None, "<p>text</p>").has_diagrams());
        assert!(!post_with(None, "<img src=\"/hero.png\" alt=\"x\" />").has_diagrams());
        assert!(!post_with(None, "<figure class=\"photo\"><svg></svg></figure>").has_diagrams());
    }

    #[test]
    fn detects_a_highlighted_code_block() {
        // The exact wrapper Highlighter::to_html emits, so the two stay in
        // step; a real fenced block always renders through it.
        let highlighted = crate::highlight::Highlighter::new()
            .to_html("x\n", "bash")
            .unwrap();
        assert!(post_with(None, &highlighted).has_syntax());
    }

    #[test]
    fn a_post_with_only_inline_code_spans_does_not_need_syntax_css() {
        // Inline `code` spans are jotdown's own <code>, never routed through
        // the highlighter, so they must not trip this on.
        assert!(!post_with(None, "<p>run <code>ls</code> first</p>").has_syntax());
    }

    #[test]
    fn a_post_with_no_code_at_all_does_not_need_syntax_css() {
        assert!(!post_with(None, "<p>no code here</p>").has_syntax());
    }

    #[test]
    fn rejects_a_post_with_no_frontmatter() {
        assert!(split_frontmatter("just a body").is_err());
    }

    #[test]
    fn rejects_frontmatter_that_is_never_closed() {
        assert!(split_frontmatter("+++\ntitle = \"x\"\n").is_err());
    }

    #[test]
    fn formats_dates_for_display_and_machines() {
        let (front, _) = split_frontmatter(SAMPLE).unwrap();
        let post = Post {
            path: front.path,
            title: front.title,
            date: front.date,
            description: front.description,
            hero: None,
            hero_alt: None,
            card: None,
            card_alt: None,
            body: String::new(),
            hero_html: None,
        };
        assert_eq!(post.date_iso(), "2025-03-03");
        assert_eq!(post.date_long(), "March 3, 2025");
    }

    /// Writes a minimal `.dj` fixture: `{dir}/{slug}.dj` with the given
    /// frontmatter `path`/`date` and body.
    fn write_fixture(dir: &Path, slug: &str, path: &str, date: &str, body: &str) {
        fs::write(
            dir.join(format!("{slug}.dj")),
            format!(
                "+++\npath = \"{path}\"\ntitle = \"{slug}\"\ndate = {date}\ndescription = \"d\"\n+++\n{body}\n"
            ),
        )
        .unwrap();
    }

    #[test]
    fn load_posts_walks_filters_sorts_and_renders() {
        let dir = std::env::temp_dir().join("content-rs-load-posts-fixture");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        // Sort order: three dated posts, deliberately written out of order.
        write_fixture(&dir, "newest", "blog/newest", "2025-01-01", "newest body");
        write_fixture(&dir, "middle", "blog/middle", "2022-06-15", "middle body");
        write_fixture(&dir, "oldest", "blog/oldest", "2020-01-01", "oldest body");
        // Same date as "middle": WalkDir order is unspecified, so without a
        // tie-break on `path` these two could swap between machines/runs.
        write_fixture(
            &dir,
            "middle-b",
            "blog/middle-b",
            "2022-06-15",
            "middle-b body",
        );
        // Nested path: the constraint the whole plan exists to protect,
        // exercised here through the real loading path rather than a
        // hand-built `Post`.
        write_fixture(
            &dir,
            "nested",
            "blog/trongate/mx-transition",
            "2021-06-01",
            "nested body",
        );
        // Extension filtering: neither of these is a `.dj` file, so
        // `load_posts` must skip them entirely.
        fs::write(dir.join("ignored.md"), "not a post").unwrap();
        fs::write(dir.join("ignored.txt"), "also not a post").unwrap();

        // Render closure wiring: wrap the trimmed body distinctively so we
        // can prove the closure actually ran, not just that a body exists.
        let posts = load_posts(&dir, |body| Ok(format!("[{}]", body.trim()))).unwrap();

        assert_eq!(
            posts.len(),
            5,
            "the two non-.dj files must not be loaded as posts"
        );

        let titles: Vec<&str> = posts.iter().map(|p| p.title.as_str()).collect();
        assert_eq!(
            titles,
            vec!["newest", "middle", "middle-b", "nested", "oldest"],
            "posts must come back newest first, with same-date posts \
             (middle, middle-b) tie-broken by path so order is stable"
        );

        let nested = posts.iter().find(|p| p.title == "nested").unwrap();
        assert_eq!(nested.url(), "/blog/trongate/mx-transition/");

        let newest = posts.iter().find(|p| p.title == "newest").unwrap();
        assert_eq!(newest.body, "[newest body]");

        fs::remove_dir_all(&dir).unwrap();
    }

    /// Focused regression test for the tie-break itself: two posts sharing a
    /// date must always come back in the same, path-ordered sequence,
    /// regardless of which one WalkDir happens to visit first.
    #[test]
    fn same_date_posts_are_ordered_by_path_not_by_walk_order() {
        let dir = std::env::temp_dir().join("content-rs-load-posts-same-date-fixture");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        write_fixture(&dir, "b-post", "blog/b-post", "2026-07-26", "b body");
        write_fixture(&dir, "a-post", "blog/a-post", "2026-07-26", "a body");

        let posts = load_posts(&dir, |body| Ok(body.trim().to_string())).unwrap();

        let paths: Vec<&str> = posts.iter().map(|p| p.path.as_str()).collect();
        assert_eq!(
            paths,
            vec!["blog/a-post", "blog/b-post"],
            "same-date posts must sort by path, not by filesystem walk order"
        );

        fs::remove_dir_all(&dir).unwrap();
    }
}
