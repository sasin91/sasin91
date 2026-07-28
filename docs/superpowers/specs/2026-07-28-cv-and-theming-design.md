# sasin91.xyz v2 — CV, theming, and accessibility

**Date:** 2026-07-28
**Status:** approved design, ready for an implementation plan
**Builds on:** `2026-07-27-static-site-rebuild-design.md`

## Why

The site went live as a CV page and a blog. Three things changed:

1. **The timeline was wrong.** The real CV has different dates, an end date for
   the current role, per-role achievements, skills and education that the site
   never carried.
2. **Jonas is job-seeking as of February 2026.** The site is now a hiring
   surface, and should say so.
3. **A design audit found real defects** — no focus styles anywhere, and a
   figure-ground inversion where diagrams outshout the code they illustrate.

Plus one long-standing want: a PDF CV export.

## Decisions

### The zero-JavaScript rule is relaxed to minimal JavaScript

v1's hard constraint was zero JS. That now blocks a theme toggle, which is
wanted. The rule becomes: **no JS is required to read anything.** Script may
enhance, never gate. Concretely — the theme toggle is the only script on the
site, it is inline and under ~20 lines, and with JS disabled the site still
renders and follows the OS preference.

### Structure

| URL | Content |
|---|---|
| `/` | Intro, availability line, recent posts |
| `/about/` | Short and human — who he is, what he cares about, link to the CV |
| `/cv/` | The full CV: roles with achievements, skills, education, contact. Print-styled |
| `/cv.pdf` | Generated in CI from `/cv/` |

`/about` deliberately does not duplicate `/cv`. One is a person, the other is a
record.

### `content/cv.toml` becomes the single source of truth

It currently holds six one-line timeline entries. It grows to carry everything
the CV has: contact block, intro, roles with bullet achievements and end dates,
skills, education. Every surface — the homepage timeline, `/about`, `/cv`, and
the PDF — renders from it. Nothing is written twice.

Corrections from the real CV, which is authoritative:

| Role | Was on site | Actual |
|---|---|---|
| GHC Travel | from Feb 2017 | Jan 2017 – Feb 2020 |
| Syncronet | from Feb 2020 | Apr 2020 – Jan 2023 |
| JUICE | Jan 2023 | Jan 2023 – Aug 2023 |
| Supeo | Sep 2023 | Sep 2023 – Sep 2024 |
| JUICE | Sep 2024, open-ended | Sep 2024 – **Feb 2026** |

Education is new to the site: Strøm, styring & IT (Selandia CEU, Slagelse),
Jan 2012 – Aug 2013, including Cisco CCNA; and Web integrator (Roskilde Teknisk
Skole, Høng), Feb 2014 – Aug 2015.

Contact — email and phone are published deliberately; Jonas judged them already
findable and the harm negligible. Street address is not published; town is.

### Availability

A short line on the homepage stating he is available for work, with a mail
link. The timeline simply ends at February 2026 and carries no commentary.

### The stack line is rewritten

The current line — `PHP · Laravel · React · TypeScript · MySQL · Docker` —
predates several years of work. It is rewritten from the CV to reflect what he
has actually shipped, and Jonas edits the result.

### Theme toggle

Default is the OS preference. A control switches light/dark and the choice
persists in `localStorage`. An inline script in `<head>` applies the stored
choice before first paint, so there is no flash of the wrong theme. CSS keys
off `:root[data-theme=...]` with the existing `prefers-color-scheme` media
query as the no-JS fallback.

The control is a real `<button>` with an accessible name and `aria-pressed`,
reachable by keyboard, never a bare styled `<div>`.

A `:target`/hash-based CSS-only toggle was considered and rejected: the site
already publishes eight heading anchors (`#What-went-wrong` and friends), so a
theme hash would collide with them — following a heading link would reset the
theme, and setting the theme would scroll to a phantom element. It also would
not survive navigation to another page.

Separately, `@view-transition { navigation: auto; }` is added for smooth
cross-document navigation. It needs no JavaScript, degrades silently where
unsupported, and is orthogonal to the toggle — one animates, the other
persists.

### SVG diagrams are inlined

**This is forced by the toggle.** An SVG referenced via `<img>` is a separate
document and cannot see the parent's `data-theme` — it can only read
`prefers-color-scheme`. With a toggle, that means toggling against your OS
setting turns the page dark and leaves the diagrams light.

So the generator inlines local `.svg` images into the HTML, where they inherit
the theme like any other element. Four files, 2.7–5.6 KB each, on two posts;
it also removes four requests.

Within each SVG, only the **chrome** adapts — panel background, label text,
rules — driven by CSS custom properties. The **data** colours stay fixed: the
orange ARC bar, blue Postgres, purple SvelteKit and so on encode categories,
and a category that changes colour with the theme breaks the reader's mapping
between legend and mark.

### Accessibility fixes

From the audit, with measured contrast ratios:

- **Focus styles** — currently absent entirely; keyboard users get only the
  browser default, worsened by `text-decoration: none` on nav links. A visible
  ring on every interactive element, in both themes.
- **Code blocks get a region in light mode.** Solarized cream `#fdf6e3` on a
  `#fdfdfb` page is 1.06:1 — the block does not read as an object. A border and
  a slightly stronger surface fix the figure-ground inversion the audit found.
- **Dark media is framed consistently** so the Trongate logo (38% of its
  opaque pixels are near-white, and it vanishes on the light page) and the
  diagrams read as one deliberate family rather than stray black slabs.
- **ARIA and tab order** reviewed across every template: landmarks, the skip
  link, the toggle's state, and inline SVGs carrying `role="img"` with their
  alt text as an accessible name.

Contrast already passing and to be preserved: body text 17.28:1, links 6.55:1
plus underline, callout text 16.5:1.

### PDF export

`/cv/` carries a print stylesheet that strips site chrome and sets page margins.
Ctrl+P already produces a clean PDF from that. CI then runs headless Chrome over
the same page to emit `/cv.pdf`, so the download and the web page share one
layout and one source of truth. No PDF layout engine, no second implementation.

### Footer

Gains project links, currently just AthletOS. A dropdown was considered and
rejected: a menu holding one item adds a click, a keyboard trap and JS to hide
a single destination. Revisit at five or more.

## Scope

**Out:** a `/projects` page (still premature at one project), per-section CV
sub-pages, intrinsic image dimensions, an integration test for the generator.

**Not listed as a role:** the period from February 2026 is job-seeking, not
independent work, and is not given a CV entry.

## Verification

- Contrast ratios recomputed for both themes; no regression below AA.
- Every interactive element reachable and visibly focused by keyboard alone.
- Toggle works, persists across navigation, and does not flash on load.
- With JS disabled, the site renders fully and follows the OS theme.
- Inlined SVGs change with the toggle, not just with the OS setting.
- `/cv.pdf` generates in CI and matches `/cv/`.
- All existing URLs still resolve; still zero blocking scripts.

## Note for Jonas, not a code change

The source CV reads "Copenhangen" in two places.
