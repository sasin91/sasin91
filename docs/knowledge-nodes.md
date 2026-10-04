# Knowledge nodes: the AI article

`/blog/ai/` is not written as one long file. It is assembled from small *knowledge
nodes*, one Djot file each, and the same nodes produce three views:

| URL | View | Built from |
| --- | --- | --- |
| `/blog/ai/` | the long-form article | every node's body, in article order |
| `/blog/ai/timeline/` | one line per idea | every node's `title` and `summary` |
| `/blog/ai/ask/` | retrieval, in the browser | a search index generated from the nodes |

Every node has a stable anchor, so `/blog/ai/#kv-cache` links to that section from
anywhere: other posts, search results, the timeline, Ask.

```
content/ai/
  article.toml        title, parts (the reading order), learning paths
  nodes/<id>.dj       one node per file
```

## A node

```toml
+++
id = "kv-cache"                     # = file name = URL anchor. Never rename casually.
title = "KV cache"
summary = "During generation, keys and values ... only computes its own."
concepts = ["kv cache", "key-value cache", "prefix caching"]
prerequisites = ["qkv", "context-window"]   # read these first
related = ["discarded-candidates"]          # see also, either direction
search_terms = ["kv-cache", "prompt caching", "why is generation slow"]
learning_objectives = ["Explain what the KV cache stores ..."]
misconceptions = ["The KV cache stores alternative completions the model considered."]
+++

Body in Djot, like a blog post: links, `code`, ```` ``` ```` blocks, math with
$`...`, local SVG diagrams.
```

| Field | Required | Used for |
| --- | --- | --- |
| `id` | yes | anchor, file name, every reference to the node. Lowercase letters, digits, single hyphens. |
| `title` | yes | the node's heading, timeline entry, Ask result |
| `summary` | yes | the timeline line and Ask result text; at most 200 characters |
| `concepts` | yes, ≥ 1 | the ideas the node explains, in a reader's words. Ask matches them as exact phrases. |
| `prerequisites` | no | "Builds on" links under the node. Each must come *earlier* in the article. |
| `related` | no | "See also" links. Any direction. |
| `search_terms` | no | synonyms, acronyms and likely questions, for Ask only. Never shown. |
| `learning_objectives` | no | not shown yet; for guided lessons |
| `misconceptions` | no | shown under the node as "Often misread as". Write the *wrong* belief, as a reader would hold it, and make sure the body corrects it. |

Unknown fields are an error, so a misspelt `prerequisite` cannot silently mean
"none".

## Adding a node

1. Create `content/ai/nodes/<id>.dj` with the frontmatter above.
2. Add its id to a `[[part]]` in `content/ai/article.toml`, where it should be
   read.
3. `cargo run` and check `/blog/ai/#<id>`, `/blog/ai/timeline/` and a question in
   `/blog/ai/ask/` that should find it.

## Editing

- Bodies are Djot, but **no headings**: the node's title is its heading. A
  node that needs subheadings is two nodes.
- Link to another node with `[text](#other-id)`. Links to anchors that do not
  exist fail the build.
- Write the body to be readable on its own. Someone arriving from Ask or a
  deep link has not read the node before it.

## Reordering

The article's order is the order ids appear in the `[[part]]` lists. Move the
id. Nothing in the node changes. The build fails if a node now appears before
one of its `prerequisites`; either move the prerequisite too, or change it to
`related`.

## Linking from elsewhere

`/blog/ai/#<id>` from anywhere on the site or off it. Because the id is the
contract, renaming a node breaks inbound links; prefer changing the title.

## What the build checks

`cargo run` and `cargo test` (`src/knowledge.rs`) both refuse content where:

- two files share an id, or an id does not match its file name;
- an id is not a safe anchor, or collides with other ids on the page;
- a `prerequisites` or `related` id does not exist, points at the node itself,
  or is listed twice, or one id is in both lists;
- a prerequisite comes later in the article than the node that needs it;
- a node is in no part, or in more than one place in the article;
- a body contains a heading, or links to a `#anchor` that is not a node;
- a summary is empty or over 200 characters, or `concepts` is empty;
- a learning path is duplicated, empty, or names a node that does not exist.

All problems are reported at once.

## How Ask finds things

