//! sasin91.xyz - static site builder.
//!
//! Walks content/, renders through Askama templates, writes ./public.
//! Templates are type-checked against these structs at compile time: a typo
//! in `{{ post.titel }}` is a build error, not a blank space on the page.

mod content;
mod cv;
mod cv_pdf;
mod djot;
mod embed;
mod highlight;
mod html;
mod knowledge;
mod math;
mod pdf;
mod pdf_metrics;
mod search_index;
mod serve;

use anyhow::{Context, Result};
use askama::Template;
use chrono::Datelike;
use content::Post;
use cv::Cv;
use knowledge::Article;
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
    /// `og:image`, absolute under `BASE_URL` for the same reason `url` is --
    /// see that field's doc comment. `None` for every page with no suitable
    /// asset (the landing, about, CV and blog-listing pages always; a post
    /// with no `card` whose hero is missing or is an SVG -- see `og_image`).
    image: Option<String>,
    /// `og:image:alt`, carried alongside `image` rather than invented as a
    /// generic fallback string. Only meaningful when `image` is `Some`;
    /// `base.html` renders it only in that case.
    image_alt: Option<String>,
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

/// The long-form article at /ai/: every knowledge node, in reading order.
#[derive(Template)]
#[template(path = "ai.html")]
struct AiPage<'a> {
    cv: &'a Cv,
    article: &'a Article,
    year: i32,
    nav: &'static str,
    /// Which of Read / Timeline / Ask the view switcher marks current.
    view: &'static str,
    meta: Meta,
    syntax: bool,
    site_css: &'a str,
    syntax_css: &'a str,
}

/// /ai/timeline/: the same nodes, one line each.
#[derive(Template)]
#[template(path = "ai_timeline.html")]
struct AiTimelinePage<'a> {
    cv: &'a Cv,
    article: &'a Article,
    year: i32,
    nav: &'static str,
    view: &'static str,
    meta: Meta,
    syntax: bool,
    site_css: &'a str,
    syntax_css: &'a str,
}

/// /ai/ask/: retrieval over the nodes, run in the reader's browser.
#[derive(Template)]
#[template(path = "ai_ask.html")]
struct AiAskPage<'a> {
    cv: &'a Cv,
    article: &'a Article,
    year: i32,
    nav: &'static str,
    view: &'static str,
    meta: Meta,
    syntax: bool,
    site_css: &'a str,
    syntax_css: &'a str,
    /// Root-relative URL of the hashed search index.
    index_url: &'a str,
    model: &'a search_index::ModelInfo,
    script: &'static str,
}

/// The Ask page's only script: the retrieval logic followed by the code that
/// drives the form, inlined as one module like every other script on the
/// site. Nothing heavy is in it -- the model and index are fetched by this
/// script only once the reader starts using the form.
const ASK_SCRIPT: &str = concat!(
    include_str!("../js/retrieval.mjs"),
    "\n",
    include_str!("../js/ask.mjs")
);

/// The site-wide search palette (Ctrl+K on any page): the same retrieval
/// logic, plus the palette UI. Not inlined -- base.html imports it from
/// /search/palette.js only when a reader first opens the palette, so no page
/// pays for it otherwise. `build_search` prefixes the hashed index URL.
const PALETTE_SCRIPT: &str = concat!(
    include_str!("../js/retrieval.mjs"),
    "
",
    include_str!("../js/palette.mjs")
);

/// Where the committed embedding model lives; see models/README.md.
const AI_MODEL_DIR: &str = "models/potion-base-4M";
const AI_MODEL_NAME: &str = "potion-base-4M";

/// The search assets every search UI shares.
struct SearchAssets {
    /// Root-relative URL of the hashed site-wide index.
    index_url: String,
    model: search_index::ModelInfo,
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
    article: &'a Article,
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
    hashed_name(stem, "css", bytes)
}

/// The same content-hashed naming for any other immutable asset -- the Ask
/// view's model, vocabulary and index. docs/deploy.md extends the Caddy
/// immutable rule to these extensions.
fn hashed_name(stem: &str, extension: &str, bytes: &[u8]) -> String {
    format!("{stem}.{}.{extension}", hash_css(bytes))
}

