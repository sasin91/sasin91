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

use anyhow::{Context, Result};
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
    pub timeline: Vec<Job>,
}

#[derive(Deserialize)]
pub struct Profile {
    pub name: String,
    pub title: String,
    pub stack: String,
    pub since: String,
    pub links: Links,
}

#[derive(Deserialize)]
pub struct Links {
    pub github: String,
    pub linkedin: String,
    pub email: String,
}

#[derive(Deserialize)]
pub struct Job {
    pub date: String,
    pub name: String,
    pub description: String,
}

impl Job {
    /// "2024-09" -> "September 2024"
    pub fn month(&self) -> String {
        NaiveDate::parse_from_str(&format!("{}-01", self.date), "%Y-%m-%d")
            .map(|d| d.format("%B %Y").to_string())
            .unwrap_or_else(|_| self.date.clone())
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
    for entry in WalkDir::new("static").into_iter().filter_map(Result::ok) {
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
    }
    Ok(())
}

fn main() -> Result<()> {
    let started = std::time::Instant::now();

    let hl = highlight::Highlighter::new();
    let cv: Cv = toml::from_str(&fs::read_to_string("content/cv.toml")?)
        .context("parsing content/cv.toml")?;
    let posts = content::load_posts(Path::new("content/blog"), |body| djot::render(body, &hl))?;
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
        &IndexPage { cv: &cv, posts: &posts, year }.render()?,
    )?;
    write(
        format!("{OUT}/about/index.html"),
        &AboutPage { cv: &cv, year }.render()?,
    )?;
    write(
        format!("{OUT}/blog/index.html"),
        &BlogPage { cv: &cv, posts: &posts, year }.render()?,
    )?;

    for post in &posts {
        write(
            format!("{OUT}/{}/index.html", post.path),
            &PostPage { cv: &cv, post, year }.render()?,
        )?;
    }

    write(
        format!("{OUT}/rss.xml"),
        &Feed { cv: &cv, posts: &posts, base: BASE_URL }.render()?,
    )?;
    write(
        format!("{OUT}/sitemap.xml"),
        &Sitemap { posts: &posts, base: BASE_URL }.render()?,
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
