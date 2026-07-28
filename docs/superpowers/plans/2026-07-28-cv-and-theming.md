# CV, Theming and Accessibility Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a persistent theme toggle, fix the accessibility and figure-ground defects a design audit found, and turn `cv.toml` into the single source for a full `/cv` page that also exports as PDF.

**Architecture:** Theme state lives on `<html data-theme>`, set by an inline pre-paint script and toggled by one button; CSS resolves every colour through `light-dark()` driven by `color-scheme`. SVG diagrams are inlined by the generator so they inherit that state. `cv.toml` grows to carry the full CV and feeds the homepage, `/about`, `/cv` and a CI-generated `/cv.pdf`.

**Tech Stack:** Rust 2024 (`jotdown`, `askama`, `syntect`, `latex2mathml`, `serde`, `toml`, `chrono`, `walkdir`, `anyhow`), plain CSS, ~18 lines of vanilla JS, headless Chrome in CI.

## Global Constraints

- Design doc: `docs/superpowers/specs/2026-07-28-cv-and-theming-design.md`. Read it first.
- **No JavaScript may be required to read anything.** Script enhances, never gates. With JS disabled the site must render fully and follow the OS theme.
- The only script on the site is the theme toggle: one inline pre-paint snippet and one inline handler, together under ~25 lines. No libraries, no bundler, no external requests.
- These URLs must not break: `/`, `/blog`, `/blog/trongate`, `/blog/trongate/mx-transition`, `/blog/freebsd-on-hetzner`, `/blog/athletos-freebsd`, `/about/`.
- The CV in `docs/reference/cv-source.md` is authoritative for dates and achievements. Do not invent employment history, achievements, or skills.
- Contrast must not regress below WCAG AA in either theme. Current: body 17.28:1, links 6.55:1, callout text 16.5:1.
- Every interactive element must be keyboard reachable with a visible focus indicator.
- No new Rust dependencies.
- `public/` is disposable and fully regenerated. Every task ends with a commit.
- Verify with `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt --check` — all must stay green.

---

## File Structure

| File | Responsibility |
|---|---|
| `docs/reference/cv-source.md` | Transcribed source CV — reference, not rendered |
| `content/cv.toml` | Profile, contact, roles with achievements, skills, education |
| `src/main.rs` | Build orchestration and template structs |
| `src/djot.rs` | Djot rendering; gains SVG inlining |
| `src/highlight.rs` | syntect wrapper; stylesheet gains `data-theme` support |
| `templates/base.html` | Shell: skip link, landmarks, theme toggle |
| `templates/cv.html` | The full CV page |
| `templates/about.html` | Short bio |
| `static/site.css` | All styling, including print rules |
| `static/images/**/*.svg` | Diagrams, reworked to theme-aware chrome |
| `.github/workflows/deploy.yml` | Gains the PDF generation step |

---

### Task 1: Transcribe the source CV into the repo

Everything downstream reads from this. It is reference material, not a rendered page.

**Files:**
- Create: `docs/reference/cv-source.md`

**Interfaces:**
- Consumes: nothing
- Produces: the authoritative record later tasks draw from

- [ ] **Step 1: Write the transcription**

Create `docs/reference/cv-source.md` with exactly this content. Danish characters matter — `Næstved`, `Strøm`, `Høng`. The source PDF misspells Copenhagen as "Copenhangen" in two places; corrected here, and noted at the bottom.

````markdown
# CV source — Jonas Hansen

Transcribed from `CV_Jonas_Hansen_Softwareengineer.pdf`, 2026-07-28.
This file is the authoritative record for `content/cv.toml`. It is not rendered.

**Jonas Hansen — Software developer**
Slagelse, 4200 · +45 50106917 · jonas.kerwin.hansen@gmail.com

## Intro

I have been working with PHP and Laravel since 2015 and since then I have helped
develop a ticketing agency, a video streaming platform and a recruitment platform.

I enjoy designing and delivering customized and solid solutions, but also takes
support and listening to user requests and issues with a smile.

## Work history & achievements

### January 2017 – February 2020 — IT & developer, GHC Travel / Iraqi Airways, Copenhagen
Airline booking platform

- Maintained and developed a basic PHP platform
- Migrated from traditional PHP web host to VPS servers I setup, managed and monitored
- Successfully migrated to Laravel 6 and maintained up version 8, including comprehensive PHPUnit test suite ensuring fast feedback loops on changes
- Built 3 distinct frontends in Nuxt 2 consuming API hosted on Laravel backend, ensuring better scalability, user experience and content delivery via CDN
- Greatly improved ticketing workflows by automating airport terminal interactions using UiPath RPA, improving ticket sales and customer satisfaction

