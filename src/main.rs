//! sasin91.xyz - static site builder.
//!
//! Walks content/, renders through Askama templates, writes ./public.
//! Templates are type-checked against these structs at compile time: a typo
//! in `{{ post.titel }}` is a build error, not a blank space on the page.

mod content;
mod cv;
mod cv_pdf;
mod djot;
mod highlight;
mod html;
mod math;
mod pdf;
mod pdf_metrics;

use anyhow::{Context, Result};
use askama::Template;
use chrono::Datelike;
use content::Post;
use cv::Cv;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

const OUT: &str = "public";
const BASE_URL: &str = "https://sasin91.xyz";

/// Everything the document head needs that differs per page. A struct rather
/// than Askama blocks because the description text is needed twice -- once as
/// `<meta name="description">` and once as `og:description` -- and a block
/// cannot be expanded twice.
struct Meta {
    title: String,
    description: String,
    /// Absolute, because og:url and canonical must both be absolute; a
    /// relative URL is silently ignored by every crawler that reads them.
    url: String,
    /// "website" for the landing, listing and CV pages; "article" for a post.
    og_type: &'static str,
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexPage<'a> {
    cv: &'a Cv,
    posts: &'a [Post],
    year: i32,
    /// Which `site-nav` link `base.html` marks `aria-current="page"` — see
    /// that template's nav block. The home page matches none of them.
    nav: &'static str,
    meta: Meta,
    /// The landing page has no code on it, so `syntax.css` never loads here.
    syntax: bool,
    /// Content-hashed `site.<hash>.css`, computed once in `main` -- see
    /// `hash_css`. Constant for the whole build, so every page shares the
    /// same value rather than each recomputing it.
    site_css: &'a str,
    /// Content-hashed `syntax.<hash>.css`. Present even when `syntax` is
    /// false, since Askama needs the field to exist; `base.html` only
    /// renders the `<link>` when `syntax` is true.
    syntax_css: &'a str,
}

#[derive(Template)]
#[template(path = "about.html")]
struct AboutPage<'a> {
    cv: &'a Cv,
    year: i32,
    nav: &'static str,
    meta: Meta,
    syntax: bool,
    site_css: &'a str,
    syntax_css: &'a str,
}

#[derive(Template)]
#[template(path = "cv.html")]
struct CvPage<'a> {
    cv: &'a Cv,
    year: i32,
    nav: &'static str,
    meta: Meta,
    syntax: bool,
    site_css: &'a str,
    syntax_css: &'a str,
}

#[derive(Template)]
#[template(path = "blog.html")]
struct BlogPage<'a> {
    cv: &'a Cv,
    posts: &'a [Post],
    year: i32,
    nav: &'static str,
    meta: Meta,
    /// The listing shows only titles and descriptions, never a post's body,
    /// so there is never highlighted code on this page either.
    syntax: bool,
    site_css: &'a str,
    syntax_css: &'a str,
}

#[derive(Template)]
#[template(path = "post.html")]
struct PostPage<'a> {
    cv: &'a Cv,
    post: &'a Post,
    year: i32,
    /// A post lives under /blog/, so "Writing" stays highlighted while
    /// reading one, matching the URL a reader is actually on.
    nav: &'static str,
    meta: Meta,
    /// Only true when `post.has_syntax()` found the highlighter's own
    /// wrapper in the rendered body — never hardcoded, since most posts
    /// mix prose with code and some (see `Post::has_syntax`) have none.
    syntax: bool,
    site_css: &'a str,
    syntax_css: &'a str,
}

#[derive(Template)]
#[template(path = "rss.xml", escape = "xml")]
struct Feed<'a> {
    cv: &'a Cv,
    posts: &'a [Post],
    base: &'a str,
    /// The newest post's date, RFC 2822. Deliberately not the wall clock at
    /// build time — that would make output non-deterministic and defeat the
    /// stable post ordering in `content::load_posts`.
    last_build_date: &'a str,
}

#[derive(Template)]
#[template(path = "sitemap.xml", escape = "xml")]
struct Sitemap<'a> {
    posts: &'a [Post],
    base: &'a str,
}