fn write_bytes(path: impl AsRef<Path>, contents: &[u8]) -> Result<()> {
    let path = path.as_ref();
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).with_context(|| format!("creating directory {}", dir.display()))?;
    }
    fs::write(path, contents).with_context(|| format!("writing {}", path.display()))
}

fn write(path: impl AsRef<Path>, contents: &str) -> Result<()> {
    write_bytes(path, contents.as_bytes())
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

/// Extensions a post's hero must end in for it to become `Meta::image`.
/// SVG is deliberately excluded, even though it is most posts' hero format
/// today (`athletos-freebsd`, `freebsd-on-hetzner`): Facebook, LinkedIn and
/// Twitter/X all refuse to render an SVG as a link-share image and fall back
/// to no image at all. Emitting `og:image` for an SVG hero would look
/// handled -- the tag is there, the build is green -- while every crawler
/// silently drops it, which is worse than omitting the tag, because nothing
/// about the build would ever point at the gap.
const OG_IMAGE_EXTENSIONS: [&str; 4] = [".png", ".jpg", ".jpeg", ".webp"];

fn is_raster(path: &str) -> bool {
    OG_IMAGE_EXTENSIONS.iter().any(|ext| path.ends_with(ext))
}

/// A post's `og:image` and its alt text, in this order:
///
/// 1. `card`, if the post declares one;
/// 2. otherwise `hero`, but only when it is a raster image
///    (see `OG_IMAGE_EXTENSIONS`);
/// 3. otherwise nothing.
///
/// `card` exists as a field of its own because a post's hero and its
/// link-share image are not the same picture and cannot always be the same
/// file — see [`content::FrontMatter::card`]. `freebsd-on-hetzner` is the
/// concrete case: its hero is an SVG, which is right for the page (inlined,
/// so it follows the theme toggle) and invisible to every crawler, all of
/// which drop an SVG `og:image` and render the link with no image at all.
/// Collapsing the two fields into one would force a choice between a
/// theme-aware page and a shareable link.
///
/// The image, when present, is made absolute under `BASE_URL` for the same
/// reason `Meta::url` must be (a relative `og:image` is silently ignored by
/// every crawler that reads it).
///
/// Fails when `card` points at an SVG. That is the exact bug this field was
/// added to fix, re-entered by hand, and it is invisible in the output: the
/// tag would be present, the build green, the link-share still blank. The
/// check lives here rather than in a separate validator so it cannot be
/// skipped -- this is the only place a `card` is ever read.
fn og_image(post: &Post) -> Result<(Option<String>, Option<String>)> {
    if let Some(card) = &post.card {
        anyhow::ensure!(
            is_raster(card),
            "post {:?}: card = {card:?} is not a raster image. \
             A card exists precisely because crawlers refuse an SVG, so an \
             SVG card is never shared; use one of {OG_IMAGE_EXTENSIONS:?}",
            post.path
        );
        // Deliberately not falling back to `hero_alt`. The card and the hero
        // are different pictures, so the hero's words would describe the wrong
        // image -- and wrongly in the one place nobody proofreads, since
        // `og:image:alt` is only ever surfaced by someone else's client.
        let alt = post.card_alt.clone();
        anyhow::ensure!(
            alt.is_some(),
            "post {:?}: card = {card:?} has no card_alt. The card is a \
             different image from the hero, so it needs its own description",
            post.path
        );
        return Ok((Some(format!("{BASE_URL}{card}")), alt));
    }

    let Some(hero) = &post.hero else {
        return Ok((None, None));
    };
    if !is_raster(hero) {
        return Ok((None, None));
    }
    Ok((Some(format!("{BASE_URL}{hero}")), post.hero_alt.clone()))
}

/// `site` builds ./public. `site serve [port]` builds, then serves ./public
/// on localhost for development (see `serve.rs`).
fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        None => build(),
        Some("serve") => {
            let port = match args.next() {
                Some(p) => p
                    .parse()
                    .with_context(|| format!("not a port number: {p:?}"))?,
                None => serve::DEFAULT_PORT,
            };
            build()?;
            serve::run(Path::new(OUT), port)
        }
        Some(other) => anyhow::bail!("unknown command {other:?}; usage: site [serve [port]]"),
    }
}