### April 2020 – January 2023 — Software developer, Syncronet, Slagelse
Live streaming social media platform

- Led migration from an expensive setup on Azure using media services to Linode with mux.com for video delivery
- Architectured a Kubernetes cluster with Go services and Nginx endpoints wrapping FFmpeg for internal video processing and delivery
- Led frontend development in Nuxt 3 and mobile development in React Native while maintaining Laravel backend

### January 2023 – August 2023 — Web developer, JUICE ApS, Copenhagen
Job & candidate matchmaking platform

- Introduced CI/CD enabling efficient and fast delivery of Symfony 6
- Contributed to multiple features in Symfony & Twig + Stimulus.js

### September 2023 – September 2024 — Web developer, Supeo, Næstved
Web development agency

- Delivered multiple features on Supeo Flex, in React, Express.js & GraphQL
- Gained sole responsibility of customer interactions and development of SamFocus in Laravel 9

### September 2024 – February 2026 — Web developer, JUICE ApS, Copenhagen
Job & candidate matchmaking platform

- Upgraded Symfony 6 to 7
- Built a comprehensive ranking and sorting engine, delivering a quick and efficient method of finding candidates and sorting by relevancy
- AI integration, making it a breeze to upload a job ad and receiving a SmartMatch job post

## Skills

- Linux server management and maintenance
- Web & App development
- Database administration

## Education

Two short-cycle higher educations.

### January 2012 – August 2013 — Strøm, styring & IT, Selandia CEU, Slagelse
Including Cisco CCNA and IP based network management.

### February 2014 – August 2015 — Web integrator, Roskilde Teknisk Skole, Høng

## Notes

- The source PDF spells Copenhagen as "Copenhangen" twice. Corrected above.
- Since February 2026: job-seeking. Not listed as a role.
````

- [ ] **Step 2: Commit**

```bash
git add docs/reference/cv-source.md
git commit -m "docs: transcribe the source CV as the authoritative record"
```

---

### Task 2: Accessibility floor — skip link, landmarks, focus styles

The audit found **no focus styles anywhere**. This task fixes that and the surrounding structure before anything new is added on top.

**Files:**
- Modify: `templates/base.html`, `static/site.css`

**Interfaces:**
- Consumes: nothing
- Produces: `.skip-link` and a focus-ring convention later tasks reuse

- [ ] **Step 1: Add a skip link and landmarks to `templates/base.html`**

Immediately after `<body>`, before `<div class="wrap">`:

```html
<a class="skip-link" href="#content">Skip to content</a>
```

Give the existing `<nav class="site-nav">` an accessible name, and the main element an id:

```html
<nav class="site-nav" aria-label="Main">
```
```html
<main id="content">
```

The `<header class="site-head">` and `<footer class="site-foot">` are already implicit `banner` and `contentinfo` landmarks because they are direct children of `body`'s wrapper — verify that is still true given `.wrap` sits between them, and if not, add `role="banner"` and `role="contentinfo"` explicitly. Check rather than assume; a `<header>` nested inside a `<div>` is still `banner` only if not scoped to an article or section.

- [ ] **Step 2: Add focus and skip-link styles to `static/site.css`**

```css
/* ---- focus ----
   The audit found no focus styles at all: keyboard users got only the browser
   default, which `text-decoration: none` on nav links made worse. */

:focus-visible {
  outline: 2px solid var(--accent);
  outline-offset: 2px;
  border-radius: 2px;
}

/* Only suppress the ring for pointer users, never for keyboard. */
:focus:not(:focus-visible) {
  outline: none;
}

.skip-link {
  position: absolute;
  left: -9999px;
  top: 0;
  padding: 0.6rem 1rem;
  background: var(--bg);
  color: var(--accent);
  border: 1px solid var(--accent);
  border-radius: 0 0 5px 0;
  z-index: 10;
}

.skip-link:focus {
  left: 0;
}
```

- [ ] **Step 3: Verify by keyboard**

Build, serve, and tab through a post page from the top. Confirm in order: the skip link appears on first Tab, activating it moves focus into `<main>`, every nav link shows a visible ring, and every in-content link shows one. Report the tab order you observed.

- [ ] **Step 4: Commit**

```bash
git add templates/base.html static/site.css
git commit -m "feat(a11y): skip link, landmark labels and visible focus styles"
```

---

### Task 3: Colour tokens move to `light-dark()`