/// FNV-1a over an asset's own bytes, formatted as fixed-width lowercase hex.
///
/// Duplicated rather than reusing `pdf::fnv1a` (private, and scoped to the
/// PDF `/ID`): a six-line hash copied here is cheaper than making the PDF
/// writer's internals `pub` just so the asset pipeline could borrow them, and
/// it keeps the two writers free to change their hashing independently.
///
/// Must be a pure function of the bytes -- no timestamps, no filesystem
/// paths, no iteration order -- because the deploy hardlinks unchanged files
/// from the previous release (`rsync --link-dest`, see docs/deploy.md); a
/// hash that moved without the content moving would re-transfer both
/// stylesheets and invalidate every client's cache on every deploy for no
/// reason. `{:016x}` rather than bare `{:x}` so the digest is always 16
/// characters -- a leading zero must not silently shrink the name below the
/// Caddyfile's `\.[0-9a-f]{8,}\.css$` immutable-cache rule.
fn hash_css(bytes: &[u8]) -> String {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{hash:016x}")
}

/// Builds the on-disk name for a hashed asset: `<stem>.<hash>.css`. This is
/// the only seam holding the builder and the Caddyfile's cache rule (see
/// `docs/deploy.md`) together -- that rule matches `\.[0-9a-f]{8,}\.css$`,
/// so this exact construction is what the test below checks against that
/// shape directly, rather than trusting `hash_css`'s own format in
/// isolation. A typo here (a dropped dot, a different separator) would
/// compile, pass every other test, build fine, and only show up as the
/// Caddy rule silently failing to match once it's live.
fn hashed_css_name(stem: &str, bytes: &[u8]) -> String {
    format!("{stem}.{}.css", hash_css(bytes))
}

fn write(path: impl AsRef<Path>, contents: &str) -> Result<()> {
    let path = path.as_ref();
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).with_context(|| format!("creating directory {}", dir.display()))?;
    }
    fs::write(path, contents).with_context(|| format!("writing {}", path.display()))
}

/// Walks `static/` into `public/`, renaming `site.css` to its content-hashed
/// name (`site_css`, e.g. `site.<hash>.css`) on the way through. Renaming
/// during the copy rather than after means an unhashed `public/site.css` is
/// never created in the first place -- there is no stray file to remember to
/// delete, and no window where both the old and new names exist and a page
/// could end up linking the wrong one.
fn copy_static(site_css: &str) -> Result<()> {
    let mut copied = 0usize;

    for entry in WalkDir::new("static") {
        let entry = entry.context("walking static/ (is it missing or unreadable?)")?;
        if !entry.file_type().is_file() {
            continue;
        }
        let rel = entry.path().strip_prefix("static")?;
        let dest = if rel == Path::new("site.css") {
            Path::new(OUT).join(site_css)
        } else {
            Path::new(OUT).join(rel)
        };
        if let Some(dir) = dest.parent() {
            fs::create_dir_all(dir)
                .with_context(|| format!("creating directory {}", dir.display()))?;
        }
        fs::copy(entry.path(), &dest)
            .with_context(|| format!("copying {} to {}", entry.path().display(), dest.display()))?;
        copied += 1;
    }

    anyhow::ensure!(
        copied > 0,
        "static/ produced zero files — refusing to build with an empty asset set \
         (a deploy would delete every live asset)"
    );

    Ok(())
}

/// A post's hero comes from frontmatter and, unlike the body, never passes
/// through `djot::render` -- `templates/post.html` renders it straight to an
/// `<img>`. Left alone, that `<img>` would keep loading the SVG as its own
/// document, which can only see `prefers-color-scheme` and so ignores the
/// theme toggle -- exactly the problem inlining fixed for images in the
/// body. A template cannot read files, so do the same inlining here, once
/// posts are loaded, and stash the result on `hero_html` for the template to
/// prefer over the raw `<img>` when present.
fn inline_svg_heroes(posts: &mut [Post]) -> Result<()> {
    for post in posts {
        let Some(hero) = &post.hero else { continue };
        if !djot::is_local_svg(hero) {
            continue;
        }

        let path = Path::new("static").join(hero.trim_start_matches('/'));
        let svg = fs::read_to_string(&path)
            .with_context(|| format!("missing SVG referenced by post hero: {hero}"))?;
        post.hero_html = Some(djot::svg_figure(post.alt(), &svg));
    }

    Ok(())
}