fn build() -> Result<()> {
    let started = std::time::Instant::now();

    let hl = highlight::Highlighter::new();
    let cv: Cv = toml::from_str(&fs::read_to_string("content/cv.toml")?)
        .context("parsing content/cv.toml")?;
    cv.validate().context("content/cv.toml has a bad date")?;
    let mut posts = content::load_posts(Path::new("content/blog"), |body| djot::render(body, &hl))?;
    inline_svg_heroes(&mut posts)?;
    let article = knowledge::load(Path::new("content/ai"), |body| djot::render(body, &hl))?;
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
                // No site-wide social card exists; pointing this at a
                // missing file would be worse than omitting it.
                image: None,
                image_alt: None,
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
                image: None,
                image_alt: None,
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
                image: None,
                image_alt: None,
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
    fs::write(
        format!("{OUT}/cv_jonas_hansen_software_developer.pdf"),
        cv_pdf::render(&cv),
    )
    .context("writing public/cv_jonas_hansen_software_developer.pdf")?;
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
                image: None,
                image_alt: None,
            },
            syntax: false,
            site_css: &site_css,
            syntax_css: &syntax_css,
        }
        .render()?,
    )?;

    for post in &posts {
        let (image, image_alt) = og_image(post)?;
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
                    image,
                    image_alt,
                },
                syntax: post.has_syntax(),
                site_css: &site_css,
                syntax_css: &syntax_css,
            }
            .render()?,
        )?;
    }

    let search = build_search(&article, &posts)?;
    build_ai(&cv, &article, year, &site_css, &syntax_css, &search)?;

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
            article: &article,
            base: BASE_URL,
        }
        .render()?,
    )?;
    write(
        format!("{OUT}/robots.txt"),
        &format!("User-agent: *\nAllow: /\nSitemap: {BASE_URL}/sitemap.xml\n"),
    )?;

    println!(
        "built {} posts and {} knowledge nodes in {:.0?} -> {OUT}/",
        posts.len(),
        article.nodes.len(),
        started.elapsed()
    );

    Ok(())
}

/// Writes everything search needs under /search/: the model and vocabulary,
/// copied under content-hashed names; the site-wide index of AI article nodes
/// and blog posts, embedded here with the same model the browser will use;
/// and the palette module, which knows the index's hashed URL.
fn build_search(article: &Article, posts: &[Post]) -> Result<SearchAssets> {
    let base = format!("{OUT}/search");
    let url = "/search/";

    let model_dir = Path::new(AI_MODEL_DIR);
    let vocab = fs::read_to_string(model_dir.join("vocab.txt"))
        .with_context(|| format!("reading {AI_MODEL_DIR}/vocab.txt"))?;
    let weights = fs::read(model_dir.join("embeddings.q8.bin"))
        .with_context(|| format!("reading {AI_MODEL_DIR}/embeddings.q8.bin"))?;
    let model = embed::StaticModel::from_parts(&vocab, &weights)?;

    let weights_name = hashed_name("model", "bin", &weights);
    let vocab_name = hashed_name("vocab", "txt", vocab.as_bytes());
    write_bytes(format!("{base}/{weights_name}"), &weights)?;
    write(format!("{base}/{vocab_name}"), &vocab)?;

    let info = search_index::ModelInfo {
        name: AI_MODEL_NAME.to_string(),
        weights_url: format!("{url}{weights_name}"),
        vocab_url: format!("{url}{vocab_name}"),
        bytes: weights.len() + vocab.len(),
        dim: model.dim(),
    };
    let index = search_index::build(article, posts, &model, &info)?;
    let index_name = hashed_name("index", "json", index.as_bytes());
    write(format!("{base}/{index_name}"), &index)?;
    let index_url = format!("{url}{index_name}");

    write(
        format!("{base}/palette.js"),
        &format!(
            "const SEARCH_INDEX_URL = {index_url:?};
{PALETTE_SCRIPT}"
        ),
    )?;

    Ok(SearchAssets {
        index_url,
        model: info,
    })
}

