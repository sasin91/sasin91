# cv.pdf without a browser — a PDF writer in the site binary

**Date:** 2026-07-28
**Status:** approved design, ready for an implementation plan
**Supersedes:** the `/cv.pdf` row in `2026-07-28-cv-and-theming-design.md`
("Generated in CI from `/cv/`")

## Why

Generating one two-page PDF currently costs the deploy job a Python web server,
a curl warm-up loop, a grep pre-check, headless Chrome, `poppler-utils`, and
four `pdftotext` assertions. Every one of those is scaffolding for a single
question: *did the browser fetch the right page?*

That question has already been answered wrong in production. The comment in
`.github/workflows/deploy.yml` records it: `npx serve -s` rewrote every
unmatched route to `/index.html`, so `/cv/` served the homepage and **cv.pdf
shipped as a snapshot of the landing page for several deploys**. Neither
`test -s public/cv.pdf` nor `test -f public/cv/index.html` could catch it. The
`pdftotext` guards exist because that class of bug is invisible to the obvious
checks.

The fidelity the machinery buys is also weaker than it looks. `static/site.css`
sets `--sans: ui-sans-serif, system-ui, -apple-system, "Segoe UI", Roboto, …`.
None of those fonts exist on `ubuntu-latest`; headless Chrome falls through to
whatever fontconfig offers, probably DejaVu Sans. **The typeface in the shipped
PDF was never chosen by anyone**, and it changes if GitHub changes the runner
image.

So the browser is being kept for a rendering nobody picked, at the cost of five
external dependencies and a bug class that needed two guards to detect.

## Decision

`cv.pdf` becomes a build artifact of the site binary, written directly from the
`Cv` struct. It gets its own deliberate layout — Helvetica, A4, chosen margins
— rather than mirroring the web page's CSS.

This is a real trade. The PDF and `/cv/` become two layouts, not one. What
stops them diverging is that both render from the same `Cv`: the *content* is
shared by construction, and only presentation differs. The earlier "one layout
to maintain" argument was defending shared presentation that, per the font
finding above, was never actually shared in any deliberate sense.

Scope is bounded: `cv.pdf` is the only PDF this site will ever produce. No
general HTML-to-PDF renderer is being built, and nothing here needs to handle
code blocks, syntax highlighting, diagrams or math.

### Rejected

**wkhtmltopdf.** Archived upstream since 2023, built on an ancient WebKit fork,
and it still needs the local server and an external binary. The cost of Chrome
with worse output.

**A box-model layout engine** — nested boxes, margin collapsing, the CSS shape.
For a document made of headings, paragraphs and bullets that is writing a
browser to avoid depending on one.

**A forced single page.** Five roles carrying eighteen achievements between
them do not fit on one A4 page at a readable size. It would mean cutting real
content to simplify the code.

## Architecture

Three modules, flat in `src/` alongside `content.rs`, `cv.rs`, `djot.rs`,
`highlight.rs`, `html.rs` and `math.rs`.

### `src/pdf_metrics.rs` — the width tables

Pure data, no logic: two `[u16; 224]` arrays of glyph widths in 1/1000 em,
indexed by WinAnsi code point minus 32. Kept in its own file because it is
machine-derived and never hand-edited; mixing it into the writer would invite
someone to "fix" a width by eye.

### `src/pdf.rs` — the writer

Knows nothing about CVs. You hand it positioned strings; it hands you bytes.

- `Font { Helvetica, HelveticaBold }`, each with
  `width(self, text: &str, size_pt: f32) -> f32` backed by the tables above.
- `Placement { x_mm, y_mm, size_pt, font, text }`, `Page { placements }`.
- `write_pdf(title, width_mm, height_mm, &[Page]) -> Vec<u8>` — object
  serialization with byte-accurate xref offsets.
- `winansi` encoding and PDF string escaping.

Text measurement is the only addition to the sample this design starts from.
Everything else is that sample unchanged.

### `src/cv_pdf.rs` — the layout

`pub fn render(cv: &Cv) -> Vec<u8>`. A cursor over A4 (210×297mm) with `18mm`
vertical and `16mm` horizontal margins — the same numbers as the `@page` rule
in `static/site.css`, so the printed measure does not change. Column width is
therefore 178mm.

The cursor wraps text greedily to the column using `Font::width`, tracks `y`,
and starts a new page when a block will not fit. Two pagination rules carry
over from the print stylesheet, expressed arithmetically:

- **A role never splits.** Its full height is measured before placement; if the
  remaining space is short, it moves to the next page. This is
  `break-inside: avoid` on `.cv-role`.
- **A heading never ends a page.** A section heading that would be the last
  thing placed moves with its first following block. This is
  `break-after: avoid` on `h2`.

Block types are few: document header, contact line, paragraph, section
heading, role entry (title, meta line, summary, bullets), and skill list.

## Data flow

```
content/cv.toml  →  Cv  →  templates/cv.html  →  public/cv/index.html
                     └──→  cv_pdf::render      →  public/cv.pdf
```

`main.rs` writes the PDF immediately after the `/cv/index.html` write, inside
the same build. There is no second process, no server, and no network.

## Error handling

`render` is infallible. Dates are validated by `Cv::validate` before the build
reaches this point — the same guarantee `Role::start_label()` already relies on
when it unwraps. The only fallible step is `fs::write`, which `main` already
handles for every other output.

The one silent-failure risk is a character outside WinAnsi degrading to `?`.
Today's content is entirely Latin-1 (`Næstved`, `Høng`, `Strøm`), but a curly
apostrophe or an em dash pasted into `content/cv.toml` later would ship as `?`
on a document used in job applications. A test asserts every string in the real
`content/cv.toml` encodes cleanly, so that fails the build instead.

## Testing

`cargo test` replaces the CI guards. The tests, in `src/pdf.rs` and
`src/cv_pdf.rs`:

**Structural** (from the reference sample) — output starts `%PDF-1.4` and ends
`%%EOF`; the catalog, page tree and font objects are present; the `startxref`
offset in the trailer points at the `xref` keyword; a multi-page document
references every page.

**Content** — every role title, company and achievement from the real
`content/cv.toml` appears in the content streams, as do "Experience", "Skills"
and "Education".

**Layout** — no laid-out line exceeds the 178mm column; no page ends on a
section heading; page count is between 1 and 3, so a runaway layout bug
surfaces as a failure rather than a forty-page PDF.

**Encoding** — every string in `content/cv.toml` is WinAnsi-representable.

The bug that motivated the `pdftotext` guards is not tested for, because it
becomes unrepresentable: `render` takes a `&Cv`. There is no URL to get wrong
and no homepage to fetch by accident.

### The AFM tables — resolved

Originally logged as the main risk here, on the assumption the widths would be
transcribed by hand. They were not. They are derived from Adobe's published
1997 Core-14 AFM metrics and cross-checked against two independent lineages —
URW's Nimbus Sans (a metrically compatible clone with a different glyph set)
and Mozilla pdf.js's separately transcribed table. All three agree on all 224
code points for both faces, with no disagreement.

The one real trap was the mapping. AFM files list `C <code> ; WX <width> ; N
<glyphname>`, where `C` is *AdobeStandardEncoding*. Sixty-six of the 224 WinAnsi
code points carry `C -1` — including every accented Latin-1 letter. Indexing by
`C` would have silently dropped `æ`, `ø` and `å`, which is to say `Næstved`,
`Høng` and `Strøm`. The tables are therefore built by resolving each WinAnsi
code point to a glyph *name* (per PDF 32000-1 Annex D.2) and looking that name
up. The encoding map itself was verified by round-tripping 0x20–0xFF through
CP-1252.

What remains is not correctness but fit: whether Helvetica at the chosen sizes
looks right. That is the comparison gate below, and it is a human's call.

## What leaves `.github/workflows/deploy.yml`

The entire `Render the CV to PDF` step, and with it:

| Removed | Was there to |
|---|---|
| `apt-get install poppler-utils` | provide `pdftotext` for the guards |
| `python3 -m http.server` | resolve `/cv/` honestly, unlike `npx serve -s` |
| the 30-iteration curl loop | wait for that server |
| the `grep -q 'Experience'` pre-check | prove the server served the CV |
| `google-chrome --headless` | render |
| four `pdftotext` assertions | prove the PDF was not the homepage |

`cv.pdf` is added to the existing required-URLs check. The deployed-site
verification already curls `/cv.pdf` and needs no change.

## Order of work

1. `src/pdf.rs` with tests.
2. `src/cv_pdf.rs` with tests.
3. Wire into `main.rs`; build locally.
4. **Comparison gate.** Put the generated `public/cv.pdf` beside the current
   live `sasin91.xyz/cv.pdf` and look at both. The new layout is unproven until
   seen.
5. Only after that holds up: edit `deploy.yml`.

If the comparison fails, two new files are deleted and nothing has shipped.