Prerequisite for the toggle. Today the dark palette lives in a `prefers-color-scheme` block, which a `data-theme` attribute cannot override. Moving to `light-dark()` means each token is declared once and `color-scheme` decides which half applies — so the toggle only has to change `color-scheme`.

**Files:**
- Modify: `static/site.css`

**Interfaces:**
- Consumes: nothing
- Produces: `:root[data-theme="light"|"dark"]` as the theme switch point

- [ ] **Step 1: Replace the two palette blocks with one**

Currently `:root { ... }` holds the light values and `@media (prefers-color-scheme: dark) { :root { ... } }` holds the dark ones. Replace both with a single block using `light-dark()`, pairing each existing light value with its existing dark counterpart:

```css
:root {
  color-scheme: light dark;

  --bg:        light-dark(#fdfdfb, #141417);
  --surface:   light-dark(#f5f5f1, #1d1d21);
  --text:      light-dark(#191918, #e8e8e4);
  --muted:     light-dark(#6c6c68, #97978f);
  --rule:      light-dark(#e3e3dd, #2b2b31);
  --accent:    light-dark(#2f5aa8, #8fb2ea);
  --warn-bg:   light-dark(#fdf7ec, #241f14);
  --warn-edge: light-dark(#c99a3e, #b08434);

  --sans: ui-sans-serif, system-ui, -apple-system, "Segoe UI", Roboto,
    "Helvetica Neue", Arial, sans-serif;
  --mono: ui-monospace, "SF Mono", "Cascadia Mono", "JetBrains Mono", Menlo,
    Consolas, monospace;

  --measure: 68ch;
}

/* The toggle writes data-theme; these pin color-scheme so light-dark() resolves
   the chosen side regardless of the OS setting. */
:root[data-theme="light"] { color-scheme: light; }
:root[data-theme="dark"]  { color-scheme: dark; }
```

Then find every remaining `@media (prefers-color-scheme: dark)` block in the file and convert its declarations to `light-dark()` on the base rule, deleting the media query. The filename-bar rule is one of these. Do not leave any `prefers-color-scheme` block behind that sets a colour — each one is a place the toggle would fail to reach.

- [ ] **Step 2: Verify no colour is left behind a media query**

```bash
grep -n 'prefers-color-scheme' static/site.css
```

Expected: no matches, or only matches that set something other than a colour. Report what remains and why.

- [ ] **Step 3: Verify both themes still render**

Build and screenshot the homepage and one post under `colorScheme: 'light'` and `'dark'`. Nothing should look different from before this task — this is a refactor, not a redesign. Report any visual change you see, because there should be none.

- [ ] **Step 4: Commit**

```bash
git add static/site.css
git commit -m "refactor(css): resolve colour tokens through light-dark()

Each token is declared once; color-scheme decides which half applies, so
a data-theme attribute can override the OS preference."
```

---

### Task 4: The syntax stylesheet must follow `data-theme` too

`src/highlight.rs` emits the dark palette inside `@media (prefers-color-scheme: dark)`. That media query cannot see `data-theme`, so with a toggle the page would change theme and the code blocks would not.

**Files:**
- Modify: `src/highlight.rs`

**Interfaces:**
- Consumes: nothing
- Produces: `syntax.css` responding to both the OS preference and `data-theme`

- [ ] **Step 1: Write the failing test**

Add to `src/highlight.rs`'s test module:

```rust
#[test]
fn stylesheet_follows_the_data_theme_attribute_as_well_as_the_os() {
    let hl = Highlighter::new();
    let css = hl
        .stylesheet("Solarized (light)", "base16-ocean.dark")
        .unwrap();

    // The OS path, but yielding to an explicit light choice.
    assert!(css.contains("prefers-color-scheme: dark"), "got: {css}");
    assert!(css.contains("[data-theme=\"light\"]"), "got: {css}");

    // The explicit dark choice, independent of the OS.
    assert!(css.contains("[data-theme=\"dark\"]"), "got: {css}");
}
```

- [ ] **Step 2: Run it to verify it fails**

Run: `cargo test highlight::tests::stylesheet_follows_the_data_theme_attribute_as_well_as_the_os`
Expected: FAIL — no `data-theme` in the generated CSS.

- [ ] **Step 3: Emit the dark palette under both conditions**

Replace the body of `stylesheet()`'s `Ok(format!(...))` with:

