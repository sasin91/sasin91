// Drives the Ask form: on /blog/ai/ask/, and the box at the top of every post
// (where it arrives as /search/ask.js, imported by the loader in base.html). Inlined directly after retrieval.mjs into the
// same inline module script (see ASK_SCRIPT in src/main.rs), so
// prepareIndex, rank, fetchIndex and fetchModel are in scope here without an
// import.
// Never write the word "script" inside angle brackets in either file: the
// HTML parser would end the inline script there.
//
// Nothing is downloaded until the reader focuses the question field or picks
// an example: the article and timeline never load any of this, and the Ask
// page itself renders without it.

const form = document.querySelector("form[data-ask], form[data-post-ask]");
if (form) setUp(form);

function setUp(form) {
  const input = form.querySelector("#ask-query");
  const submit = form.querySelector('button[type="submit"]');
  const meaning = form.querySelector("#ask-semantic");
  const status = document.getElementById("ask-status");
  const examples = document.querySelector(".ask-examples");
  const resultsSection = document.querySelector(".ask-results");
  const results = document.getElementById("ask-results");
  const steps = {
    tokens: document.getElementById("ask-step-tokens"),
    vector: document.getElementById("ask-step-vector"),
    compare: document.getElementById("ask-step-compare"),
    words: document.getElementById("ask-step-words"),
  };

  const say = (message) => (status.textContent = message);

  // The post box has no examples unless the post lists some, and no
  // meaning toggle or pipeline: those only exist on /blog/ai/ask/.
  input.disabled = submit.disabled = false;
  if (meaning) meaning.disabled = false;
  if (examples) examples.hidden = false;
  const byMeaning = () => !meaning || meaning.checked;

  let loading = null;
  const load = () =>
    (loading ??= loadEverything(form.dataset.index, form.dataset.scope, Number(form.dataset.modelBytes), say));
  input.addEventListener("focus", load, { once: true });

  let lastQuery = "";

  async function ask(query) {
    query = query.trim();
    if (!query) {
      say("Type a question first.");
      return;
    }
    lastQuery = query;
    const { index, model } = await load();
    if (!index) {
      loading = null; // let the next attempt try the network again
      return;
    }

    const encoded = model ? model.encode(query) : null;
    const vector = byMeaning() && encoded ? encoded.vector : null;
    const ranked = rank(index, query, vector);

    showResults(results, ranked, index, Boolean(vector));
    resultsSection.hidden = false;
    if (steps.tokens) showPipeline(steps, index, encoded, vector, ranked, model, byMeaning());

    const how = vector
      ? "by meaning and matching words"
      : model
        ? "by matching words only"
        : "by matching words only, because the embedding model could not be loaded";
    say(
      ranked.length
        ? `${ranked.length} sections found ${how}.`
        : "No section matched those words. Try describing the idea differently.",
    );
  }

  form.addEventListener("submit", (event) => {
    event.preventDefault();
    ask(input.value);
  });

  for (const button of examples?.querySelectorAll("button") ?? []) {
    button.addEventListener("click", () => {
      input.value = button.textContent;
      ask(input.value);
    });
  }

  meaning?.addEventListener("change", () => {
    if (lastQuery) ask(lastQuery);
  });
}

/**
 * Fetches the index, then the model. The index is small and needed for
 * any search; the model is the large part, and if it fails the page still
 * works by keyword matching. Resolves to { index, model }, either of which
 * may be null.
 */
async function loadEverything(indexUrl, scope, modelBytes, say) {
  let index = null;
  try {
    say("Loading the search index…");
    // The index covers the whole site; this page asks one document only.
    index = prepareIndex(scoped(await fetchIndex(indexUrl), scope));
  } catch {
    say("The search index could not be loaded. Please try again later.");
    return { index: null, model: null };
  }

  try {
    const megabytes = (modelBytes / 1e6).toFixed(1);
    say(`Loading the embedding model (${megabytes} MB, downloaded once)…`);
    const model = await fetchModel(index.model);
    say("Ready. Searching happens on this device.");
    return { index, model };
  } catch {
    say("The embedding model could not be loaded, so results use matching words only.");
    return { index, model: null };
  }
}

/**
 * The documents an Ask page ranks: "ai" is the article's knowledge nodes,
 * "post:<path>" the sections of that post (written by src/search_index.rs
 * for posts with `ask` questions).
 */
function scoped(index, scope) {
  if (scope.startsWith("post:")) {
    const post = index.nodes.find((n) => n.kind === "post" && n.id === scope.slice(5));
    if (!post?.sections) throw new Error(`no sections for ${scope}`);
    return { ...index, nodes: post.sections };
  }
  return { ...index, nodes: index.nodes.filter((n) => n.kind === scope) };
}