/// Writes the three views of the AI article.
fn build_ai(
    cv: &Cv,
    article: &Article,
    year: i32,
    site_css: &str,
    syntax_css: &str,
    search: &SearchAssets,
) -> Result<()> {
    let base = format!("{OUT}/{}", article.path);
    let url = article.url();

    let meta = |title: String, description: String, path: &str, og_type: &'static str| Meta {
        title: format!("{title} — {}", cv.site.name),
        description,
        url: format!("{BASE_URL}{url}{path}"),
        og_type,
        image: None,
        image_alt: None,
    };

    write(
        format!("{base}/index.html"),
        &AiPage {
            cv,
            article,
            year,
            nav: "ai",
            view: "read",
            meta: meta(
                article.title.clone(),
                article.description.clone(),
                "",
                "article",
            ),
            syntax: article.has_syntax(),
            site_css,
            syntax_css,
        }
        .render()?,
    )?;
    write(
        format!("{base}/timeline/index.html"),
        &AiTimelinePage {
            cv,
            article,
            year,
            nav: "ai",
            view: "timeline",
            meta: meta(
                format!("Timeline: {}", article.title),
                format!(
                    "{} One line per idea, each linking to its full section.",
                    article.description
                ),
                "timeline/",
                "website",
            ),
            syntax: false,
            site_css,
            syntax_css,
        }
        .render()?,
    )?;
    write(
        format!("{base}/ask/index.html"),
        &AiAskPage {
            cv,
            article,
            year,
            nav: "ai",
            view: "ask",
            meta: meta(
                format!("Ask: {}", article.title),
                "Ask a question and get the most relevant sections of the article, found by \
                 semantic search that runs entirely in your browser."
                    .to_string(),
                "ask/",
                "website",
            ),
            syntax: false,
            site_css,
            syntax_css,
            index_url: &search.index_url,
            model: &search.model,
            script: ASK_SCRIPT,
        }
        .render()?,
    )?;

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
            card: None,
            card_alt: None,
            series: None,
            part: None,
            body: String::new(),
            source: String::new(),
            hero_html: None,
        }
    }

    /// A `Meta` whose `url` is absolute under `BASE_URL`, matching what
    /// `main` builds for a real page — content beyond that doesn't matter to
    /// the tests that use this fixture. `image`/`image_alt` are always
    /// `None` here, matching every non-post page and any post fixture with
    /// no hero; the image-specific tests below build a `Meta` directly
    /// instead of going through this helper.
    fn meta_fixture(path: &str, og_type: &'static str) -> Meta {
        Meta {
            title: "x".into(),
            description: "x".into(),
            url: format!("{BASE_URL}{path}"),
            og_type,
            image: None,
            image_alt: None,
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
        for label in ["Writing", "AI", "About", "CV"] {
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
            ("ai", ai_page_html()),
            ("ai-timeline", ai_timeline_html()),
            ("ai-ask", ai_ask_html()),
        ]
    }

    /// The real article, with bodies rendered through the real Djot renderer
    /// so the page tests see what ships.
    fn real_article() -> Article {
        let hl = highlight::Highlighter::new();
        knowledge::load(Path::new("content/ai"), |body| djot::render(body, &hl)).unwrap()
    }

    fn model_info_fixture() -> search_index::ModelInfo {
        search_index::ModelInfo {
            name: AI_MODEL_NAME.into(),
            weights_url: "/search/model.cccccccccccccccc.bin".into(),
            vocab_url: "/search/vocab.dddddddddddddddd.txt".into(),
            bytes: 4_000_000,
            dim: 128,
        }
    }

    fn ai_page_html() -> String {
        let cv = cv_fixture();
        let article = real_article();
        AiPage {
            cv: &cv,
            article: &article,
            year: 2026,
            nav: "ai",
            view: "read",
            meta: meta_fixture("/ai/", "article"),
            syntax: article.has_syntax(),
            site_css: SITE_CSS_FIXTURE,
            syntax_css: SYNTAX_CSS_FIXTURE,
        }
        .render()
        .unwrap()
    }

    fn ai_timeline_html() -> String {
        let cv = cv_fixture();
        let article = real_article();
        AiTimelinePage {
            cv: &cv,
            article: &article,
            year: 2026,
            nav: "ai",
            view: "timeline",
            meta: meta_fixture("/ai/timeline/", "website"),
            syntax: false,
            site_css: SITE_CSS_FIXTURE,
            syntax_css: SYNTAX_CSS_FIXTURE,
        }
        .render()
        .unwrap()
    }

    fn ai_ask_html() -> String {
        let cv = cv_fixture();
        let article = real_article();
        let model = model_info_fixture();
        AiAskPage {
            cv: &cv,
            article: &article,
            year: 2026,
            nav: "ai",
            view: "ask",
            meta: meta_fixture("/ai/ask/", "website"),
            syntax: false,
            site_css: SITE_CSS_FIXTURE,
            syntax_css: SYNTAX_CSS_FIXTURE,
            index_url: "/search/index.eeeeeeeeeeeeeeee.json",
            model: &model,
            script: ASK_SCRIPT,
        }
        .render()
        .unwrap()
    }

    #[test]
    fn the_article_renders_every_node_once_with_a_stable_anchor_and_permalink() {
        let html = ai_page_html();
        let article = real_article();
        for node in &article.nodes {
            let section = format!("<section class=\"node\" id=\"{}\">", node.id);
            assert_eq!(html.matches(&section).count(), 1, "{}", node.id);
            assert!(
                html.contains(&format!("class=\"node-anchor\" href=\"#{}\"", node.id)),
                "{}",
                node.id
            );
        }
        // One h1; parts are h2; nodes are h3 -- no level is skipped.
        assert_eq!(html.matches("<h1").count(), 1);
        assert_eq!(html.matches("<h2").count(), article.parts.len());
        assert_eq!(html.matches("<h3").count(), article.nodes.len());
        assert!(!html.contains("<h4"), "node bodies must not add headings");
    }

    #[test]
    fn the_article_shows_relationships_and_misreadings() {
        let html = ai_page_html();
        // kv-cache builds on qkv and context-window.
        let kv = &html[html.find("id=\"kv-cache\"").unwrap()..];
        let kv = &kv[..kv.find("</section>").unwrap()];
        assert!(kv.contains("Builds on"), "{kv}");
        assert!(
            kv.contains("<a href=\"#qkv\">Queries, keys and values</a>"),
            "{kv}"
        );
        assert!(kv.contains("Often misread as"), "{kv}");
    }

    /// The article and timeline are content first: they must work without
    /// JavaScript and must not pay for the Ask view's script.
    #[test]
    fn the_article_and_timeline_carry_no_script_beyond_the_site_wide_ones() {
        // The two theme scripts and the palette loader, as on every page.
        for html in [ai_page_html(), ai_timeline_html()] {
            assert_eq!(html.matches("<script").count(), 3, "{html}");
            assert!(!html.contains("data-ask"));
        }
    }

    /// Every page offers the palette, but only as a trigger: a hidden button
    /// the loader reveals, and a dynamic import of the module on first use.
    /// Nothing is fetched up front.
    #[test]
    fn every_page_carries_the_palette_loader_but_not_the_palette() {
        for (name, html) in all_pages_html() {
            assert!(
                html.contains("data-search hidden"),
                "{name}: button must start hidden"
            );
            assert_eq!(
                html.matches("import('/search/palette.js')").count(),
                1,
                "{name}"
            );
            assert!(
                !html.contains("modulepreload"),
                "{name}: must not load eagerly"
            );
        }
    }

    #[test]
    fn the_palette_module_has_the_retrieval_code_and_exports_open() {
        assert!(PALETTE_SCRIPT.contains("export function open()"));
        assert!(PALETTE_SCRIPT.contains("export function rank("));
        assert!(PALETTE_SCRIPT.contains("SEARCH_INDEX_URL"));
    }

    #[test]
    fn each_view_marks_itself_current_in_the_view_switcher() {
        for (html, current) in [
            (ai_page_html(), "Read"),
            (ai_timeline_html(), "Timeline"),
            (ai_ask_html(), "Ask"),
        ] {
            // The site nav's "AI" link, plus this view in the switcher.
            assert_eq!(html.matches("aria-current=\"page\"").count(), 2);
            asserts_single_nav_link_current(&html, &["AI"]);
            assert!(
                html.contains(&format!("aria-current=\"page\">{current}</a>")),
                "{current}"
            );
        }
    }

    #[test]
    fn the_timeline_links_every_node_to_its_section_in_order() {
        let html = ai_timeline_html();
        let article = real_article();
        let mut last = 0;
        for node in &article.nodes {
            let link = format!("href=\"/ai/#{}\"", node.id);
            let at = html
                .find(&link)
                .unwrap_or_else(|| panic!("no timeline entry for {}", node.id));
            assert!(at > last, "{} is out of order", node.id);
            last = at;
        }
        // Each part's list continues the numbering from the previous part.
        for part in &article.parts {
            let first = &article.nodes_in(part)[0];
            assert!(html.contains(&format!(
                "<ol class=\"timeline\" start=\"{}\">",
                first.order
            )));
        }
    }

    #[test]
    fn the_ask_page_inlines_one_lazy_module_and_starts_disabled() {
        let html = ai_ask_html();
        assert_eq!(
            html.matches("<script").count(),
            4,
            "theme scripts, palette loader and Ask"
        );
        assert!(html.contains("<script type=\"module\">"));
        assert!(html.contains("data-ask data-index=\"/search/index.eeeeeeeeeeeeeeee.json\""));
        // Without JavaScript, the form must not be able to submit a question
        // to the server in a URL.
        assert!(html.contains("type=\"search\" autocomplete=\"off\" spellcheck=\"false\""));
        let input = &html[html.find("id=\"ask-query\"").unwrap()..];
        assert!(
            input[..input.find('>').unwrap()].contains("disabled"),
            "{input}"
        );
        assert!(html.contains("<noscript>"));
        // The status line is announced as it changes.
        assert!(html.contains("role=\"status\" aria-live=\"polite\""));
    }

    /// The Ask script is inlined verbatim, so the HTML parser sees it: any
    /// `<script` or `</script` text inside would end it early (and the deploy
    /// guard counts `<script` occurrences per page).
    #[test]
    fn the_inlined_ask_script_cannot_close_its_own_tag() {
        let lower = ASK_SCRIPT.to_lowercase();
        assert!(!lower.contains("<script"));
        assert!(!lower.contains("</script"));
        assert!(!lower.contains("<!--"));
    }

    #[test]
    fn the_sitemap_lists_the_article_and_timeline() {
        let article = real_article();
        let xml = Sitemap {
            posts: &[],
            article: &article,
            base: BASE_URL,
        }
        .render()
        .unwrap();
        assert!(xml.contains("<loc>https://sasin91.xyz/ai/</loc>"), "{xml}");
        assert!(
            xml.contains("<loc>https://sasin91.xyz/ai/timeline/</loc>"),
            "{xml}"
        );
    }

    #[test]
    fn hashed_asset_names_match_the_immutable_cache_shape() {
        let name = hashed_name("model", "bin", b"weights");
        let (stem, rest) = name.split_once('.').unwrap();
        let (hash, ext) = rest.split_once('.').unwrap();
        assert_eq!((stem, ext), ("model", "bin"));
        assert_eq!(hash.len(), 16);
        assert!(
            hash.chars()
                .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
        );
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

    /// The landing, about, CV and blog-listing pages have no social-card
    /// asset to point at — see the comment on `Meta::image` and each
    /// `Meta { image: None, .. }` in `main` — so they must emit neither
    /// `og:image` nor the large-card `twitter:card`. The "post" fixture in
    /// `all_pages_html` has no hero either, so it belongs in this same
    /// assertion; the posts that *do* get an image are covered by the two
    /// tests below instead.
    #[test]
    fn pages_without_a_hero_image_emit_no_og_image() {
        for (name, html) in all_pages_html() {
            assert!(!html.contains("og:image"), "{name}: {html}");
            assert!(
                html.contains("name=\"twitter:card\" content=\"summary\""),
                "{name}: {html}"
            );
        }
    }

    /// Loads the real posts under `content/blog` rather than a hand-built
    /// fixture. The two og:image tests below key off which of these real
    /// posts has an SVG hero and which has a raster one, so a future post
    /// that changes format (SVG becomes the raster path's own test subject,
    /// or vice versa) is caught here instead of a stale fixture quietly
    /// drifting from what actually ships.
    fn real_posts() -> Vec<Post> {
        content::load_posts(Path::new("content/blog"), |body| Ok(body.to_string()))
            .expect("content/blog must parse for these tests")
    }

    /// The regression this guards: a naive "always use the hero as
    /// og:image" would emit an SVG URL here, which Facebook, LinkedIn and
    /// Twitter/X all silently discard (see `og_image`'s doc comment) — so
    /// this post must come out exactly like one with no hero at all.
    /// Checked against a real post, not a fixture, so a future edit that
    /// swaps this post's hero to a raster format is what breaks this test,
    /// rather than the test quietly asserting nothing. Posts that declare a
    /// `card` are excluded: an SVG hero *plus* a card is the case the test
    /// below covers, and it is meant to come out the opposite way.
    #[test]
    fn a_post_with_an_svg_hero_and_no_card_emits_no_og_image() {
        let posts = real_posts();
        let post = posts
            .iter()
            .find(|p| p.card.is_none() && p.hero.as_deref().is_some_and(|h| h.ends_with(".svg")))
            .expect("content/blog must still have a post with an SVG hero and no card");

        let (image, image_alt) = og_image(post).unwrap();
        assert_eq!(image, None, "SVG hero must not become og:image");
        assert_eq!(image_alt, None);

        let cv = cv_fixture();
        let html = PostPage {
            cv: &cv,
            post,
            year: 2026,
            nav: "blog",
            meta: Meta {
                image,
                image_alt,
                ..meta_fixture(&post.url(), "article")
            },
            syntax: post.has_syntax(),
            site_css: SITE_CSS_FIXTURE,
            syntax_css: SYNTAX_CSS_FIXTURE,
        }
        .render()
        .unwrap();

        assert!(!html.contains("og:image"), "{html}");
        assert!(
            html.contains("name=\"twitter:card\" content=\"summary\""),
            "{html}"
        );
    }

    /// The post this whole feature exists for, and the case `card` was added
    /// to make possible: an SVG hero — right on the page, worthless to a
    /// crawler — alongside a PNG card that must become an absolute
    /// `og:image`, carry `hero_alt` as `og:image:alt`, and flip
    /// `twitter:card` to `summary_large_image` (a `summary` card with a large
    /// image renders as a cramped thumbnail). Before `card` existed this post
    /// shared as a bare link.
    #[test]
    fn a_post_with_an_svg_hero_and_a_card_gets_an_absolute_og_image_and_a_large_card() {
        let posts = real_posts();
        let post = posts
            .iter()
            .find(|p| p.path == "blog/freebsd-on-hetzner")
            .expect("content/blog/freebsd-on-hetzner.dj must exist");
        assert_eq!(
            post.hero.as_deref(),
            Some("/images/freebsd-on-hetzner/header.svg"),
            "fixture assumption: this post's hero is the SVG that makes card necessary"
        );
        assert_eq!(
            post.card.as_deref(),
            Some("/images/freebsd-on-hetzner/card.png"),
            "fixture assumption: this post's card moved"
        );

        let (image, image_alt) = og_image(post).unwrap();
        assert_eq!(
            image.as_deref(),
            Some(concat!(
                "https://sasin91.xyz",
                "/images/freebsd-on-hetzner/card.png"
            )),
            "the card must win over the hero, and be absolute"
        );
        assert!(
            image_alt.is_some(),
            "hero_alt must carry through as the image alt"
        );

        let cv = cv_fixture();
        let html = PostPage {
            cv: &cv,
            post,
            year: 2026,
            nav: "blog",
            meta: Meta {
                image: image.clone(),
                image_alt: image_alt.clone(),
                ..meta_fixture(&post.url(), "article")
            },
            syntax: post.has_syntax(),
            site_css: SITE_CSS_FIXTURE,
            syntax_css: SYNTAX_CSS_FIXTURE,
        }
        .render()
        .unwrap();

        assert!(
            html.contains(&format!(
                "property=\"og:image\" content=\"{}\"",
                image.unwrap()
            )),
            "{html}"
        );
        assert!(
            html.contains(&format!(
                "property=\"og:image:alt\" content=\"{}\"",
                image_alt.unwrap()
            )),
            "{html}"
        );
        assert!(
            html.contains("name=\"twitter:card\" content=\"summary_large_image\""),
            "{html}"
        );
    }

    /// Builds a `Post` carrying just the fields `og_image` reads.
    fn post_with_images(hero: Option<&str>, card: Option<&str>) -> Post {
        Post {
            hero: hero.map(Into::into),
            hero_alt: Some("alt".into()),
            card: card.map(Into::into),
            ..post_fixture()
        }
    }

    /// With no `card`, a raster hero is still the fallback — adding `card`
    /// must not have quietly made the hero path dead.
    #[test]
    fn a_raster_hero_is_still_used_when_no_card_is_set() {
        let (image, _) = og_image(&post_with_images(Some("/images/x/hero.png"), None)).unwrap();
        assert_eq!(
            image.as_deref(),
            Some("https://sasin91.xyz/images/x/hero.png")
        );
    }

    /// An SVG `card` is the original bug typed into the field invented to
    /// prevent it, and it fails silently everywhere it matters: the tag ships,
    /// the page looks fine, and every crawler drops the image. Stopping the
    /// build is the only place it can be noticed.
    #[test]
    fn an_svg_card_stops_the_build() {
        let err = og_image(&post_with_images(None, Some("/images/x/card.svg")))
            .expect_err("an SVG card must not build");
        let msg = err.to_string();
        assert!(msg.contains("card.svg"), "error must name the value: {msg}");
        assert!(msg.contains("blog/x"), "error must name the post: {msg}");
    }

    /// A card with no description of its own would otherwise silently inherit
    /// `hero_alt`, which describes a different picture. `og:image:alt` is only
    /// ever surfaced by someone else's client, so a wrong value here is the
    /// kind nobody on this side ever sees.
    #[test]
    fn a_card_without_its_own_alt_stops_the_build() {
        let err = og_image(&post_with_images(None, Some("/images/x/card.png")))
            .expect_err("a card with no card_alt must not build");
        let msg = err.to_string();
        assert!(msg.contains("card_alt"), "error must name the field: {msg}");
        assert!(msg.contains("blog/x"), "error must name the post: {msg}");
    }

    /// The real shape on `freebsd-on-hetzner`: an SVG hero for the page and a
    /// PNG card for crawlers, each described in its own words.
    #[test]
    fn a_card_is_shared_with_its_own_description_not_the_heros() {
        let post = Post {
            card_alt: Some("a latency chart".into()),
            series: None,
            part: None,
            ..post_with_images(Some("/images/x/hero.svg"), Some("/images/x/card.png"))
        };
        let (image, alt) = og_image(&post).unwrap();
        assert_eq!(
            image.as_deref(),
            Some("https://sasin91.xyz/images/x/card.png")
        );
        assert_eq!(
            alt.as_deref(),
            Some("a latency chart"),
            "must not fall back to hero_alt"
        );
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