```rust
        let dark_css = css_for_theme_with_class_style(dark, STYLE)?;

        Ok(format!(
            "/* generated by the site builder - do not edit */\n\
             {}\n\
             /* The OS preference, unless an explicit light choice overrides it. */\n\
             @media (prefers-color-scheme: dark) {{\n\
             :root:not([data-theme=\"light\"]) {{\n{}\n}}\n\
             }}\n\
             /* An explicit dark choice, whatever the OS says. */\n\
             :root[data-theme=\"dark\"] {{\n{}\n}}\n",
            css_for_theme_with_class_style(light, STYLE)?,
            dark_css,
            dark_css,
        ))
    }
```

This relies on CSS nesting: the generated `.hl-*` rules sit inside a `:root[...]` block and resolve as descendant selectors. **Verify this actually works in a browser** rather than trusting it — render a page, force `data-theme="dark"` on `<html>` via devtools or a temporary attribute, and confirm the code block colours change. If nesting does not apply, report back rather than working around it.

- [ ] **Step 4: Run the tests**

Run: `cargo test`
Expected: 28 passing.

- [ ] **Step 5: Commit**

```bash
git add src/highlight.rs
git commit -m "feat(highlight): make syntax.css follow data-theme, not just the OS"
```

---

### Task 5: The theme toggle

**Files:**
- Modify: `templates/base.html`, `static/site.css`

**Interfaces:**
- Consumes: `:root[data-theme]` from Task 3, `syntax.css` from Task 4
- Produces: a working, persistent, keyboard-accessible toggle

- [ ] **Step 1: Add the pre-paint script to `<head>`**

It must come before the stylesheet links so the attribute is set before first paint — otherwise the page flashes the wrong theme.

```html
<script>
  (function () {
    try {
      var t = localStorage.getItem('theme');
      if (t === 'light' || t === 'dark') {
        document.documentElement.dataset.theme = t;
      }
    } catch (e) {}
  })();
</script>
```

The `try`/`catch` matters: `localStorage` throws in some privacy modes, and an uncaught error here would leave the rest of the page unstyled.

- [ ] **Step 2: Add the button to the nav**

Inside `<nav class="site-nav">`, after the links:

```html
<button type="button" id="theme-toggle" class="theme-toggle" aria-pressed="false">
  Dark theme
</button>
```

`aria-pressed` is correct here: the button's label names a state ("Dark theme") and pressed reports whether it is active.

- [ ] **Step 3: Add the handler before `</body>`**

```html
<script>
  (function () {
    var button = document.getElementById('theme-toggle');
    if (!button) return;
    var root = document.documentElement;

    function current() {
      return root.dataset.theme ||
        (matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light');
    }

    function sync() {
      button.setAttribute('aria-pressed', String(current() === 'dark'));
    }

    sync();

    button.addEventListener('click', function () {
      var next = current() === 'dark' ? 'light' : 'dark';

      function apply() {
        root.dataset.theme = next;
        try { localStorage.setItem('theme', next); } catch (e) {}
        sync();
      }

      // Same-document view transition: the crossfade of a page transition
      // without the reload, so scroll position needs no handling at all.
      if (document.startViewTransition &&
          !matchMedia('(prefers-reduced-motion: reduce)').matches) {
        document.startViewTransition(apply);
      } else {
        apply();
      }
    });
  })();
</script>
```

- [ ] **Step 4: Style the button and add view transitions**

```css
.theme-toggle {
  font: inherit;
  font-size: 0.94rem;
  color: var(--muted);
  background: none;
  border: none;
  padding: 0;
  cursor: pointer;
}

.theme-toggle:hover {
  color: var(--accent);
}

/* Cross-document transitions, for moving between pages. Needs no JavaScript
   and is silently ignored where unsupported. The theme flip uses the
   same-document API instead — see the toggle handler. */
@view-transition {
  navigation: auto;
}

::view-transition-old(root),
::view-transition-new(root) {
  animation-duration: 220ms;
}

@media (prefers-reduced-motion: reduce) {
  ::view-transition-group(*),
  ::view-transition-old(*),
  ::view-transition-new(*) {
    animation: none !important;
  }
}
```

- [ ] **Step 5: Verify all four behaviours**

Serve the site and check each, reporting what you observed:

1. **Persistence** — toggle to dark, navigate to another page, confirm it stays dark.
2. **No flash** — hard-reload with dark stored and watch for a light flash before paint. Throttle the network if needed to make it visible.
3. **Keyboard** — reach the button by Tab alone, activate with both Enter and Space, and confirm the focus ring is visible.
4. **No-JS** — disable JavaScript, reload, and confirm the page renders fully and follows the OS theme. The button will do nothing; that is acceptable, but the page must not be broken.