// ---------- rendering ----------

function element(tag, attributes = {}, ...children) {
  const node = document.createElement(tag);
  for (const [name, value] of Object.entries(attributes)) node.setAttribute(name, value);
  node.append(...children);
  return node;
}

function ordinal(n) {
  const suffix = n % 100 >= 11 && n % 100 <= 13 ? "th" : { 1: "st", 2: "nd", 3: "rd" }[n % 10] ?? "th";
  return `${n}${suffix}`;
}

const FIELD_NAMES = {
  title: "title",
  concept: "concept",
  term: "search term",
  keyword: "keyword",
  text: "text",
};

function describeMatches(matched) {
  if (!matched.length) return "No exact words in common; this is a match by meaning only.";
  return matched.map((m) => `“${m.text}” (${FIELD_NAMES[m.field]})`).join(", ");
}

function showResults(list, ranked, index, semantic) {
  const total = index.nodes.length;
  list.replaceChildren(
    ...ranked.map((result) => {
      const why = element("dl");
      if (semantic) {
        why.append(
          element("dt", {}, "Meaning"),
          element(
            "dd",
            {},
            `${ordinal(result.semanticRank)} closest of ${total} sections. Closest passage: `,
            element("q", {}, result.passage.snippet),
          ),
        );
      }
      why.append(
        element("dt", {}, "Words"),
        element("dd", {}, describeMatches(result.matched)),
        element("dt", {}, "Numbers"),
        element(
          "dd",
          {},
          (semantic ? `cosine similarity ${result.semantic.toFixed(2)}, ` : "") +
            `word score ${result.lexical.toFixed(2)}, combined ${result.score.toFixed(2)}. ` +
            "These only order the results; they are not a confidence.",
        ),
      );

      return element(
        "li",
        {},
        element("h3", {}, element("a", { href: result.node.url }, result.node.title)),
        element("p", {}, result.node.summary),
        element("details", {}, element("summary", {}, "Why this matched"), why),
      );
    }),
  );
}

function showPipeline(steps, index, encoded, vector, ranked, model, wantedMeaning) {
  if (encoded) {
    const tokens = element("span", { class: "pipeline-tokens" });
    for (const token of encoded.tokens) {
      tokens.append(element("span", { class: token === "[UNK]" ? "pipeline-token is-unknown" : "pipeline-token" }, token));
    }
    const note = encoded.unknown ? `, ${encoded.unknown} not in the vocabulary and skipped` : "";
    steps.tokens.replaceChildren(`${encoded.tokens.length} tokens${note}: `, tokens);
  } else {
    steps.tokens.textContent = "Skipped: the model is not loaded.";
  }

  if (vector) {
    // The first few dimensions as bars: enough to show "a list of numbers",
    // with no pretence that any single dimension means something.
    const shown = Array.from(vector.slice(0, 32));
    const peak = Math.max(...shown.map(Math.abs)) || 1;
    const bars = element("span", { class: "pipeline-bars", "aria-hidden": "true" });
    for (const v of shown) {
      const bar = element("span", { class: v < 0 ? "is-negative" : "" });
      bar.style.height = `${Math.max(4, (Math.abs(v) / peak) * 100)}%`;
      bars.append(bar);
    }
    const preview = shown.slice(0, 3).map((v) => v.toFixed(3)).join(", ");
    steps.vector.replaceChildren(bars, `${vector.length} numbers, starting ${preview}, …`);
  } else {
    steps.vector.textContent = !model
      ? "Skipped: the model is not loaded."
      : !wantedMeaning
        ? "Skipped: ranking by meaning is switched off."
        : "Skipped: none of the question's tokens are in the vocabulary.";
  }

  const passages = index.nodes.reduce((sum, node) => sum + node.passages.length, 0);
  if (vector && ranked.length) {
    const best = [...ranked].sort((a, b) => a.semanticRank - b.semanticRank)[0];
    steps.compare.textContent =
      `Compared with ${passages} passage vectors from ${index.nodes.length} sections. ` +
      `Closest in meaning among the results: “${best.node.title}”.`;
  } else {
    steps.compare.textContent = "Skipped: ranking by matching words only.";
  }

  const matched = new Set(ranked.flatMap((r) => r.matched.map((m) => m.text)));
  steps.words.textContent = matched.size
    ? `Matched: ${[...matched].map((t) => `“${t}”`).join(", ")}.`
    : "No exact words or phrases matched.";
}
