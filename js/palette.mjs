// The site-wide search palette: Ctrl/⌘+K, Ctrl+Space or "/" on any page.
//
// Written to /search/palette.js by `build_search` in src/main.rs, directly
// after retrieval.mjs in the same module and after first lines declaring
// SEARCH_INDEX_URL and AI_ARTICLE_URL (from content/ai/article.toml), so prepareIndex, rank, resultUrl, fetchIndex and
// fetchModel are in scope here without an import. base.html imports it on
// first use and calls open().
//
// It searches the blog posts and the AI article's knowledge nodes with the
// same hybrid ranking as /blog/ai/ask/, in the reader's browser. Word matches
// show as soon as the index has loaded; ranking by meaning joins in once the
// embedding model has arrived, unless the reader has asked to save data.

/* global SEARCH_INDEX_URL, AI_ARTICLE_URL */

// Places on the site, shown before anything is typed and matched by title
// while typing. A command palette's "go to" list, not search results.
const COMMANDS = [
  { title: "Home", url: "/" },
  { title: "Writing", hint: "Every post", url: "/blog/" },
  { title: "How modern language models work", hint: "The AI article", url: AI_ARTICLE_URL },
  { title: "Timeline", hint: "The AI article, one line per idea", url: `${AI_ARTICLE_URL}timeline/` },
  { title: "Ask this article", hint: "Retrieval over the AI article", url: `${AI_ARTICLE_URL}ask/` },
  { title: "About", url: "/about/" },
  { title: "CV", url: "/cv/" },
  { title: "Download CV", hint: "PDF", url: "/cv_jonas_hansen_software_developer.pdf" },
  {
    title: "Toggle dark theme",
    hint: "Switch between light and dark",
    run: () => document.getElementById("theme-toggle")?.click(),
  },
];

const RESULT_LIMIT = 8;

// Results scoring below this fraction of the best one are left out. Ranking
// always produces a full list, and in a palette the tail of it is noise: for
// "MCP" the MCP section scores about 1.0 and the next best about 0.35.
const RELATIVE_CUTOFF = 0.65;

let ui = null;
// modelLoading is true only while the model download is in flight.
const search = { index: null, model: null, loading: null, failed: false, modelLoading: false };

export function open() {
  ui ??= build();
  if (ui.dialog.open) return;
  ui.dialog.showModal();
  ui.input.select();
  load();
  update();
}

// ---------- loading ----------

function saveData() {
  return Boolean(navigator.connection?.saveData);
}

function load() {
  search.loading ??= (async () => {
    try {
      search.index = prepareIndex(await fetchIndex(SEARCH_INDEX_URL));
    } catch {
      search.failed = true;
      search.loading = null; // try the network again next time
      update();
      return;
    }
    update();
    if (saveData()) return;
    search.modelLoading = true;
    update();
    try {
      search.model = await fetchModel(search.index.model);
    } catch {
      // Word matching keeps working; status() says why meaning is missing.
    }
    search.modelLoading = false;
    update();
  })();
  return search.loading;
}

// ---------- the dialog ----------

function element(tag, attributes = {}, ...children) {
  const node = document.createElement(tag);
  for (const [name, value] of Object.entries(attributes)) node.setAttribute(name, value);
  node.append(...children);
  return node;
}

function build() {
  const input = element("input", {
    type: "text",
    role: "combobox",
    "aria-label": "Search the site",
    "aria-autocomplete": "list",
    "aria-controls": "palette-list",
    "aria-expanded": "true",
    autocomplete: "off",
    spellcheck: "false",
    placeholder: "Search posts and the AI article…",
  });
  const list = element("div", { id: "palette-list", role: "listbox", "aria-label": "Results", class: "palette-list" });
  const empty = element("p", { class: "palette-empty", hidden: "" });
  const status = element("span", { role: "status", class: "palette-status" });
  const keys = element(
    "span",
    { class: "palette-keys", "aria-hidden": "true" },
    element("kbd", {}, "↑"),
    element("kbd", {}, "↓"),
    " move ",
    element("kbd", {}, "↵"),
    " open ",
    element("kbd", {}, "esc"),
    " close",
  );

  const icon = document.createElementNS("http://www.w3.org/2000/svg", "svg");
  icon.setAttribute("viewBox", "0 0 20 20");
  icon.setAttribute("aria-hidden", "true");
  icon.innerHTML =
    '<path fill="currentColor" fill-rule="evenodd" clip-rule="evenodd" d="M9 3.5a5.5 5.5 0 1 0 0 11 5.5 5.5 0 0 0 0-11ZM2 9a7 7 0 1 1 12.452 4.391l3.328 3.329a.75.75 0 1 1-1.06 1.06l-3.329-3.328A7 7 0 0 1 2 9Z"/>';

  const dialog = element(
    "dialog",
    { class: "palette", "aria-label": "Search the site" },
    element("div", { class: "palette-search" }, icon, input),
    list,
    empty,
    element("div", { class: "palette-foot" }, status, keys),
  );
  document.body.append(dialog);

  const state = { dialog, input, list, empty, status, items: [], active: 0, statusTimer: 0 };

  let pending = 0;
  input.addEventListener("input", () => {
    clearTimeout(pending);
    pending = setTimeout(update, 60);
  });

  input.addEventListener("keydown", (event) => {
    if (event.key === "ArrowDown" || event.key === "ArrowUp") {
      event.preventDefault();
      move(event.key === "ArrowDown" ? 1 : -1);
    } else if (event.key === "Home" && event.ctrlKey) {
      setActive(0);
    } else if (event.key === "End" && event.ctrlKey) {
      setActive(state.items.length - 1);
    } else if (event.key === "Enter") {
      event.preventDefault();
      activate(state.items[state.active], event.ctrlKey || event.metaKey);
    }
  });

  // The same shortcuts that open the palette close it.
  dialog.addEventListener("keydown", (event) => {
    const toggle =
      ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "k") ||
      ((event.ctrlKey || event.altKey) && event.code === "Space");
    if (toggle) {
      event.preventDefault();
      dialog.close();
    }
  });

  // A click on the dialog itself, not its contents, is a click on the
  // backdrop.
  dialog.addEventListener("click", (event) => {
    if (event.target === dialog) dialog.close();
  });

  return state;
}

