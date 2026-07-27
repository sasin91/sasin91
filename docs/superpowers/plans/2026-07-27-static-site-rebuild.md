# Static Site Rebuild Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the Laravel + Inertia + React application at sasin91.xyz with a ~620 line Rust binary that renders Djot content into a static site shipping zero JavaScript.

**Architecture:** A single binary walks `content/`, parses `+++ TOML +++` frontmatter, renders Djot bodies to HTML (intercepting code blocks for syntax highlighting and math for MathML), pushes everything through Askama templates that are type-checked at compile time, and writes a fully regenerated `public/`. CI builds the site on Linux and rsyncs `public/` to the FreeBSD box; the binary never runs on the server.

**Tech Stack:** Rust 2024 edition, `jotdown` (Djot), `askama` (templates), `syntect` (highlighting), `latex2mathml` (math), `serde`/`toml`, `chrono`, `walkdir`, `anyhow`.

## Global Constraints

- Design doc: `docs/superpowers/specs/2026-07-27-static-site-rebuild-design.md`. Read it first.
- **This is a fresh repository** at `~/Code/sasin91.xyz`, with no shared history. The old Laravel repo stays at `~/Herd/sasin91.xyz` as an archive and is the source for harvested material. Reach it with `git -C ~/Herd/sasin91.xyz show <ref>:<path>`.
- Working reference implementation is on branch `prototype/ssg-bakeoff` of the OLD repo, under `prototypes/rust/`. Harvest from it; do not assume it is correct — it has two known URL bugs this plan fixes.
- `escape()` lives once, in `src/html.rs`. Neither `djot.rs` nor `math.rs` defines its own copy.
- **Zero JavaScript.** `public/` must contain no `<script>` tags. No KaTeX, no MathJax, no webfonts.
- **These URLs must not break:** `/`, `/blog`, `/blog/trongate`, `/blog/trongate/mx-transition`, `/blog/freebsd-on-hetzner`, `/blog/athletos-freebsd`.
- Post output paths come from an explicit `path` key in frontmatter, never from the filename.
- Syntax themes: `Solarized (light)` for light, `base16-ocean.dark` for dark. Emitted as CSS classes with prefix `hl-`, never inline styles.
- Plain CSS only. No Tailwind, no CSS framework, no build step for styles.
- Homepage is posts-first. The career timeline lives at `/about`, not on `/`.
- `public/` is disposable and fully regenerated every run. Nothing is ever mutated in place.
- Every task ends with a commit.
- **Between Tasks 2 and 7 the modules are not yet wired into `main`, so `cargo clippy -- -D warnings` will fail on `dead_code`.** This is expected. Run `cargo test` for those tasks; clippy becomes meaningful again from Task 7, where `main` uses everything.

---

## File Structure

| File | Responsibility |
|---|---|
| `Cargo.toml` | Dependencies, binary named `site` |
| `src/main.rs` | Build orchestration: load, render, write |
| `src/content.rs` | Frontmatter parsing, the `Post` type, loading from disk |
| `src/djot.rs` | Djot → HTML, intercepting code blocks and math |
| `src/highlight.rs` | syntect wrapper: class-based HTML, dual-theme stylesheet |
| `src/math.rs` | LaTeX → MathML |
| `src/html.rs` | HTML escaping, shared by djot.rs and math.rs |
| `templates/*.html`, `templates/*.xml` | Askama templates |
| `content/cv.toml` | Profile and career timeline |
| `content/blog/*.dj` | One file per post |
| `static/site.css`, `static/images/` | Copied verbatim into `public/` |
| `.github/workflows/deploy.yml` | Build and rsync |

---

### Task 1: Scaffold the new repository

The repo already exists at `~/Code/sasin91.xyz` with the design docs committed. This task adds the Cargo project, the images the posts need, the README, and CI.

**Files:**
- Create: `Cargo.toml`, `src/main.rs`, `.gitignore`, `README.md`, `.github/workflows/pipeline.yml`
- Create: `static/images/` (copied from the old repo)

**Interfaces:**
- Consumes: nothing
- Produces: a `site` binary that compiles and prints a placeholder line

- [ ] **Step 1: Record the URLs that must survive**

These come from the old repo's routes and are the contract this rebuild must honour.

```bash
git -C ~/Herd/sasin91.xyz show main:routes/web.php | grep -oP "Route::get\('\K/blog[^']*"
```

Expected output, exactly:

```
/blog
/blog/trongate
/blog/trongate/mx-transition
/blog/freebsd-on-hetzner
/blog/athletos-freebsd
```

- [ ] **Step 2: Copy the images the posts need out of the old repo**

```bash
mkdir -p static/images
cp -r ~/Herd/sasin91.xyz/resources/images/blog/* static/images/
ls -R static/images
```

Expected: `athletos-freebsd/` (3 svg), `freebsd-on-hetzner/` (1 svg), `trongate/` (4 files).

- [ ] **Step 3: Write `Cargo.toml`**

```toml
[package]
name = "site"
version = "0.1.0"
edition = "2024"

[[bin]]
name = "site"
path = "src/main.rs"

[dependencies]
anyhow = "1"
askama = "0.14"
chrono = { version = "0.4", default-features = false, features = ["std", "clock", "serde"] }
jotdown = "0.10"
latex2mathml = "0.2"
serde = { version = "1", features = ["derive"] }
syntect = { version = "5", default-features = false, features = ["default-fancy"] }
toml = "0.8"
walkdir = "2"

[profile.release]
opt-level = 2
```