fn main() -> Result<()> {
    let started = std::time::Instant::now();

    let hl = highlight::Highlighter::new();
    let cv: Cv = toml::from_str(&fs::read_to_string("content/cv.toml")?)
        .context("parsing content/cv.toml")?;
    cv.validate().context("content/cv.toml has a bad date")?;
    let mut posts = content::load_posts(Path::new("content/blog"), |body| djot::render(body, &hl))?;
    inline_svg_heroes(&mut posts)?;
    let year = chrono::Local::now().year();

    // Hashed once here, not per page: the hash is a pure function of each
    // file's own bytes (see `hash_css`), so it is the same value on every
    // page and recomputing it per page would just repeat the same read.
    let site_css_bytes =
        fs::read("static/site.css").context("reading static/site.css to hash it")?;
    let site_css = hashed_css_name("site", &site_css_bytes);
    let syntax_css_body = hl.stylesheet("Solarized (light)", "base16-ocean.dark")?;
    let syntax_css = hashed_css_name("syntax", syntax_css_body.as_bytes());

    if Path::new(OUT).exists() {
        fs::remove_dir_all(OUT).with_context(|| {
            format!(
                "removing {OUT}/ (is a server still serving it? \
                 stop anything with {OUT}/ as its working directory and retry)"
            )
        })?;
    }
    copy_static(&site_css)?;

    write(format!("{OUT}/{syntax_css}"), &syntax_css_body)?;

    write(
        format!("{OUT}/index.html"),
        &IndexPage {
            cv: &cv,
            posts: &posts,
            year,
            nav: "",
            meta: Meta {
                title: format!("{} — {}", cv.site.name, cv.site.title),
                description: cv.site.stack_line(),
                url: format!("{BASE_URL}/"),
                og_type: "website",
            },
            syntax: false,
            site_css: &site_css,
            syntax_css: &syntax_css,
        }
        .render()?,
    )?;
    write(
        format!("{OUT}/about/index.html"),
        &AboutPage {
            cv: &cv,
            year,
            nav: "about",
            meta: Meta {
                title: format!("About — {}", cv.site.name),
                description: format!(
                    "{} in {}. {}.",
                    cv.site.title, cv.contact.town, cv.site.available_note
                ),
                url: format!("{BASE_URL}/about/"),
                og_type: "website",
            },
            syntax: false,
            site_css: &site_css,
            syntax_css: &syntax_css,
        }
        .render()?,
    )?;
    write(
        format!("{OUT}/cv/index.html"),
        &CvPage {
            cv: &cv,
            year,
            nav: "cv",
            meta: Meta {
                title: format!("CV — {}", cv.site.name),
                description: format!(
                    "{}, {}. {}, {}.",
                    cv.site.name, cv.site.title, cv.contact.town, cv.contact.postcode
                ),
                url: format!("{BASE_URL}/cv/"),
                og_type: "website",
            },
            syntax: false,
            site_css: &site_css,
            syntax_css: &syntax_css,
        }
        .render()?,
    )?;
    // Generated from the same `Cv` as the page above, so the two cannot carry
    // different content. This used to be a CI step that pointed headless Chrome
    // at a local server; that server once resolved /cv/ to the homepage and the
    // site shipped the landing page as cv.pdf for several deploys. There is no
    // URL to get wrong here.
    fs::write(format!("{OUT}/cv.pdf"), cv_pdf::render(&cv)).context("writing public/cv.pdf")?;
    write(
        format!("{OUT}/blog/index.html"),
        &BlogPage {
            cv: &cv,
            posts: &posts,
            year,
            nav: "blog",
            meta: Meta {
                title: format!("Writing — {}", cv.site.name),
                description: "Notes on things I built and what broke on the way.".to_string(),
                url: format!("{BASE_URL}/blog/"),
                og_type: "website",
            },
            syntax: false,
            site_css: &site_css,
            syntax_css: &syntax_css,
        }
        .render()?,
    )?;

    for post in &posts {
        write(
            format!("{OUT}/{}/index.html", post.path),
            &PostPage {
                cv: &cv,
                post,
                year,
                nav: "blog",
                meta: Meta {
                    title: format!("{} — {}", post.title, cv.site.name),
                    description: post.description.clone(),
                    url: format!("{BASE_URL}{}", post.url()),
                    og_type: "article",
                },
                syntax: post.has_syntax(),
                site_css: &site_css,
                syntax_css: &syntax_css,
            }
            .render()?,
        )?;
    }

    let last_build_date = posts.first().map(|p| p.date_rfc2822()).unwrap_or_default();
    write(
        format!("{OUT}/rss.xml"),
        &Feed {
            cv: &cv,
            posts: &posts,
            base: BASE_URL,
            last_build_date: &last_build_date,
        }
        .render()?,
    )?;
    write(
        format!("{OUT}/sitemap.xml"),
        &Sitemap {
            posts: &posts,
            base: BASE_URL,
        }
        .render()?,
    )?;
    write(
        format!("{OUT}/robots.txt"),
        &format!("User-agent: *\nAllow: /\nSitemap: {BASE_URL}/sitemap.xml\n"),
    )?;

    println!(
        "built {} posts in {:.0?} -> {OUT}/",
        posts.len(),
        started.elapsed()
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Minimal but complete `Cv` TOML — same shape as `cv::tests::cv_with_dates`,
    /// trimmed to the fields these page templates actually read.
    fn cv_fixture() -> Cv {
        let src = r#"
intro = []
about = []
roles = []
skills = []
education = []

[site]
name = "x"
title = "x"
stack = ["x"]
available = false
available_note = "x"

[site.links]
github = "https://github.com/x"
linkedin = "x"
email = "x"

[contact]
town = "x"
postcode = "x"
phone = "x"
email = "x"
"#;
        toml::from_str(src).expect("fixture TOML must itself be well-formed")
    }

    fn post_fixture() -> Post {
        Post {
            path: "blog/x".into(),
            title: "x".into(),
            date: chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            description: "x".into(),
            hero: None,
            hero_alt: None,
            body: String::new(),
            hero_html: None,
        }
    }

    /// A `Meta` whose `url` is absolute under `BASE_URL`, matching what
    /// `main` builds for a real page — content beyond that doesn't matter to
    /// the tests that use this fixture.
    fn meta_fixture(path: &str, og_type: &'static str) -> Meta {
        Meta {
            title: "x".into(),
            description: "x".into(),
            url: format!("{BASE_URL}{path}"),
            og_type,
        }
    }

    /// Stand-ins for the hashed names `main` computes once per real build.
    /// Fixed strings, not `hash_css` output, because these tests care about
    /// the value being threaded through to every page unchanged, not about
    /// hashing itself (that property has its own tests below).
    const SITE_CSS_FIXTURE: &str = "site.aaaaaaaaaaaaaaaa.css";
    const SYNTAX_CSS_FIXTURE: &str = "syntax.bbbbbbbbbbbbbbbb.css";

    /// Every page must mark exactly one `site-nav` link (or none, for the
    /// home page) with `aria-current="page"` — the bug this guards is the
    /// nav rendering identically on every page, so no link is ever marked
    /// active.
    fn asserts_single_nav_link_current(html: &str, expected: &[&str]) {
        for label in ["Writing", "About", "CV"] {
            let marked = html.contains(&format!("aria-current=\"page\">{label}"));
            assert_eq!(
                marked,
                expected.contains(&label),
                "{label} link's aria-current=\"page\" state is wrong in:\n{html}"
            );
        }
    }

    #[test]
    fn index_page_highlights_no_nav_link() {
        let cv = cv_fixture();
        let posts = [];
        let html = IndexPage {
            cv: &cv,
            posts: &posts,
            year: 2026,
            nav: "",
            meta: meta_fixture("/", "website"),
            syntax: false,
            site_css: SITE_CSS_FIXTURE,
            syntax_css: SYNTAX_CSS_FIXTURE,
        }
        .render()
        .unwrap();
        asserts_single_nav_link_current(&html, &[]);
    }

    #[test]
    fn about_page_highlights_the_about_link() {
        let cv = cv_fixture();
        let html = AboutPage {
            cv: &cv,
            year: 2026,
            nav: "about",
            meta: meta_fixture("/about/", "website"),
            syntax: false,
            site_css: SITE_CSS_FIXTURE,
            syntax_css: SYNTAX_CSS_FIXTURE,
        }
        .render()
        .unwrap();
        asserts_single_nav_link_current(&html, &["About"]);
    }

    #[test]
    fn cv_page_highlights_the_cv_link() {
        let cv = cv_fixture();
        let html = CvPage {
            cv: &cv,
            year: 2026,
            nav: "cv",
            meta: meta_fixture("/cv/", "website"),
            syntax: false,
            site_css: SITE_CSS_FIXTURE,
            syntax_css: SYNTAX_CSS_FIXTURE,
        }
        .render()
        .unwrap();
        asserts_single_nav_link_current(&html, &["CV"]);
    }

    #[test]
    fn blog_page_highlights_the_writing_link() {
        let cv = cv_fixture();
        let posts = [];
        let html = BlogPage {
            cv: &cv,
            posts: &posts,
            year: 2026,
            nav: "blog",
            meta: meta_fixture("/blog/", "website"),
            syntax: false,
            site_css: SITE_CSS_FIXTURE,
            syntax_css: SYNTAX_CSS_FIXTURE,
        }
        .render()
        .unwrap();
        asserts_single_nav_link_current(&html, &["Writing"]);
    }

    /// A post lives under /blog/, so it highlights the same link the blog
    /// index does rather than none at all.
    #[test]
    fn post_page_highlights_the_writing_link() {
        let cv = cv_fixture();
        let post = post_fixture();
        let html = PostPage {
            cv: &cv,
            post: &post,
            year: 2026,
            nav: "blog",
            meta: meta_fixture(&post.url(), "article"),
            syntax: false,
            site_css: SITE_CSS_FIXTURE,
            syntax_css: SYNTAX_CSS_FIXTURE,
        }
        .render()
        .unwrap();
        asserts_single_nav_link_current(&html, &["Writing"]);
    }

    /// Every page's head must carry exactly one of each of these — two would
    /// mean a block got expanded twice (the reason `Meta` replaced Askama
    /// blocks: see the struct's doc comment), zero would mean the head
    /// markup silently dropped out of `base.html`.
    fn all_pages_html() -> Vec<(&'static str, String)> {
        let cv = cv_fixture();
        let post = post_fixture();
        let posts = [post_fixture()];

        vec![
            (
                "index",
                IndexPage {
                    cv: &cv,
                    posts: &posts,
                    year: 2026,
                    nav: "",
                    meta: meta_fixture("/", "website"),
                    syntax: false,
                    site_css: SITE_CSS_FIXTURE,
                    syntax_css: SYNTAX_CSS_FIXTURE,
                }
                .render()
                .unwrap(),
            ),
            (
                "about",
                AboutPage {
                    cv: &cv,
                    year: 2026,
                    nav: "about",
                    meta: meta_fixture("/about/", "website"),
                    syntax: false,
                    site_css: SITE_CSS_FIXTURE,
                    syntax_css: SYNTAX_CSS_FIXTURE,
                }
                .render()
                .unwrap(),
            ),
            (
                "cv",
                CvPage {
                    cv: &cv,
                    year: 2026,
                    nav: "cv",
                    meta: meta_fixture("/cv/", "website"),
                    syntax: false,
                    site_css: SITE_CSS_FIXTURE,
                    syntax_css: SYNTAX_CSS_FIXTURE,
                }
                .render()
                .unwrap(),
            ),
            (
                "blog",
                BlogPage {
                    cv: &cv,
                    posts: &posts,
                    year: 2026,
                    nav: "blog",
                    meta: meta_fixture("/blog/", "website"),
                    syntax: false,
                    site_css: SITE_CSS_FIXTURE,
                    syntax_css: SYNTAX_CSS_FIXTURE,
                }
                .render()
                .unwrap(),
            ),
            (
                "post",
                PostPage {
                    cv: &cv,
                    post: &post,
                    year: 2026,
                    nav: "blog",
                    meta: meta_fixture(&post.url(), "article"),
                    syntax: false,
                    site_css: SITE_CSS_FIXTURE,
                    syntax_css: SYNTAX_CSS_FIXTURE,
                }
                .render()
                .unwrap(),
            ),
        ]
    }

    #[test]
    fn every_page_emits_exactly_one_title_description_and_canonical() {
        for (name, html) in all_pages_html() {
            assert_eq!(html.matches("<title>").count(), 1, "{name}: title\n{html}");
            assert_eq!(
                html.matches("<meta name=\"description\"").count(),
                1,
                "{name}: description\n{html}"
            );
            assert_eq!(
                html.matches("<link rel=\"canonical\"").count(),
                1,
                "{name}: canonical\n{html}"
            );
        }
    }

    /// A relative `og:url` is silently ignored by every crawler that reads
    /// it — the exact bug `Meta::url`'s doc comment names — so every page's
    /// value must start with `BASE_URL`, not merely be present.
    #[test]
    fn og_url_is_absolute_on_every_page() {
        let marker = format!("property=\"og:url\" content=\"{BASE_URL}");
        for (name, html) in all_pages_html() {
            assert!(html.contains(&marker), "{name}: {html}");
        }
    }

    /// Guards against `og:image` being added before a real 1200x630 card
    /// asset exists — see the comment in `templates/base.html` explaining
    /// why a missing or undersized image is worse than none at all.
    #[test]
    fn no_page_emits_an_og_image() {
        for (name, html) in all_pages_html() {
            assert!(!html.contains("og:image"), "{name}: {html}");
        }
    }

    #[test]
    fn a_page_with_no_code_does_not_link_syntax_css() {
        let cv = cv_fixture();
        let html = AboutPage {
            cv: &cv,
            year: 2026,
            nav: "about",
            meta: meta_fixture("/about/", "website"),
            syntax: false,
            site_css: SITE_CSS_FIXTURE,
            syntax_css: SYNTAX_CSS_FIXTURE,
        }
        .render()
        .unwrap();
        assert!(!html.contains(SYNTAX_CSS_FIXTURE), "{html}");
    }

    #[test]
    fn a_post_that_contains_code_links_syntax_css() {
        let cv = cv_fixture();
        let post = post_fixture();
        let html = PostPage {
            cv: &cv,
            post: &post,
            year: 2026,
            nav: "blog",
            meta: meta_fixture(&post.url(), "article"),
            syntax: true,
            site_css: SITE_CSS_FIXTURE,
            syntax_css: SYNTAX_CSS_FIXTURE,
        }
        .render()
        .unwrap();
        assert!(html.contains(SYNTAX_CSS_FIXTURE), "{html}");
    }

    /// The whole caching scheme rests on this: unchanged bytes must hash
    /// identically across builds (or `rsync --link-dest` cannot hardlink and
    /// every deploy re-transfers the stylesheet), and changed bytes must hash
    /// differently (or a stale, long-cached copy would keep being served
    /// under a name nothing tells the browser to drop).
    #[test]
    fn hash_css_is_stable_for_the_same_bytes_and_changes_with_the_content() {
        let a = hash_css(b"body { color: red; }");
        let b = hash_css(b"body { color: red; }");
        let c = hash_css(b"body { color: blue; }");
        assert_eq!(a, b, "identical bytes must hash identically");
        assert_ne!(a, c, "a single changed byte must change the hash");
    }

    /// Mirrors the Caddyfile's `\.[0-9a-f]{8,}\.css$` immutable-cache matcher
    /// (see docs/deploy.md) character-for-character, without pulling in a
    /// regex crate for the one pattern: a literal dot, 8+ lowercase hex
    /// digits, then a literal `.css` at the end of the string.
    fn matches_caddy_hashed_css_pattern(name: &str) -> bool {
        let Some(before_css) = name.strip_suffix(".css") else {
            return false;
        };
        let Some(dot_at) = before_css.rfind('.') else {
            return false;
        };
        let hash_part = &before_css[dot_at + 1..];
        hash_part.len() >= 8
            && hash_part
                .chars()
                .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
    }

    /// Exercises the real `hashed_css_name` construction `main` actually
    /// calls -- not just `hash_css`'s own output in isolation, and not a
    /// hand-written fixture already shaped to pass. A dropped dot or a
    /// changed separator here would compile, pass every other test in this
    /// suite, and build fine locally; it would only show up as the Caddy
    /// rule above silently never matching once the site was live, with no
    /// error and no failed request to point at it.
    #[test]
    fn hashed_css_name_matches_the_caddy_immutable_pattern() {
        for name in [
            hashed_css_name("site", b"some stylesheet bytes"),
            hashed_css_name("syntax", b"some other stylesheet bytes"),
        ] {
            assert!(matches_caddy_hashed_css_pattern(&name), "got: {name}");
        }
    }

    /// A single stale `/site.css` or `/syntax.css` reference is a 404
    /// stylesheet on a live page -- every page must link only the hashed
    /// names threaded through from `main`, never the bare ones.
    #[test]
    fn every_page_links_only_hashed_stylesheet_names() {
        for (name, html) in all_pages_html() {
            assert!(
                html.contains(&format!("href=\"/{SITE_CSS_FIXTURE}\"")),
                "{name}: missing hashed site.css link\n{html}"
            );
            assert!(
                !html.contains("href=\"/site.css\""),
                "{name}: linked bare /site.css\n{html}"
            );
            assert!(
                !html.contains("href=\"/syntax.css\""),
                "{name}: linked bare /syntax.css\n{html}"
            );
        }
    }
}
