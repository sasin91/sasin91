//! sasin91.xyz - static site builder.
//!
//! Walks content/, renders through Askama templates, writes ./public.
//! Templates are type-checked against these structs at compile time: a typo
//! in `{{ post.titel }}` is a build error, not a blank space on the page.

mod content;
mod djot;
mod highlight;
mod html;
mod math;

use anyhow::{Context, Result, anyhow};
use askama::Template;
use chrono::{Datelike, NaiveDate};
use content::Post;
use serde::Deserialize;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

const OUT: &str = "public";
const BASE_URL: &str = "https://sasin91.xyz";

#[derive(Deserialize)]
pub struct Cv {
    pub site: Profile,
    pub contact: Contact,
    pub intro: Vec<String>,
    pub roles: Vec<Role>,
    pub skills: Vec<Skill>,
    pub education: Vec<Education>,
    /// Descriptor for the education section as a whole (e.g. "Two
    /// short-cycle higher educations."), not tied to any single entry.
    #[serde(default)]
    pub education_note: Option<String>,
}

impl Cv {
    /// Every `start`/`end` in `roles` and `education` must parse as
    /// `YYYY-MM`. This is a person's employment history used in job
    /// applications, so a malformed date must stop the build — with the
    /// offending value and field named — rather than let `month()` fall back
    /// to rendering the raw TOML string on the live `/cv` page and emitting
    /// an invalid `<time datetime>`. Called once, right after parsing
    /// `content/cv.toml`, so every template-facing `*_label()` method below
    /// can assume its date is already valid and stay infallible.
    fn validate(&self) -> Result<()> {
        for role in &self.roles {
            month(&role.start).with_context(|| {
                format!("role {:?} at {:?}: start date", role.title, role.company)
            })?;
            if let Some(end) = &role.end {
                month(end).with_context(|| {
                    format!("role {:?} at {:?}: end date", role.title, role.company)
                })?;
            }
        }
        for edu in &self.education {
            month(&edu.start).with_context(|| {
                format!("education {:?} at {:?}: start date", edu.title, edu.school)
            })?;
            if let Some(end) = &edu.end {
                month(end).with_context(|| {
                    format!("education {:?} at {:?}: end date", edu.title, edu.school)
                })?;
            }
        }
        Ok(())
    }
}

#[derive(Deserialize)]
pub struct Profile {
    pub name: String,
    pub title: String,
    pub stack: String,
    pub available: bool,
    pub available_note: String,
    pub links: Links,
}

#[derive(Deserialize)]
pub struct Links {
    pub github: String,
    pub linkedin: String,
    pub email: String,
}

#[derive(Deserialize)]
pub struct Contact {
    pub town: String,
    pub postcode: String,
    pub phone: String,
    pub email: String,
}

impl Contact {
    /// `phone` with its spaces stripped, for a `tel:` href — RFC 3966 has no
    /// concept of the visual grouping spaces `+45 50106917` has for a
    /// reader, and a raw space in the URI is invalid there. `phone` itself
    /// is left untouched for display, where the spacing helps.
    pub fn phone_href(&self) -> String {
        self.phone.replace(' ', "")
    }
}

#[derive(Deserialize)]
pub struct Role {
    pub start: String,
    pub end: Option<String>,
    pub title: String,
    pub company: String,
    pub location: String,
    pub summary: String,
    #[serde(default)]
    pub achievements: Vec<String>,
}

/// "2024-09" -> "September 2024". Errors name the bad value so a caller can
/// report which field it came from; see `Cv::validate`, which is the only
/// place this can fail once the build is past parsing `content/cv.toml` —
/// every `*_label()` method below relies on that and unwraps.
fn month(m: &str) -> Result<String> {
    NaiveDate::parse_from_str(&format!("{m}-01"), "%Y-%m-%d")
        .map(|d| d.format("%B %Y").to_string())
        .map_err(|e| anyhow!("invalid date {m:?} (want YYYY-MM): {e}"))
}

impl Role {
    /// "September 2024", from `start`.
    pub fn start_label(&self) -> String {
        month(&self.start).expect("date already validated by Cv::validate")
    }

    /// "February 2026", from `end` — `None` while the role is still open, so
    /// a template can tell "ended" from "ongoing" rather than guessing from
    /// an empty string.
    pub fn end_label(&self) -> Option<String> {
        self.end
            .as_deref()
            .map(|e| month(e).expect("date already validated by Cv::validate"))
    }
}