Note `syntect` uses `default-fancy`, the pure-Rust regex engine. Do not use `default-onig`; it needs a C toolchain and will fail on a clean CI runner.

- [ ] **Step 4: Write `.gitignore`**

```gitignore
/target
/public
/.superpowers
```

- [ ] **Step 5: Write `README.md`**

````markdown
# sasin91.xyz

A static site: a short CV, and posts about things I built.

Content is [Djot](https://djot.net/) under `content/`. A Rust binary renders it
into `public/`, which is disposable and regenerated on every build. The site
ships no JavaScript.

## Build

```sh
cargo run --release      # writes ./public
```

## Write

```sh
cargo install watchexec-cli               # once
watchexec -e dj,html,css,rs -- cargo run  # rebuild on change
cd public && python -m http.server 8000   # serve, in another shell
```

A post is one `.dj` file under `content/blog/` with a `+++` TOML header. The
`path` key is the URL, and is deliberately not derived from the filename.

## Deploy

Pushing to `main` builds the site in CI and rsyncs it to the FreeBSD box.
See `docs/deploy.md`.

## History

This replaced a Laravel + Inertia + React application. That repo is archived
separately; nothing here shares history with it.
````

- [ ] **Step 6: Write a placeholder `src/main.rs`**

```rust
fn main() -> anyhow::Result<()> {
    println!("site builder");
    Ok(())
}
```

- [ ] **Step 7: Verify it builds and runs**

Run: `cargo run --release`
Expected: compiles, prints `site builder`.

- [ ] **Step 8: Write the CI pipeline**

Create `.github/workflows/pipeline.yml`:

```yaml
name: pipeline

on:
  pull_request:
    branches: [main]
  push:
    branches: [main]
  workflow_dispatch: {}

concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true

permissions:
  contents: read

jobs:
  check:
    name: Check
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v5
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy, rustfmt
      - uses: Swatinem/rust-cache@v2
      - name: Format
        run: cargo fmt --check
      - name: Clippy
        run: cargo clippy -- -D warnings
      - name: Test
        run: cargo test
```

- [ ] **Step 9: Verify formatting and lints pass locally**

Run: `cargo fmt --check && cargo clippy -- -D warnings && cargo test`
Expected: all pass. `cargo test` reports `0 passed` — there are no tests yet.

- [ ] **Step 10: Commit**

```bash
git add -A
git commit -m "feat: scaffold the Rust site builder

Cargo project, CI, and the blog images carried over from the old repo."
```

### Task 2: Content model — frontmatter, explicit paths, loading

The `path` key is the fix for the first known URL bug: `mx-transition` is nested under `trongate/` and must not be derived from a filename.

**Files:**
- Create: `src/content.rs`
- Modify: `src/main.rs`

**Interfaces:**
- Consumes: nothing
- Produces:
  - `pub struct Post { pub path: String, pub title: String, pub date: NaiveDate, pub description: String, pub hero: Option<String>, pub hero_alt: Option<String>, pub body: String }`
  - `impl Post { pub fn url(&self) -> String; pub fn date_long(&self) -> String; pub fn date_iso(&self) -> String; pub fn date_rfc2822(&self) -> String; pub fn alt(&self) -> &str }`
  - `pub fn split_frontmatter(source: &str) -> Result<(FrontMatter, &str)>`
  - `pub struct FrontMatter { pub path: String, pub title: String, pub date: NaiveDate, pub description: String, pub hero: Option<String>, pub hero_alt: Option<String> }`
  - `pub fn load_posts(dir: &Path, render: impl Fn(&str) -> Result<String>) -> Result<Vec<Post>>`

`load_posts` takes the body renderer as a closure so this module has no dependency on Djot and can be tested with a stub.

- [ ] **Step 1: Write the failing tests**

Create `src/content.rs` containing only:

```rust
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
```

- [ ] **Step 2: Run the tests to verify they fail**

Add `mod content;` to `src/main.rs`, then run: `cargo test`
Expected: FAIL — `cannot find function split_frontmatter`, `cannot find struct Post`.

- [ ] **Step 3: Implement the content module**

Insert above the `mod tests` block in `src/content.rs`:

```rust
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
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test`
Expected: 5 passed.

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml src/content.rs src/main.rs
git commit -m "feat: content model with explicit post paths

Post paths come from a frontmatter key rather than the filename, so
/blog/trongate/mx-transition keeps its nesting."
```

---

### Task 3: Syntax highlighting as CSS classes

**Files:**
- Create: `src/highlight.rs`
- Modify: `src/main.rs`

**Interfaces:**
- Consumes: nothing
- Produces:
  - `pub struct Highlighter`
  - `impl Highlighter { pub fn new() -> Self; pub fn to_html(&self, code: &str, lang: &str) -> Result<String>; pub fn stylesheet(&self, light: &str, dark: &str) -> Result<String> }`
  - `to_html` returns `<pre class="hl-code"><code>…</code></pre>`

`hl-code` is the class syntect attaches the theme's own foreground and background to, so the block picks up both palettes without extra CSS.

- [ ] **Step 1: Write the failing tests**

Create `src/highlight.rs` containing only:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emits_classes_not_inline_styles() {
        let hl = Highlighter::new();
        let html = hl.to_html("echo hello\n", "bash").unwrap();
        assert!(html.contains("class=\"hl-"), "expected hl- classes: {html}");
        assert!(!html.contains("style="), "must not inline styles: {html}");
    }

    #[test]
    fn wraps_output_in_a_themed_pre() {
        let hl = Highlighter::new();
        let html = hl.to_html("echo hello\n", "bash").unwrap();
        assert!(html.starts_with("<pre class=\"hl-code\"><code>"));
        assert!(html.ends_with("</code></pre>"));
    }

    #[test]
    fn falls_back_to_plain_text_for_an_unknown_language() {
        let hl = Highlighter::new();
        // Caddyfile has no syntect definition; it must not panic or error.
        let html = hl.to_html("{$APP_DOMAIN} {\n}\n", "caddy").unwrap();
        assert!(html.contains("APP_DOMAIN"));
    }

    #[test]
    fn stylesheet_carries_both_palettes() {
        let hl = Highlighter::new();
        let css = hl
            .stylesheet("Solarized (light)", "base16-ocean.dark")
            .unwrap();
        assert!(css.contains("@media (prefers-color-scheme: dark)"));
        // Solarized light's background, proving the light theme is present.
        assert!(css.to_lowercase().contains("#fdf6e3"));
    }

    #[test]
    fn stylesheet_rejects_an_unknown_theme() {
        let hl = Highlighter::new();
        assert!(hl.stylesheet("No Such Theme", "base16-ocean.dark").is_err());
    }
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Add `mod highlight;` to `src/main.rs`, then run: `cargo test`
Expected: FAIL — `cannot find struct Highlighter`.

- [ ] **Step 3: Implement the highlighter**

Insert above the `mod tests` block in `src/highlight.rs`:

```rust
//! Syntax highlighting emitted as CSS classes rather than inline styles, so
//! both palettes live once in a stylesheet instead of being repeated on every
//! token of every page.

use anyhow::{anyhow, Result};
use syntect::highlighting::ThemeSet;
use syntect::html::{css_for_theme_with_class_style, ClassStyle, ClassedHTMLGenerator};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

const STYLE: ClassStyle = ClassStyle::SpacedPrefixed { prefix: "hl-" };

pub struct Highlighter {
    syntaxes: SyntaxSet,
    themes: ThemeSet,
}

impl Highlighter {
    pub fn new() -> Self {
        Self {
            syntaxes: SyntaxSet::load_defaults_newlines(),
            themes: ThemeSet::load_defaults(),
        }
    }

    pub fn to_html(&self, code: &str, lang: &str) -> Result<String> {
        let syntax = self
            .syntaxes
            .find_syntax_by_token(lang)
            .or_else(|| self.syntaxes.find_syntax_by_extension(lang))
            .unwrap_or_else(|| self.syntaxes.find_syntax_plain_text());

        let mut generator =
            ClassedHTMLGenerator::new_with_class_style(syntax, &self.syntaxes, STYLE);

        for line in LinesWithEndings::from(code) {
            generator.parse_html_for_line_which_includes_newline(line)?;
        }

        Ok(format!(
            "<pre class=\"hl-code\"><code>{}</code></pre>",
            generator.finalize()
        ))
    }

    /// Both palettes, written once at build time into public/syntax.css.
    pub fn stylesheet(&self, light: &str, dark: &str) -> Result<String> {
        let light = self
            .themes
            .themes
            .get(light)
            .ok_or_else(|| anyhow!("unknown light theme: {light}"))?;
        let dark = self
            .themes
            .themes
            .get(dark)
            .ok_or_else(|| anyhow!("unknown dark theme: {dark}"))?;

        Ok(format!(
            "/* generated by the site builder - do not edit */\n\
             :root {{ color-scheme: light dark; }}\n\
             {}\n\
             @media (prefers-color-scheme: dark) {{\n{}\n}}\n",
            css_for_theme_with_class_style(light, STYLE)?,
            css_for_theme_with_class_style(dark, STYLE)?,
        ))
    }
}

impl Default for Highlighter {
    fn default() -> Self {
        Self::new()
    }
}
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test`
Expected: 10 passed.

- [ ] **Step 5: Commit**

```bash
git add src/highlight.rs src/main.rs
git commit -m "feat: class-based syntax highlighting with dual themes"
```

---

### Task 4: HTML escaping

One escaper, used by both the Djot renderer and the math renderer. Defined once so the two cannot drift apart.

**Files:**
- Create: `src/html.rs`
- Modify: `src/main.rs`

**Interfaces:**
- Consumes: nothing
- Produces: `pub fn escape(raw: &str) -> String`

- [ ] **Step 1: Write the failing tests**

Create `src/html.rs` containing only:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes_the_four_dangerous_characters() {
        assert_eq!(escape(r#"<a href="x">&</a>"#), "&lt;a href=&quot;x&quot;&gt;&amp;&lt;/a&gt;");
    }

    #[test]
    fn escapes_ampersands_before_the_entities_it_creates() {
        // A naive implementation that replaces < before & yields "&amp;lt;".
        assert_eq!(escape("&lt;"), "&amp;lt;");
    }

    #[test]
    fn leaves_ordinary_text_alone() {
        assert_eq!(escape("4-remaster.sh"), "4-remaster.sh");
    }
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Add `mod html;` to `src/main.rs`, then run: `cargo test`
Expected: FAIL — `cannot find function escape`.

- [ ] **Step 3: Implement the escaper**

Insert above the `mod tests` block in `src/html.rs`:

```rust
//! HTML escaping, shared by the Djot and math renderers so the two cannot
//! drift apart.

/// Escape text for insertion into HTML, including inside a double-quoted
/// attribute value.
///
/// Ampersand is replaced first, so the entities produced by the later
/// replacements are not themselves re-escaped.
pub fn escape(raw: &str) -> String {
    raw.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test`
Expected: 13 passed.

- [ ] **Step 5: Commit**

```bash
git add src/html.rs src/main.rs
git commit -m "feat: shared HTML escaper"
```

---

### Task 5: LaTeX to MathML

**Files:**
- Create: `src/math.rs`
- Modify: `src/main.rs`

**Interfaces:**
- Consumes: nothing
- Produces: `pub fn to_mathml(tex: &str, display: bool) -> String`

Never returns an error. A formula that fails to parse renders visibly as `<code class="math-error">` so a mistake is obvious on the page rather than silently missing.

- [ ] **Step 1: Write the failing tests**

Create `src/math.rs` containing only:

```rust
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
        let html = to_mathml("<script>alert(1)</script>", false);
        assert!(!html.contains("<script>"), "got: {html}");
    }
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Add `mod math;` to `src/main.rs`, then run: `cargo test`
Expected: FAIL — `cannot find function to_mathml`.

- [ ] **Step 3: Implement the math module**

Insert above the `mod tests` block in `src/math.rs`:

```rust
//! LaTeX to MathML at build time. Browsers render MathML natively, so a
//! formula costs the reader no JavaScript.

use latex2mathml::{latex_to_mathml, DisplayStyle};

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

```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test`
Expected: 17 passed.

If `renders_broken_math_visibly_instead_of_dropping_it` fails because `latex2mathml` accepted `\frac{`, replace the input with `\frac` alone and re-run. Do not delete the test — some malformed input must exercise the error path.

- [ ] **Step 5: Commit**

```bash
git add src/math.rs src/main.rs
git commit -m "feat: render LaTeX to MathML at build time"
```

---

### Task 6: Djot rendering

Intercepts two event types off jotdown's stream. Everything else — divs, attributes, links, footnotes, raw HTML — is jotdown's own renderer.

**Files:**
- Create: `src/djot.rs`
- Modify: `src/main.rs`

**Interfaces:**
- Consumes: `highlight::Highlighter::to_html`, `math::to_mathml`
- Produces: `pub fn render(source: &str, hl: &Highlighter) -> Result<String>`

- [ ] **Step 1: Write the failing tests**

Create `src/djot.rs` containing only:

```rust
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
```

- [ ] **Step 2: Run the tests to verify they fail**

Add `mod djot;` to `src/main.rs`, then run: `cargo test`
Expected: FAIL — `cannot find function render`.

- [ ] **Step 3: Implement the Djot renderer**

Insert above the `mod tests` block in `src/djot.rs`:

```rust
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

```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test`
Expected: 23 passed.

- [ ] **Step 5: Commit**

```bash
git add src/djot.rs src/main.rs
git commit -m "feat: render Djot, with highlighted code blocks and inline MathML"
```

---

### Task 7: Templates and page generation

**Files:**
- Create: `templates/base.html`, `templates/index.html`, `templates/about.html`, `templates/blog.html`, `templates/post.html`
- Create: `content/cv.toml`
- Modify: `src/main.rs`

**Interfaces:**
- Consumes: `content::{Post, load_posts}`, `djot::render`, `highlight::Highlighter`
- Produces:
  - `pub struct Cv { pub site: Profile, pub timeline: Vec<Job> }`
  - `pub struct Profile { pub name: String, pub title: String, pub stack: String, pub since: String, pub links: Links }`
  - `pub struct Links { pub github: String, pub linkedin: String, pub email: String }`
  - `pub struct Job { pub date: String, pub name: String, pub description: String }` with `pub fn month(&self) -> String`
  - `public/index.html`, `public/about/index.html`, `public/blog/index.html`, `public/<post.path>/index.html`

Askama checks every template against these structs at compile time. A typo in `{{ post.titel }}` is a build error.

- [ ] **Step 1: Write `content/cv.toml`**

```toml
[site]
name = "Jonas Hansen"
title = "Full-Stack Developer"
stack = "PHP · Laravel · React · TypeScript · MySQL · Docker"
since = "10+ years of experience (since 2015)"

[site.links]
github = "https://github.com/sasin91"
linkedin = "https://www.linkedin.com/in/jonas-hansen-2b6828110"
email = "mailto:jonas.kerwin.hansen@gmail.com"

[[timeline]]
date = "2024-09"
name = "Web developer at JUICE"
description = "Like a boomerang, I'm back at Juice again."

[[timeline]]
date = "2023-09"
name = "Developer at Supeo"
description = "Primarily Laravel and React, across several different domains in a changeable everyday life."

[[timeline]]
date = "2023-01"
name = "Web developer at JUICE"
description = "Helped develop a platform that turns the job market upside down."

[[timeline]]
date = "2020-02"
name = "Developer at Syncronet"
description = "Helped develop their video streaming platform. In broad strokes, I took most of it."

[[timeline]]
date = "2017-02"
name = "Web developer at GHC Travel"
description = "Responsible for migrating and modernizing their existing website from pure PHP to Laravel 6."

[[timeline]]
date = "2015-08"
name = "Trained WebIntegrator"
description = "Passed my training as a WebIntegrator with a 12."
```

- [ ] **Step 2: Write `templates/base.html`**

```html
<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>{% block title %}{{ cv.site.name }}{% endblock %}</title>
    {% block description %}{% endblock %}
    <link rel="stylesheet" href="/site.css" />
    <link rel="stylesheet" href="/syntax.css" />
    <link rel="alternate" type="application/rss+xml" title="{{ cv.site.name }}" href="/rss.xml" />
  </head>
  <body>
    <div class="wrap">
      <header class="site-head">
        <a class="brand" href="/">{{ cv.site.name }}</a>
        <nav class="site-nav">
          <a href="/blog/">Writing</a>
          <a href="/about/">About</a>
          <a href="{{ cv.site.links.github }}">GitHub</a>
        </nav>
      </header>

      <main>
        {% block content %}{% endblock %}
      </main>

      <footer class="site-foot">
        <span>&copy; {{ year }} {{ cv.site.name }}</span>
        <a href="/rss.xml">RSS</a>
      </footer>
    </div>
  </body>
</html>
```

- [ ] **Step 3: Write `templates/index.html`** — posts-first, no timeline

```html
{% extends "base.html" %}

{% block title %}{{ cv.site.name }} — {{ cv.site.title }}{% endblock %}
{% block description %}<meta name="description" content="{{ cv.site.stack }}" />{% endblock %}

{% block content %}
  <section class="intro">
    <h1>{{ cv.site.name }}</h1>
    <p class="lede">{{ cv.site.title }}</p>
    <p class="stack">{{ cv.site.stack }}</p>

    <div class="social">
      <a href="{{ cv.site.links.github }}">GitHub</a>
      <a href="{{ cv.site.links.linkedin }}">LinkedIn</a>
      <a href="{{ cv.site.links.email }}">Email</a>
    </div>
  </section>

  <h2>Writing</h2>
  <ul class="post-list">
    {% for post in posts %}
      <li>
        <div class="meta">
          <time datetime="{{ post.date_iso() }}">{{ post.date_long() }}</time>
        </div>
        <h3><a href="{{ post.url() }}">{{ post.title }}</a></h3>
        <p>{{ post.description }}</p>
      </li>
    {% endfor %}
  </ul>
{% endblock %}
```

- [ ] **Step 4: Write `templates/about.html`** — the timeline lives here now

```html
{% extends "base.html" %}

{% block title %}About — {{ cv.site.name }}{% endblock %}

{% block content %}
  <h1>About</h1>
  <p class="lede">{{ cv.site.title }}. {{ cv.site.since }}.</p>
  <p class="stack">{{ cv.site.stack }}</p>

  <h2>Experience</h2>
  <ul class="timeline">
    {% for job in cv.timeline %}
      <li>
        <div class="meta">{{ job.month() }}</div>
        <div class="role">{{ job.name }}</div>
        <p>{{ job.description }}</p>
      </li>
    {% endfor %}
  </ul>
{% endblock %}
```

- [ ] **Step 5: Write `templates/blog.html`**

```html
{% extends "base.html" %}

{% block title %}Writing — {{ cv.site.name }}{% endblock %}

{% block content %}
  <h1>Writing</h1>
  <p class="lede">Notes on things I built and what broke on the way.</p>

  <ul class="post-list">
    {% for post in posts %}
      <li>
        <div class="meta">
          <time datetime="{{ post.date_iso() }}">{{ post.date_long() }}</time>
        </div>
        <h2><a href="{{ post.url() }}">{{ post.title }}</a></h2>
        <p>{{ post.description }}</p>
      </li>
    {% endfor %}
  </ul>
{% endblock %}
```

- [ ] **Step 6: Write `templates/post.html`**

```html
{% extends "base.html" %}

{% block title %}{{ post.title }}{% endblock %}
{% block description %}<meta name="description" content="{{ post.description }}" />{% endblock %}

{% block content %}
  <article class="prose">
    {% if let Some(hero) = post.hero %}
      <img src="{{ hero }}" alt="{{ post.alt() }}" />
    {% endif %}

    <h1>{{ post.title }}</h1>
    <div class="meta">
      <time datetime="{{ post.date_iso() }}">{{ post.date_long() }}</time>
    </div>

    {{ post.body|safe }}
  </article>
{% endblock %}
```

- [ ] **Step 7: Write `src/main.rs`**

```rust
//! sasin91.xyz - static site builder.
//!
//! Walks content/, renders through Askama templates, writes ./public.
//! Templates are type-checked against these structs at compile time: a typo
//! in `{{ post.titel }}` is a build error, not a blank space on the page.

mod content;
mod djot;
mod highlight;
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

fn write(path: impl AsRef<Path>, contents: &str) -> Result<()> {
    let path = path.as_ref();
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
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
            fs::create_dir_all(dir)?;
        }
        fs::copy(entry.path(), &dest)?;
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
        fs::remove_dir_all(OUT)?;
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

    println!(
        "built {} posts in {:.0?} -> {OUT}/",
        posts.len(),
        started.elapsed()
    );

    Ok(())
}
```

- [ ] **Step 8: Prove the compile-time check is real**

Temporarily break a template, confirm the build refuses, then restore it:

```bash
cp templates/post.html templates/post.html.bak
sed -i 's/{{ post.title }}/{{ post.titel }}/' templates/post.html
cargo build 2>&1 | grep -E '^error' | head -2
mv templates/post.html.bak templates/post.html
```

Expected: `error[E0609]: no field 'titel' on type '&Post'`. If the build succeeds instead, Askama is not checking the template and the task is not done.

- [ ] **Step 9: Verify a clean build and run**

Run: `cargo build --release && ./target/release/site`
Expected: compiles, prints `built 0 posts in …` — there is no content yet, which is correct.

- [ ] **Step 10: Commit**

```bash
git add templates content/cv.toml src/main.rs
git commit -m "feat: page templates, checked against their structs at compile time

Homepage is posts-first; the career timeline moves to /about."
```

---

### Task 8: Feeds — RSS, sitemap, robots

**Files:**
- Create: `templates/rss.xml`, `templates/sitemap.xml`
- Modify: `src/main.rs`

**Interfaces:**
- Consumes: `content::Post`, `Cv`
- Produces: `public/rss.xml`, `public/sitemap.xml`, `public/robots.txt`

- [ ] **Step 1: Write `templates/rss.xml`**

```xml
<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <title>{{ cv.site.name }}</title>
    <link>{{ base }}</link>
    <description>{{ cv.site.title }}</description>
{%- for post in posts %}
    <item>
      <title>{{ post.title }}</title>
      <link>{{ base }}{{ post.url() }}</link>
      <guid>{{ base }}{{ post.url() }}</guid>
      <pubDate>{{ post.date_rfc2822() }}</pubDate>
      <description>{{ post.description }}</description>
    </item>
{%- endfor %}
  </channel>
</rss>
```

- [ ] **Step 2: Write `templates/sitemap.xml`**

```xml
<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
  <url><loc>{{ base }}/</loc></url>
  <url><loc>{{ base }}/about/</loc></url>
  <url><loc>{{ base }}/blog/</loc></url>
{%- for post in posts %}
  <url>
    <loc>{{ base }}{{ post.url() }}</loc>
    <lastmod>{{ post.date_iso() }}</lastmod>
  </url>
{%- endfor %}
</urlset>
```

- [ ] **Step 3: Add the feed structs and writes to `src/main.rs`**

Add after the `PostPage` struct:

```rust
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
```

Add before the final `println!` in `main`:

```rust
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
```

- [ ] **Step 4: Verify the feeds are generated and well-formed**

```bash
cargo run --release
python -c "import xml.dom.minidom as m; m.parse('public/rss.xml'); m.parse('public/sitemap.xml'); print('both parse')"
cat public/robots.txt
```

Expected: `both parse`, and robots.txt names the sitemap.

- [ ] **Step 5: Commit**

```bash
git add templates/rss.xml templates/sitemap.xml src/main.rs
git commit -m "feat: generate RSS, sitemap and robots.txt"
```

---

### Task 9: The stylesheet

Plain CSS styling semantic elements, with a small number of classes. Harvest from `prototypes/_shared/site.css` on the prototype branch, which already carries the fix for the `<pre>` padding bug.

**Files:**
- Create: `static/site.css`

**Interfaces:**
- Consumes: the class names produced by Tasks 5 and 6 — `.wrap`, `.site-head`, `.brand`, `.site-nav`, `.site-foot`, `.intro`, `.lede`, `.stack`, `.social`, `.meta`, `.post-list`, `.timeline`, `.role`, `.prose`, `.codeblock`, `.filename`, `.callout`, `.warning`, `.note`, `.math-error`
- Produces: `public/site.css`

- [ ] **Step 1: Harvest the stylesheet from the prototype**

```bash
git -C ~/Herd/sasin91.xyz show prototype/ssg-bakeoff:prototypes/_shared/site.css > static/site.css
wc -l static/site.css
```

Expected: roughly 300 lines.

- [ ] **Step 2: Adapt the callout selectors to Djot's output**

Djot emits `<div class="warning">`, not `<aside class="callout">`. Append to `static/site.css`:

```css
/* ---- Djot output ----
   `::: warning` renders as <div class="warning">, and the heading inside it
   is the callout's title. */

.prose .warning,
.prose .note {
  margin: 1.8rem 0;
  padding: 1rem 1.15rem;
  border-left: 2px solid var(--warn-edge);
  background: var(--warn-bg);
  border-radius: 0 5px 5px 0;
  font-size: 0.95rem;
}

.prose .note {
  border-left-color: var(--accent);
  background: var(--surface);
}

.prose .warning > h2,
.prose .note > h2 {
  margin: 0 0 0.35rem;
  font-size: 1rem;
  font-weight: 620;
}

.prose .warning > :last-child,
.prose .note > :last-child {
  margin-bottom: 0;
}

/* syntect writes both palettes into syntax.css keyed on .hl-code, so the
   frame must not impose its own background. Only the filename bar tracks
   the theme. */

.codeblock {
  background: none;
}

.codeblock .filename {
  color: #93a1a1;
  background: #eee8d5;
  border-bottom: 1px solid rgb(0 0 0 / 0.08);
}

@media (prefers-color-scheme: dark) {
  .codeblock .filename {
    color: #a7adba;
    background: #343d46;
    border-bottom-color: rgb(255 255 255 / 0.06);
  }
}

.math-error {
  background: var(--warn-bg);
  border-bottom: 1px dotted var(--warn-edge);
}

math {
  font-size: 1.05em;
}
```

- [ ] **Step 3: Verify the CSS reaches the output**

```bash
cargo run --release
test -f public/site.css && echo "site.css copied"
grep -c 'prefers-color-scheme' public/site.css
```

Expected: `site.css copied`, and at least 2 matches.

- [ ] **Step 4: Commit**

```bash
git add static/site.css
git commit -m "feat: stylesheet, adapted to Djot's callout markup"
```

---

### Task 10: Convert the four existing posts to Djot

Djot inverts Markdown's emphasis, so this is not a rename. Each post must be verified by reading the rendered output, not assumed correct.

**Files:**
- Create: `content/blog/trongate.dj`, `content/blog/mx-transition.dj`, `content/blog/freebsd-on-hetzner.dj`, `content/blog/athletos-freebsd.dj`

**Interfaces:**
- Consumes: `content::split_frontmatter` — every file needs `path`, `title`, `date`, `description`
- Produces: four posts at the URLs listed in Global Constraints

- [ ] **Step 1: Recover the original post sources**

```bash
mkdir -p /tmp/oldposts
for p in trongate mx-transition freebsd-on-hetzner athletos-freebsd; do
  git -C ~/Herd/sasin91.xyz show prototype/ssg-bakeoff:resources/js/pages/blog/$p.tsx > /tmp/oldposts/$p.tsx 2>/dev/null \
    || git show main:resources/js/pages/blog/$p.tsx > /tmp/oldposts/$p.tsx
done
wc -l /tmp/oldposts/*.tsx
```

- [ ] **Step 2: Harvest the two posts already converted on the prototype branch**

```bash
git -C ~/Herd/sasin91.xyz show prototype/ssg-bakeoff:prototypes/rust/content/blog/freebsd-on-hetzner.dj \
  > content/blog/freebsd-on-hetzner.dj
git -C ~/Herd/sasin91.xyz show prototype/ssg-bakeoff:prototypes/rust/content/blog/athletos-freebsd.dj \
  > content/blog/athletos-freebsd.dj
```

- [ ] **Step 3: Add the `path` key to both, and delete the invented math section**

Every harvested file's frontmatter lacks `path` — the prototype derived it from the filename. Add it as the first key:

```toml
# content/blog/freebsd-on-hetzner.dj
path = "blog/freebsd-on-hetzner"

# content/blog/athletos-freebsd.dj
path = "blog/athletos-freebsd"
```

Then delete the section titled `## What the box actually computes` from `athletos-freebsd.dj`, including the two formulas and the paragraph beginning "For a set of five at 100 kg". That prose was written to demonstrate math rendering and is not Jonas's writing. Ask before substituting anything in its place.

- [ ] **Step 4: Convert `trongate.tsx` and `mx-transition.tsx` by hand**

These two were never ported. Read `/tmp/oldposts/trongate.tsx` and `/tmp/oldposts/mx-transition.tsx` and transcribe the prose into Djot. Rules:

- `**bold**` becomes `*bold*`, and `*italic*` becomes `_italic_` — Djot inverts these
- `<CodeBlock language="x" filename="y" code={...} />` becomes an attribute line then a fence:

  ```
  {title="y"}
  ```x
  ...code...
  ```
  ```
- `<img src={someImg} alt="..." />` becomes `![alt](/images/trongate/<file>)`
- `<BlogLink href="/blog/...">text</BlogLink>` becomes `[text](/blog/.../)`
- `<Underline active={true}>word</Underline>` becomes `*word*` (Djot strong)

**Watch for computed values.** `trongate.tsx` contains:

```tsx
const currentYear = new Date().getFullYear();
const phpExpYears = currentYear - 2014;
...
I have worked with PHP for {phpExpYears} years, and one recurring thing is
```

That number recalculates on every render today. In a static file it freezes.
Write the literal for the current year and accept that it ages, or reword the
sentence to something that does not — "since 2014" rather than "for 12 years".
Prefer the reword. Do not silently bake in a number that will quietly become
wrong.

Worked example. This fragment of `trongate.tsx`:

```tsx
<h2 className="text-xl font-bold tracking-tight text-primary">
  I have worked with PHP for {phpExpYears} years, and one recurring thing is{' '}
  <Underline active={true}>unnecessary</Underline> code that needs maintenance.
</h2>

<figure className="mt-4 border-l border-indigo-600 pl-9">
  <blockquote className="font-semibold text-primary-900">
    <p>
      This is where Trongate shines; you get a simple starting point with a
      solid starting architecture due to everything being divided into modules.
    </p>
  </blockquote>
</figure>
```

becomes:

```djot
## I have worked with PHP since 2014, and one recurring thing is *unnecessary* code that needs maintenance.

> This is where Trongate shines; you get a simple starting point with a solid
> starting architecture due to everything being divided into modules.
```

Frontmatter for each, copied exactly — these descriptions come from the current blog index:

```toml
+++
path = "blog/trongate"
title = "Trongate PHP"
date = 2024-09-14
description = "Trongate is often misunderstood and gets a bad reputation because it breaks with common standards and takes a journey back to its roots. In this article, I will explore and highlight this rough diamond that deserves a spot in the limelight."
+++
```

```toml
+++
path = "blog/trongate/mx-transition"
title = "Trongate mx-transition attribute"
date = 2025-03-03
description = "MX transition provides an easy and intuitive way to add animations to items and the whole page."
+++
```

- [ ] **Step 5: Build and verify every required URL exists**

```bash
cargo run --release
for u in index about/index blog/index blog/trongate/index \
         blog/trongate/mx-transition/index blog/freebsd-on-hetzner/index \
         blog/athletos-freebsd/index; do
  test -f "public/$u.html" && echo "OK   $u" || echo "MISS $u"
done
```

Expected: seven `OK` lines, no `MISS`. A `MISS` on `blog/trongate/mx-transition` means the `path` key was not honoured.

- [ ] **Step 6: Verify no JavaScript and no broken math**

```bash
echo "script tags: $(grep -ro '<script' public --include=*.html | wc -l)"
echo "math errors: $(grep -ro 'math-error' public --include=*.html | wc -l)"
```

Expected: both `0`.

- [ ] **Step 7: Read the rendered posts and compare against the live site**

Serve locally and open each post beside the current sasin91.xyz:

```bash
cargo run --release && (cd public && python -m http.server 8000)
```

Check specifically: emphasis is not doubled or missing (the `*`/`_` inversion), every code block kept its filename, every image resolves, and internal links work. This step is a human read, not a grep.

- [ ] **Step 8: Commit**

```bash
git add content/blog
git commit -m "feat: convert the four existing posts to Djot

Emphasis is inverted in Djot; each post was read against the live site
rather than converted mechanically."
```

---

### Task 11: Deploy — CI build and rsync to the FreeBSD box

The binary never runs on the server. CI builds the site on Linux and copies `public/`.

**Files:**
- Create: `.github/workflows/deploy.yml`
- Create: `docs/deploy.md`

**Interfaces:**
- Consumes: `cargo run --release` producing `public/`
- Produces: the deployed site, and a Caddy site block for sasin91.xyz

- [ ] **Step 1: Write `.github/workflows/deploy.yml`**

```yaml
name: deploy

on:
  push:
    branches: [main]
  workflow_dispatch: {}

concurrency:
  group: deploy
  cancel-in-progress: false

permissions:
  contents: read

jobs:
  deploy:
    name: Build and deploy
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v5
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2

      - name: Build the site
        run: cargo run --release

      - name: Fail if any JavaScript slipped in
        run: |
          count=$(grep -ro '<script' public --include=*.html | wc -l)
          echo "script tags: $count"
          test "$count" -eq 0

      - name: Fail if a required URL is missing
        run: |
          for u in index about/index blog/index blog/trongate/index \
                   blog/trongate/mx-transition/index \
                   blog/freebsd-on-hetzner/index blog/athletos-freebsd/index; do
            test -f "public/$u.html" || { echo "missing: $u"; exit 1; }
          done
          echo "all required URLs present"

      - name: Add the host key
        run: |
          mkdir -p ~/.ssh
          echo "${{ secrets.DEPLOY_KNOWN_HOSTS }}" > ~/.ssh/known_hosts
          echo "${{ secrets.DEPLOY_SSH_KEY }}" > ~/.ssh/id_ed25519
          chmod 600 ~/.ssh/id_ed25519

      - name: Rsync to the box
        run: |
          rsync -az --delete public/ \
            "${{ secrets.DEPLOY_USER }}@${{ secrets.DEPLOY_HOST }}:/usr/local/www/sasin91.xyz/"
```

Three repository secrets are required: `DEPLOY_SSH_KEY` (a private key whose public half is in the deploy user's `authorized_keys`), `DEPLOY_HOST`, `DEPLOY_USER`, and `DEPLOY_KNOWN_HOSTS` (output of `ssh-keyscan <host>`).

- [ ] **Step 2: Write `docs/deploy.md` with the Caddy block**

````markdown
# Deploying sasin91.xyz

CI builds the site on Linux and rsyncs `public/` to
`/usr/local/www/sasin91.xyz/` on the FreeBSD box that also runs athletos.app.
The site builder binary never runs on the server.

## Caddy

Add alongside the existing athletos.app block in `/usr/local/etc/caddy/Caddyfile`:

```caddy
sasin91.xyz, www.sasin91.xyz {
	encode zstd gzip
	root * /usr/local/www/sasin91.xyz

	# The old Laravel routes had no trailing slash, and the generator writes
	# <path>/index.html. This resolves /blog/trongate without a redirect.
	try_files {path} {path}/ {path}/index.html
	file_server
}
```

Reload without dropping connections:

```sh
caddy reload --config /usr/local/etc/caddy/Caddyfile
```

## One-time setup

```sh
mkdir -p /usr/local/www/sasin91.xyz
chown deploy:deploy /usr/local/www/sasin91.xyz
```
````

- [ ] **Step 3: Verify the workflow's guards catch a regression**

Prove the URL check works before trusting it:

```bash
cargo run --release
mv public/blog/trongate/mx-transition/index.html /tmp/held.html
for u in blog/trongate/mx-transition/index; do
  test -f "public/$u.html" || echo "correctly detected missing: $u"
done
mv /tmp/held.html public/blog/trongate/mx-transition/index.html
```

Expected: `correctly detected missing: blog/trongate/mx-transition/index`.

- [ ] **Step 4: Commit**

```bash
git add .github/workflows/deploy.yml docs/deploy.md
git commit -m "feat: build in CI and rsync to the FreeBSD box

Guards the deploy on zero script tags and on every pre-existing URL
still resolving."
```

- [ ] **Step 5: Deploy, then verify against the real site**

After merging and letting the workflow run, check the live URLs — trailing-slash resolution is Caddy's behaviour and cannot be verified from `public/` alone:

```bash
for u in / /about/ /blog /blog/trongate /blog/trongate/mx-transition \
         /blog/freebsd-on-hetzner /blog/athletos-freebsd /rss.xml; do
  printf "%-40s %s\n" "$u" "$(curl -s -o /dev/null -w '%{http_code}' https://sasin91.xyz$u)"
done
```

Expected: `200` for every line. A `404` on `/blog/trongate` means the `try_files` directive is wrong or missing.

---

## Local development

```bash
cargo install watchexec-cli          # once
watchexec -e dj,html,css,rs -- cargo run
```

Serve the output in another shell:

```bash
cd public && python -m http.server 8000
```

## Definition of done

- `cargo fmt --check`, `cargo clippy -- -D warnings` and `cargo test` all pass
- `cargo run --release` builds all four posts
- Zero `<script>` tags and zero `math-error` occurrences in `public/`
- All seven required URLs return 200 on the deployed site
- RSS and sitemap parse as valid XML
- Each converted post has been read against the live site for emphasis, code block filenames, images and internal links