Also confirm the code blocks change with the toggle, not just with the OS — this is what Task 4 was for.

- [ ] **Step 6: Commit**

```bash
git add templates/base.html static/site.css
git commit -m "feat: persistent theme toggle and cross-document view transitions"
```

---

### Task 6: Inline SVG diagrams so they inherit the theme

An SVG in an `<img>` is a separate document and cannot see `data-theme`. Without this, toggling against the OS setting leaves the diagrams on the wrong theme.

**Files:**
- Modify: `src/djot.rs`

**Interfaces:**
- Consumes: `html::escape`
- Produces: local `.svg` images inlined into the document

- [ ] **Step 1: Verify jotdown's image event shape**

Before writing anything, read the `Container` enum in
`~/.cargo/registry/src/*/jotdown-0.10.0/src/lib.rs` and find the image variant and its fields. Write a scratch program that renders `![alt](/images/x.svg)` and prints the event stream, so you know exactly what you are matching on. Report the actual shape — do not assume it matches any guess in this plan.

- [ ] **Step 2: Write the failing tests**

Add to `src/djot.rs`'s test module:

```rust
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
    assert!(html.contains("A diagram"), "alt must survive as a label: {html}");

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
```

- [ ] **Step 3: Run them to verify they fail**

Run: `cargo test djot::`
Expected: FAIL — `render_with_assets` does not exist.

- [ ] **Step 4: Implement**

Change `render` to take the static root, keeping a convenience wrapper so existing tests and call sites stay readable:

```rust
/// Render with no asset root — SVG inlining is skipped.
pub fn render(source: &str, hl: &Highlighter) -> Result<String> {
    render_with_assets(source, hl, Path::new("static"))
}

/// Render, inlining local `.svg` images found under `assets`.
pub fn render_with_assets(source: &str, hl: &Highlighter, assets: &Path) -> Result<String> {
```

Inside the event loop, match the image start event you confirmed in Step 1. When the destination starts with `/` and ends with `.svg`, read `assets.join(destination.trim_start_matches('/'))`, and splice the file's contents as a raw block wrapped so it carries the alt text as an accessible name:

```rust
format!(
    "<figure class=\"diagram\" role=\"img\" aria-label=\"{}\">{}</figure>",
    escape(&alt),
    svg
)
```

Collect the alt text from the `Event::Str` events between the image's start and end, as the code-block handling already does for its buffer. A missing file must return `Err` with context naming the path — a diagram that silently vanishes is worse than a failed build.

Leave remote URLs and non-`.svg` images to jotdown's normal image rendering.

- [ ] **Step 5: Run the tests**

Run: `cargo test`
Expected: 31 passing.

- [ ] **Step 6: Verify against the real posts**

```bash
cargo run --release
grep -c '<svg' public/blog/athletos-freebsd/index.html
grep -c '<img' public/blog/athletos-freebsd/index.html
```

Expected: 3 inlined SVGs (hero plus two figures), and no `<img>` left pointing at a `.svg`. Confirm the pages still look right.

- [ ] **Step 7: Commit**

```bash
git add src/djot.rs
git commit -m "feat(djot): inline local SVGs so diagrams inherit the page theme"
```

---

### Task 7: Make the diagram chrome theme-aware

**Files:**
- Modify: `static/images/athletos-freebsd/architecture.svg`, `deploy-timeline.svg`, `footprint.svg`, `static/images/freebsd-on-hetzner/header.svg`
- Modify: `static/site.css`

**Interfaces:**
- Consumes: inlining from Task 6, colour tokens from Task 3
- Produces: diagrams that follow the toggle

- [ ] **Step 1: Identify chrome versus data in each file**

For each SVG, list its colours and classify each as **chrome** (panel background, gridlines, axis rules, label and caption text) or **data** (the coloured bars and nodes that encode a category — ARC orange, Postgres blue, SvelteKit purple, Caddy pink, API green).

Report the classification before changing anything. Getting this wrong is the failure mode: recolouring a data mark breaks the reader's mapping between a legend and the thing it labels.

- [ ] **Step 2: Convert chrome to CSS custom properties**

In each SVG, replace chrome colour attributes with `var()` references carrying the current value as a fallback, so the file still renders standalone:

```xml
<rect width="1200" height="400" fill="var(--diagram-bg, #12121b)"/>
<text fill="var(--diagram-text, #e8e8e4)">ZFS ARC</text>
```

Leave every data colour exactly as it is.

- [ ] **Step 3: Define the diagram tokens in `static/site.css`**