function setActive(i) {
  const { items, input } = ui;
  if (!items.length) {
    input.removeAttribute("aria-activedescendant");
    return;
  }
  ui.active = (i + items.length) % items.length;
  items.forEach((item, n) => item.option.setAttribute("aria-selected", String(n === ui.active)));
  const option = items[ui.active].option;
  input.setAttribute("aria-activedescendant", option.id);
  option.scrollIntoView({ block: "nearest" });
}

function move(step) {
  setActive(ui.active + step);
}

function activate(item, newTab = false) {
  if (!item) return;
  if (item.run) {
    ui.dialog.close();
    item.run();
  } else if (newTab) {
    window.open(item.url, "_blank", "noopener");
  } else {
    ui.dialog.close();
    location.href = item.url;
  }
}

// ---------- results ----------

function matchingCommands(query) {
  if (!query) return COMMANDS;
  const q = query.toLowerCase();
  return COMMANDS.filter((c) => `${c.title} ${c.hint ?? ""}`.toLowerCase().includes(q)).slice(0, 3);
}

function contentResults(query) {
  if (!query || !search.index) return [];
  const vector = search.model ? search.model.encode(query).vector : null;
  const ranked = rank(search.index, query, vector, RESULT_LIMIT);
  const floor = (ranked[0]?.score ?? 0) * RELATIVE_CUTOFF;
  return ranked.filter((r) => r.score >= floor).map((result) => {
    const heading = result.passage?.heading;
    const context = result.node.kind === "post" && heading ? `${result.node.context} › ${heading}` : result.node.context;
    return {
      title: result.node.title,
      context,
      snippet: result.passage?.snippet ?? result.node.summary,
      url: resultUrl(result),
    };
  });
}

function status(query, count) {
  if (search.failed) return "The search index could not be loaded. Pages are still listed.";
  if (!query) return "Type to search posts and the AI article. Everything runs on this device.";
  if (!search.index) return "Loading the search index…";
  const found = count ? `${count} result${count === 1 ? "" : "s"}` : "No results";
  if (search.model) return `${found}, ranked by meaning and matching words.`;
  if (saveData()) return `${found}, by matching words (data saver is on, so the meaning model was not loaded).`;
  if (search.modelLoading) return `${found} by matching words; the meaning model is loading.`;
  return `${found}, by matching words only (the meaning model could not be loaded).`;
}

function update() {
  if (!ui) return;
  const query = ui.input.value.trim();
  const commands = matchingCommands(query);
  const results = contentResults(query);

  const groups = [];
  if (results.length) groups.push(["Results", results]);
  if (commands.length) groups.push([query ? "Pages" : "Go to", commands]);

  ui.items = [];
  const fragments = groups.map(([label, entries], g) => {
    const labelId = `palette-group-${g}`;
    const group = element(
      "div",
      { role: "group", "aria-labelledby": labelId, class: "palette-group" },
      element("p", { id: labelId, class: "palette-group-label" }, label),
    );
    for (const entry of entries) {
      const n = ui.items.length;
      const option = element(
        "div",
        { role: "option", id: `palette-option-${n}`, class: "palette-option", "aria-selected": "false" },
        element("span", { class: "palette-title" }, entry.title),
        element("span", { class: "palette-context" }, entry.context ?? entry.hint ?? ""),
      );
      if (entry.snippet) option.append(element("span", { class: "palette-snippet" }, entry.snippet));
      option.addEventListener("mousemove", () => {
        if (ui.active !== n) setActive(n);
      });
      option.addEventListener("click", (event) => activate(entry, event.ctrlKey || event.metaKey));
      ui.items.push({ ...entry, option });
      group.append(option);
    }
    return group;
  });

  ui.list.replaceChildren(...fragments);
  ui.empty.hidden = ui.items.length > 0;
  ui.empty.textContent = query ? `Nothing found for “${query}”.` : "";
  setActive(0);

  // The status is a live region: settle before announcing, so a screen
  // reader hears the result of a query, not of every keystroke.
  clearTimeout(ui.statusTimer);
  const message = status(query, results.length);
  ui.statusTimer = setTimeout(() => (ui.status.textContent = message), 400);
}