Ask is retrieval, not a chatbot, and it runs entirely in the reader's
browser: no API, no key, and the question never leaves the page.

At build time (`src/search_index.rs`) each node is split into passages: one
for its metadata (title, summary, concepts, search terms) and one per
paragraph or list of its body. Each passage is embedded with a static
embedding model (`models/README.md`), and the vectors are written with the
node metadata and a short snippet to `public/search/index.<hash>.json`. Blog
posts go into the same index, split the same way, with each passage
remembering the heading it sits under; Ask filters the index to the article's
nodes, while the site-wide palette below uses all of it.

Every post opens with an Ask box too. The build indexes each of a post's
sections as a document, nested under the post so the palette still finds the
post once, and the box ranks only those. The loader in `base.html` imports
`/search/ask.js` (the same script /blog/ai/ask/ inlines) on pages with the
box. A post may suggest questions in its frontmatter (`ask = ["...", ...]`).

In the browser (`js/retrieval.mjs`), only once the reader focuses the
question field, the page fetches that index and the model, then for each
question:

1. tokenizes it with the same WordPiece rules, and averages the tokens'
   vectors into one;
2. takes, for each node, the best cosine similarity among its passages;
3. adds a lexical score from exact phrase matches against title, concepts and
   search terms, and single-word matches against the text, weighted by how
   rare the word is;
4. ranks by `cosine + 0.5 × lexical`.

The lexical half is what makes `KV cache`, `MCP` or `top-p` land on the right
node even when the static model's sense of meaning is vague. If the model
cannot load, ranking falls back to the lexical score alone, and the page says
so.

If a reasonable question does not find a node, the fix is usually a
`search_terms` entry with the reader's wording. If misses keep coming from
the static model itself (wrong word sense, paraphrases with no shared words),
see "When to switch to a transformer model" in `models/README.md`.

Tests: `cargo test` covers the build side, and `node --test
js/retrieval.test.mjs` the browser side, including real questions against the
built index (run `cargo run --release` first).

## The search palette

Every page has a search palette: <kbd>Ctrl</kbd>/<kbd>⌘</kbd>+<kbd>K</kbd>,
<kbd>Ctrl</kbd>+<kbd>Space</kbd>, <kbd>/</kbd>, or the Search button in the
header. (<kbd>Alt</kbd>+<kbd>Space</kbd> is listened for too, but Windows
usually takes it for the window menu before the browser sees it.)

- `templates/base.html` carries only a small inline loader. It reveals the
  button and, on first use, imports `/search/palette.js`, so no page is
  heavier for readers who never search.
- `palette.js` is written by `build_search` in `src/main.rs`:
  `js/retrieval.mjs` plus `js/palette.mjs`, with the hashed index URL
  prefixed. It searches posts and nodes with the same ranking as Ask, shows
  word matches as soon as the index arrives, and adds meaning once the model
  has loaded (skipped when the browser asks to save data).
- A post result links to the section its best passage is in
  (`/blog/x/#Section-heading`); a node result to the node.
- With nothing typed, it lists the site's pages and a theme toggle, from the
  `COMMANDS` list at the top of `js/palette.mjs`.
- Results scoring under 65% of the best one are dropped, so a precise query
  shows one or two results rather than a full list.

## Later: lessons and quizzes

The content model already leaves room for both without changing a node's
shape:

- **Lessons** are `[[learning_path]]` entries in `article.toml`: an id, a
  title and an ordered list of node ids, validated today but not rendered. A
  lesson view would render those nodes in that order, reusing the node markup
  from `templates/ai.html`, at `/blog/ai/learn/<path-id>/`.
- **Quizzes** can be built from what nodes already carry:
  `learning_objectives` as what to test, `misconceptions` as plausible wrong
  answers. Hand-written questions would be a new optional frontmatter table,
  for example `[[question]]` with `prompt`, `answer` and `explanation`, added
  to `NodeFront` as an optional field so existing nodes stay valid.
- **Progress and mastery** would live in the reader's browser, in
  localStorage or IndexedDB keyed by node id, with no accounts. Node ids are
  already the stable keys this needs, which is one more reason not to rename
  them.
- **A local generative model** answering only from retrieved nodes would sit
  after step 4 above: the ranking already returns the passages a generator
  would be given.