#[derive(Deserialize)]
pub struct Skill {
    pub name: String,
}

#[derive(Deserialize)]
pub struct Education {
    pub start: String,
    pub end: Option<String>,
    pub title: String,
    pub school: String,
    pub location: String,
    #[serde(default)]
    pub note: Option<String>,
}

impl Education {
    /// "February 2014", from `start`.
    pub fn start_label(&self) -> String {
        month(&self.start).expect("date already validated by Cv::validate")
    }

    /// "August 2015", from `end` — `None` while ongoing.
    pub fn end_label(&self) -> Option<String> {
        self.end
            .as_deref()
            .map(|e| month(e).expect("date already validated by Cv::validate"))
    }
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexPage<'a> {
    cv: &'a Cv,
    posts: &'a [Post],
    year: i32,
}

#[derive(Template)]
#[template(path = "about.html")]
struct AboutPage<'a> {
    cv: &'a Cv,
    year: i32,
}

#[derive(Template)]
#[template(path = "cv.html")]
struct CvPage<'a> {
    cv: &'a Cv,
    year: i32,
}

#[derive(Template)]
#[template(path = "blog.html")]
struct BlogPage<'a> {
    cv: &'a Cv,
    posts: &'a [Post],
    year: i32,
}

#[derive(Template)]
#[template(path = "post.html")]
struct PostPage<'a> {
    cv: &'a Cv,
    post: &'a Post,
    year: i32,
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
        }
        .render()?,
    )?;
    write(
        format!("{OUT}/about/index.html"),
        &AboutPage { cv: &cv, year }.render()?,
    )?;
    write(
        format!("{OUT}/cv/index.html"),
        &CvPage { cv: &cv, year }.render()?,
    )?;
    write(
        format!("{OUT}/blog/index.html"),
        &BlogPage {
            cv: &cv,
            posts: &posts,
            year,
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

    /// Minimal but complete `Cv` TOML: every field with no `#[serde(default)]`
    /// is present, so this only fails to parse if the date under test is bad.
    fn cv_with_dates(role_start: &str, role_end: &str, edu_start: &str, edu_end: &str) -> Cv {
        let src = format!(
            r#"
intro = []

[site]
name = "x"
title = "x"
stack = "x"
available = true
available_note = "x"

[site.links]
github = "x"
linkedin = "x"
email = "x"

[contact]
town = "x"
postcode = "x"
phone = "x"
email = "x"

[[roles]]
start = "{role_start}"
end = "{role_end}"
title = "Web developer"
company = "Acme"
location = "x"
summary = "x"

[[skills]]
name = "x"

[[education]]
start = "{edu_start}"
end = "{edu_end}"
title = "Diploma"
school = "Acme Tech"
location = "x"
"#
        );
        toml::from_str(&src).expect("fixture TOML must itself be well-formed")
    }

    #[test]
    fn month_formats_a_valid_year_month() {
        assert_eq!(month("2024-09").unwrap(), "September 2024");
    }

    #[test]
    fn month_rejects_an_out_of_range_month() {
        let err = month("2024-13").unwrap_err();
        assert!(
            err.to_string().contains("2024-13"),
            "error should name the bad value: {err}"
        );
    }

    #[test]
    fn validate_accepts_well_formed_dates() {
        let cv = cv_with_dates("2024-09", "2026-02", "2014-02", "2015-08");
        assert!(cv.validate().is_ok());
    }

    #[test]
    fn validate_rejects_a_malformed_role_start_date() {
        let cv = cv_with_dates("2024-13", "2026-02", "2014-02", "2015-08");
        let err = cv.validate().unwrap_err();
        let msg = format!("{err:#}");
        assert!(msg.contains("2024-13"), "should name the bad value: {msg}");
        assert!(msg.contains("start date"), "should name the field: {msg}");
        assert!(msg.contains("Acme"), "should name which role: {msg}");
    }

    #[test]
    fn validate_rejects_a_malformed_education_end_date() {
        let cv = cv_with_dates("2024-09", "2026-02", "2014-02", "2015-99");
        let err = cv.validate().unwrap_err();
        let msg = format!("{err:#}");
        assert!(msg.contains("2015-99"), "should name the bad value: {msg}");
        assert!(msg.contains("end date"), "should name the field: {msg}");
        assert!(
            msg.contains("Acme Tech"),
            "should name which education entry: {msg}"
        );
    }
}