```css
:root {
  --diagram-bg:    light-dark(#f5f5f1, #12121b);
  --diagram-panel: light-dark(#eae9e3, #1b1b26);
  --diagram-text:  light-dark(#33332f, #e8e8e4);
  --diagram-muted: light-dark(#6c6c68, #9a9aa6);
  --diagram-rule:  light-dark(#d8d8d2, #2b2b36);
}

.prose .diagram {
  margin: 1.8rem 0;
  border: 1px solid var(--rule);
  border-radius: 7px;
  overflow: hidden;
}

.prose .diagram svg {
  display: block;
  width: 100%;
  height: auto;
}
```

- [ ] **Step 4: Verify the data colours still read against the light panel**

This is the real risk of the task. The data colours were chosen against a near-black panel; on a light one, some may lose contrast. Compute the contrast ratio of each data colour against the new light `--diagram-bg` and report the numbers. Anything below **3:1** needs its light-mode counterpart adjusted — add a `light-dark()` pair for that specific mark rather than abandoning the light panel.

- [ ] **Step 5: Screenshot all four diagrams in both themes**

Confirm each is legible in light and dark, and that toggling the theme changes them. Report anything that looks wrong.

- [ ] **Step 6: Commit**

```bash
git add static/images templates static/site.css
git commit -m "feat: theme-aware diagram chrome, fixed data colours"
```

---

### Task 8: Fix the figure-ground inversion

The audit measured dark diagram panels at 16.6:1 against the page and solarized code blocks at 1.06:1 — so images shouted and code, carrying denser information, receded.

**Files:**
- Modify: `static/site.css`

- [ ] **Step 1: Give code blocks a region in light mode**

**Read `static/site.css` before editing.** `.codeblock` (around line 292)
already declares `border`, `border-radius` and `overflow`, and
`.codeblock pre.hl-code` (around line 304) already exists. Do not re-declare
either — amend them in place. Only the *unwrapped* block is missing a border,
because a titled block gets one from its wrapper.

Add a border to the bare `pre.hl-code` rule that already exists:

```css
/* Solarized cream on a near-white page is 1.06:1, so an unwrapped block does
   not read as an object at all. A titled block gets its border from
   .codeblock instead. */
pre.hl-code {
  /* ...existing declarations... */
  border: 1px solid var(--rule);
}
```

And add one declaration to the existing `.codeblock pre.hl-code` rule, so a
framed block does not draw two borders:

```css
.codeblock pre.hl-code {
  margin: 0;
  border-radius: 0;
  border: none;
}
```

- [ ] **Step 2: Frame dark media consistently**

The Trongate logo is a near-white artwork that vanishes on the light page. Give it the same treatment as the diagrams so dark-designed media reads as one deliberate family:

Note `.prose img` already sets `border-radius: 6px` while the diagram frame
uses `7px`. Unify them on `7px` so framed media matches.

```css
/* The Trongate logo is a near-white artwork that disappears on the light page.
   Giving raster media the same frame as the diagrams makes dark-designed
   media read as one deliberate family. Inlined SVGs are not <img> and are
   handled by .diagram. */
.prose img[src$=".png"],
.prose img[src$=".webp"] {
  border: 1px solid var(--rule);
  background: var(--diagram-bg);
}
```

- [ ] **Step 3: Recompute contrast and confirm the inversion is gone**

Report the code block's border contrast against the page and confirm the block now reads as a distinct region in light mode. Screenshot a post in light mode showing a code block and a diagram together — the two should look like siblings, not like a whisper next to a shout.

- [ ] **Step 4: Commit**

```bash
git add static/site.css
git commit -m "fix(design): give code blocks a region, frame dark media consistently"
```

---

### Task 9: `cv.toml` carries the full CV

**Files:**
- Modify: `content/cv.toml`, `src/main.rs`

**Interfaces:**
- Consumes: `docs/reference/cv-source.md`
- Produces: `Cv` with `contact`, `intro`, `roles` (with `achievements`, `start`, `end`), `skills`, `education`

- [ ] **Step 1: Restructure `content/cv.toml`**

Transcribe from `docs/reference/cv-source.md`. Every date, company, title and achievement must match it exactly. Structure:

```toml
[site]
name = "Jonas Hansen"
title = "Software developer"
stack = "PHP · Laravel · Symfony · Go · Rust · Nuxt · React · Kubernetes · FreeBSD"
available = true
available_note = "Available for work"

[site.links]
github = "https://github.com/sasin91"
linkedin = "https://www.linkedin.com/in/jonas-hansen-2b6828110"
email = "mailto:jonas.kerwin.hansen@gmail.com"

[contact]
town = "Slagelse"
postcode = "4200"
phone = "+45 50106917"
email = "jonas.kerwin.hansen@gmail.com"

[[roles]]
start = "2024-09"
end = "2026-02"
title = "Web developer"
company = "JUICE ApS"
location = "Copenhagen"
summary = "Job & candidate matchmaking platform"
achievements = [
  "Upgraded Symfony 6 to 7",
  "Built a comprehensive ranking and sorting engine, delivering a quick and efficient method of finding candidates and sorting by relevancy",
  "AI integration, making it a breeze to upload a job ad and receiving a SmartMatch job post",
]
```

Continue for all five roles, newest first, exactly as `cv-source.md` records them. Then:

```toml
[[skills]]
name = "Linux server management and maintenance"

[[education]]
start = "2014-02"
end = "2015-08"
title = "Web integrator"
school = "Roskilde Teknisk Skole"
location = "Høng"
```

The `stack` line above is a first draft for Jonas to edit — flag it in your report as needing his review rather than treating it as settled.

- [ ] **Step 2: Update the Rust types in `src/main.rs`**

Replace `Job` with `Role`, and add `Contact`, `Skill` and `Education`. Every field the templates read must exist on the struct — Askama checks this at compile time, so a mismatch is a build error rather than a blank page.

```rust
#[derive(Deserialize)]
pub struct Cv {
    pub site: Profile,
    pub contact: Contact,
    pub roles: Vec<Role>,
    pub skills: Vec<Skill>,
    pub education: Vec<Education>,
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

impl Role {
    /// "September 2024 – February 2026", or "… – present" while open.
    pub fn period(&self) -> String {
        let fmt = |m: &str| {
            NaiveDate::parse_from_str(&format!("{m}-01"), "%Y-%m-%d")
                .map(|d| d.format("%B %Y").to_string())
                .unwrap_or_else(|_| m.to_string())
        };
        match &self.end {
            Some(end) => format!("{} – {}", fmt(&self.start), fmt(end)),
            None => format!("{} – present", fmt(&self.start)),
        }
    }
}
```

Add `Contact`, `Skill` and `Education` in the same shape, and give `Education` its own `period()`.

- [ ] **Step 3: Update `templates/index.html` and `about.html` for the renamed field**

Both currently loop `cv.timeline` and read `job.month()`. Change to `cv.roles` and `role.period()`. Do not add achievements to these pages — those belong on `/cv`.

- [ ] **Step 4: Build and verify the dates**

```bash
cargo run --release
grep -o 'September 2024 – February 2026' public/index.html
```

Confirm every date on the rendered page matches `cv-source.md`, and that no role shows "present".

- [ ] **Step 5: Commit**

```bash
git add content/cv.toml src/main.rs templates
git commit -m "feat(cv): carry the full CV in cv.toml, corrected against the source"
```

---

### Task 10: `/cv` and `/about`, availability, footer links

**Files:**
- Create: `templates/cv.html`
- Modify: `templates/about.html`, `templates/index.html`, `templates/base.html`, `src/main.rs`, `static/site.css`

**Interfaces:**
- Consumes: `Cv` from Task 9
- Produces: `public/cv/index.html`; `/about` reduced to a bio

- [ ] **Step 1: Write `templates/cv.html`**

A single-column document: name and contact, then roles with their achievements as `<ul>`, then skills, then education. Use `<article>` per role with an `<h3>` heading, and real `<time>` elements. No site chrome beyond what `base.html` provides — the print stylesheet in Task 11 will hide that.

- [ ] **Step 2: Add the `CvPage` struct and write in `src/main.rs`**

```rust
#[derive(Template)]
#[template(path = "cv.html")]
struct CvPage<'a> {
    cv: &'a Cv,
    year: i32,
}
```

```rust
    write(
        format!("{OUT}/cv/index.html"),
        &CvPage { cv: &cv, year }.render()?,
    )?;
```

- [ ] **Step 3: Reduce `/about` to a bio and link the CV**

`/about` must not duplicate `/cv`. It carries the intro from `cv-source.md`, a sentence or two of context, and a prominent link to `/cv/`. Remove the role list from it.

Draft the bio from the intro in `cv-source.md` plus the voice of the existing blog posts — plain, measured, specific. **Flag it in your report as needing Jonas's rewrite**; do not present invented biography as his.

- [ ] **Step 4: Add the availability line to `templates/index.html`**

Under the stack line, when `cv.site.available` is true:

