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
    let date = raw.date.ok_or_else(|| D::Error::custom("expected a date"))?;

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
}

#[derive(Debug)]
pub struct Post {
    pub path: String,
    pub title: String,
    pub date: NaiveDate,
    pub description: String,
    pub hero: Option<String>,
    pub hero_alt: Option<String>,
    /// Rendered HTML, not source.
    pub body: String,
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

    for entry in WalkDir::new(dir).into_iter().filter_map(Result::ok) {
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
            body: render(body).with_context(|| format!("rendering {}", file.display()))?,
        });
    }

    posts.sort_by(|a, b| b.date.cmp(&a.date));
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
            body: String::new(),
        };
        // The bug this guards: deriving the slug from a filename would
        // flatten this to /blog/mx-transition and break a live URL.
        assert_eq!(post.url(), "/blog/trongate/mx-transition/");
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
            body: String::new(),
        };
        assert_eq!(post.date_iso(), "2025-03-03");
        assert_eq!(post.date_long(), "March 3, 2025");
    }
}
