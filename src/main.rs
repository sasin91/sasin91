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

#[derive(Template)]
#[template(path = "index.html")]
struct IndexPage<'a> {
    cv: &'a Cv,
    posts: &'a [Post],
    year: i32,
    /// Which `site-nav` link `base.html` marks `aria-current="page"` — see
    /// that template's nav block. The home page matches none of them.
    nav: &'static str,
}

#[derive(Template)]
#[template(path = "about.html")]
struct AboutPage<'a> {
    cv: &'a Cv,
    year: i32,
    nav: &'static str,
}

#[derive(Template)]
#[template(path = "cv.html")]
struct CvPage<'a> {
    cv: &'a Cv,
    year: i32,
    nav: &'static str,
}

#[derive(Template)]
#[template(path = "blog.html")]
struct BlogPage<'a> {
    cv: &'a Cv,
    posts: &'a [Post],
    year: i32,
    nav: &'static str,
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

fn write(path: impl AsRef<Path>, contents: &str) -> Result<()> {
    let path = path.as_ref();
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).with_context(|| format!("creating directory {}", dir.display()))?;
    }
    fs::write(path, contents).with_context(|| format!("writing {}", path.display()))
}

fn copy_static() -> Result<()> {
    let mut copied = 0usize;

    for entry in WalkDir::new("static") {
        let entry = entry.context("walking static/ (is it missing or unreadable?)")?;
        if !entry.file_type().is_file() {
            continue;
        }
        let rel = entry.path().strip_prefix("static")?;
        let dest = Path::new(OUT).join(rel);
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

    if Path::new(OUT).exists() {
        fs::remove_dir_all(OUT).with_context(|| {
            format!(
                "removing {OUT}/ (is a server still serving it? \
                 stop anything with {OUT}/ as its working directory and retry)"
            )
        })?;
    }
    copy_static()?;

    write(
        format!("{OUT}/syntax.css"),
        &hl.stylesheet("Solarized (light)", "base16-ocean.dark")?,
    )?;

    write(
        format!("{OUT}/index.html"),
        &IndexPage {
            cv: &cv,
            posts: &posts,
            year,
            nav: "",
        }
        .render()?,
    )?;
    write(
        format!("{OUT}/about/index.html"),
        &AboutPage {
            cv: &cv,
            year,
            nav: "about",
        }
        .render()?,
    )?;
    write(
        format!("{OUT}/cv/index.html"),
        &CvPage {
            cv: &cv,
            year,
            nav: "cv",
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
        }
        .render()
        .unwrap();
        asserts_single_nav_link_current(&html, &["Writing"]);
    }
}