```html
{% if cv.site.available %}
  <p class="available">
    {{ cv.site.available_note }} —
    <a href="{{ cv.site.links.email }}">get in touch</a>
  </p>
{% endif %}
```

- [ ] **Step 5: Add project links to the footer in `templates/base.html`**

```html
<nav class="site-foot-links" aria-label="Projects">
  <a href="https://athletos.app">AthletOS</a>
</nav>
```

- [ ] **Step 6: Add `/cv` to the nav, the sitemap and the URL guard**

Add a nav link, add `<url><loc>{{ base }}/cv/</loc></url>` to `templates/sitemap.xml`, and add `cv/index` to the required-URL list in `.github/workflows/deploy.yml`.

- [ ] **Step 7: Verify**

```bash
cargo run --release
for u in index about/index cv/index blog/index blog/trongate/index \
         blog/trongate/mx-transition/index blog/freebsd-on-hetzner/index \
         blog/athletos-freebsd/index; do
  test -f "public/$u.html" && echo "OK   $u" || echo "MISS $u"
done
```

Expected: 8 `OK`, no `MISS`.

- [ ] **Step 8: Commit**

```bash
git add templates src/main.rs static/site.css .github
git commit -m "feat: /cv page, bio-only /about, availability line and footer links"
```

---

### Task 11: Print stylesheet and PDF export

**Files:**
- Modify: `static/site.css`, `.github/workflows/deploy.yml`, `README.md`

- [ ] **Step 1: Add print rules to `static/site.css`**

```css
@media print {
  .site-head,
  .site-foot,
  .skip-link,
  .theme-toggle {
    display: none;
  }

  /* Print is always the light palette; a dark CV wastes toner and reads badly. */
  :root {
    color-scheme: light;
  }

  body {
    font-size: 10.5pt;
    line-height: 1.45;
  }

  .wrap {
    max-width: none;
    padding: 0;
  }

  a {
    color: inherit;
    text-decoration: none;
  }

  /* Roles must not be split across a page break mid-entry. */
  .cv-role {
    break-inside: avoid;
  }

  @page {
    margin: 18mm 16mm;
  }
}
```

- [ ] **Step 2: Verify the print layout before automating it**

Render `/cv/` to PDF locally with headless Chrome and open the result:

```bash
cargo run --release
cd public && python -m http.server 8123 &
# then, from the repo root:
chrome --headless --disable-gpu --print-to-pdf=cv.pdf --no-pdf-header-footer \
  http://127.0.0.1:8123/cv/
```

Confirm: no site nav or footer, no theme toggle, light palette, sensible margins, and no role split across a page boundary. Report the page count. **Stop the server afterwards** — a stray one holding `public/` open breaks the next build.

- [ ] **Step 3: Generate `/cv.pdf` in CI**

Add a step to `.github/workflows/deploy.yml`, after the build and before the guards:

```yaml
      - name: Render the CV to PDF
        run: |
          npx --yes serve -s public -l 8123 &
          npx --yes wait-on http://127.0.0.1:8123/cv/
          google-chrome --headless --disable-gpu --no-sandbox \
            --print-to-pdf=public/cv.pdf --no-pdf-header-footer \
            http://127.0.0.1:8123/cv/
          test -s public/cv.pdf
```

`ubuntu-latest` ships Chrome preinstalled. `test -s` fails the build if the PDF is empty, so a broken render cannot deploy silently.

- [ ] **Step 4: Link the PDF from `/cv`**

Add a download link near the top of `templates/cv.html`:

```html
<p class="cv-download"><a href="/cv.pdf">Download as PDF</a></p>
```

Hide it in print — a printed page should not advertise its own download:

```css
@media print {
  .cv-download { display: none; }
}
```

- [ ] **Step 5: Document it in `README.md`**

Add a short section explaining that `/cv/` is the source and `cv.pdf` is generated from it in CI, so the CV is edited in `content/cv.toml` and never in a PDF editor.

- [ ] **Step 6: Commit**

```bash
git add static/site.css .github/workflows/deploy.yml templates/cv.html README.md
git commit -m "feat(cv): print stylesheet and CI-generated cv.pdf"
```

---

## Definition of done

- `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test` all pass
- All eight URLs resolve; no `<img>` remains pointing at a local `.svg`
- Toggle persists across navigation, does not flash, and is keyboard operable
- With JS disabled the site renders fully and follows the OS theme
- Code blocks and diagrams both change with the toggle
- Every date on `/cv` matches `docs/reference/cv-source.md`
- `cv.pdf` generates, is non-empty, and carries no site chrome
- Contrast recomputed for both themes with no regression below AA
