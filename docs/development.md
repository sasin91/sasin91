# Developing sasin91.xyz

The site is a [Trongate](https://trongate.io) v2 app. Every post is an ordinary
PHP view, rendered on request. There is no build step: edit a post, reload the
page.

Trongate's own modules (`db`, `login`, `templates`, `validation` and the rest)
and `engine/` are stock v2 and stay that way. Everything the site adds is in
these modules:

| Module | What it is |
| --- | --- |
| `content` | Loads the posts, the CV and the AI article, and renders a post's view (`Prose_view`). Answers no URL itself. |
| `prose` | What a post is written with: `section_open()`, `code_open()` and the rest (`prose_helper.php`). Answers no URL. |
| `layout` | The page shell every HTML page shares (`views/page.php`). |
| `site` | `/`, `/about/`, `/rss.xml`, `/sitemap.xml`, `/robots.txt`, the 404 page. |
| `blog` | `/blog/` and every post under it. |
| `knowledge` | The AI article: `/blog/ai/`, `timeline/`, `ask/`. |
| `cv` | `/cv/` and `/cv_jonas_hansen_software_developer.pdf`. |
| `search` | `/search/*`: the index, the model, the palette and Ask scripts. |

URLs are mapped to modules in `config/custom_routing.php`, so every page keeps
the address it had as a static site.

## Run it

PHP 8.4 with `intl` and `mbstring`, and Composer for the libraries the site
needs (league/commonmark, TOML, highlight.php, LaTeX to MathML, dompdf):

```sh
composer install
php -S 127.0.0.1:8000 -t public bin/dev-router.php
```

Or point Herd (or any web server) at `public/`; `public/.htaccess` sends every
request that is not a file to `index.php`.

A post is one view under `modules/blog/views/posts/`, starting with a `+++` TOML
header. The
`path` key is the URL, and is deliberately not derived from the filename, which
is what keeps `/blog/trongate/mx-transition` nested rather than flattened.

The body is plain HTML, with helpers in the style of Trongate's `form_open()` and
`form_close()` for the parts that need more than a tag:

```php
<?= section_open('4. Remaster the ISO') ?>
<p>Three edits …</p>
<?= code_open('bash', '4-remaster.sh') ?>
apt-get install -y xorriso qemu-system-x86
<?= code_close() ?>
<?= callout_open('warning', 'What went wrong') ?>
<p>I replaced <code>loader.conf</code> …</p>
<?= callout_close() ?>
<?= section_close() ?>
```

- `section_open($title, $level = 2)` / `section_close()`: a heading and what
  follows it, in a `<section>` with the id links point at;
- `callout_open('note' or 'warning', $title = null)` / `callout_close()`;
- `code_open($language, $title = null)` / `code_close()`: the code between them
  is shown as written, escaped and highlighted. Code that contains PHP's own
  `<?` tags would run there, so it goes to `code($language, $code, $title)` as a
  nowdoc string instead;
- `diagram($src, $alt)`: a local SVG, inlined so it follows the theme toggle;
- `math($tex)`: TeX as MathML, `math($tex, true)` on its own line.

Write typographic quotes and dashes as the characters themselves (“ ” ’ – —).

The AI article at `/blog/ai/` is different: it is assembled from knowledge nodes
under `modules/knowledge/`, with a timeline and an in-browser Ask view generated from
the same nodes. See [knowledge-nodes.md](knowledge-nodes.md) for how to add,
edit, reorder and link nodes.

Stylesheets are linked with a hash of their bytes in the query string
(`/css/site.css?v=…`), so a browser can cache them for good and still never
keep an old copy. `public/css/syntax.css` is generated from highlight.php's
palettes by `php bin/build-syntax-css.php`.

The search index is built from the posts and nodes the first time anyone asks for it
after the content changed, and kept in the system temp directory until it
changes again (`modules/search/Search.php`).

## Checks

```sh
php bin/run-tests.php modules/content/tests modules/search/tests
php bin/search-index.php > build/search-index.json && node --test modules/search/js/
```

The PHP tests are `.phpt` files next to the code they test. The JavaScript
tests use Node's built-in test runner (Node 22 or later); there is no
`package.json` and nothing to install. Their last group asks real questions of
the index the site serves, which is why it is written to `build/` first.

CI (`.github/workflows/pipeline.yml`) runs both, then serves the site and
checks that every page answers without a PHP warning and carries no script it
should not.

## Deploy

Pushing to `main` deploys to trongate.cloud. See [deploy.md](deploy.md).

## History

This replaced a static site generator written in Rust, which in turn replaced a
Laravel + Inertia + React application. The Rust version is in this
repository's history on `main`; the design decisions that led to it are in
`docs/superpowers/specs/`.
