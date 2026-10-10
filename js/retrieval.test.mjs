// Tests for js/retrieval.mjs. Run with: node --test js/
//
// No test framework beyond Node's own. The last group runs real questions
// against the built index in public/search/, so `cargo run --release` must have
// run first; CI does that (see .github/workflows/pipeline.yml), and locally
// those tests are skipped with a note if public/ is missing.

import assert from "node:assert/strict";
import { existsSync, readFileSync, readdirSync } from "node:fs";
import { test } from "node:test";

import {
  StaticModel,
  WEIGHTS,
  contentWords,
  cosine,
  prepareIndex,
  rank,
  resultUrl,
  words,
} from "./retrieval.mjs";

const MODEL_DIR = new URL("../models/potion-base-4M/", import.meta.url);

function loadModel() {
  const bytes = readFileSync(new URL("embeddings.q8.bin", MODEL_DIR));
  const buffer = bytes.buffer.slice(bytes.byteOffset, bytes.byteOffset + bytes.byteLength);
  return new StaticModel(readFileSync(new URL("vocab.txt", MODEL_DIR), "utf8"), buffer);
}

const model = loadModel();

// ---------- tokenizer and model ----------

test("tokenizes exactly like the reference Hugging Face tokenizer", () => {
  // Same fixture as src/embed.rs, so the browser and the build agree.
  const cases = JSON.parse(readFileSync(new URL("wordpiece-fixture.json", MODEL_DIR), "utf8"));
  assert.ok(cases.length >= 10);
  for (const { text, tokens } of cases) {
    assert.deepEqual(model.tokenize(text), tokens, `for ${JSON.stringify(text)}`);
  }
});

test("encodes to a unit vector of the model's dimension", () => {
  const { vector, tokens, unknown } = model.encode("What does a KV cache store?");
  assert.equal(vector.length, model.dim);
  assert.ok(Math.abs(cosine(vector, vector) - 1) < 1e-5);
  assert.deepEqual(tokens, ["what", "does", "a", "kv", "cache", "store", "?"]);
  assert.equal(unknown, 0);
});

test("drops unknown tokens instead of embedding them", () => {
  const plain = model.encode("context window");
  const noisy = model.encode("context 🚀 window");
  assert.equal(noisy.unknown, 1);
  assert.deepEqual(Array.from(noisy.vector), Array.from(plain.vector));
});

test("text with no known tokens has no vector", () => {
  assert.equal(model.encode("🚀").vector, null);
  assert.equal(model.encode("").vector, null);
});

test("rejects a weights file that does not fit the vocabulary", () => {
  assert.throws(() => new StaticModel("a\nb\nc\n", new ArrayBuffer(13)));
  assert.equal(new StaticModel("a\nb\n", new ArrayBuffer(2 * 4 + 2 * 3)).dim, 3);
});

// ---------- lexical matching ----------

test("splits words the same way src/search_index.rs does", () => {
  assert.deepEqual(
    [...new Set(words("Top-p, KV-cache and the KV cache; naïve 128k"))].sort(),
    ["128k", "and", "cache", "kv", "naïve", "p", "the", "top"],
  );
  assert.deepEqual(contentWords("What is the KV cache, and why?"), ["kv", "cache"]);
});

// A small hand-made index: three nodes, two-dimensional vectors, so the
// semantic side of each test is fully under control.
function tinyIndex() {
  const node = (id, order, vector, extra = {}) => ({
    id,
    order,
    url: `/blog/ai/#${id}`,
    title: extra.title ?? id,
    summary: extra.summary ?? "",
    concepts: extra.concepts ?? [],
    search_terms: extra.search_terms ?? [],
    words: extra.words ?? [],
    passages: [{ snippet: `${id} snippet`, vector }],
  });
  const s = Math.SQRT1_2;
  return prepareIndex({
    version: 1,
    model: { dim: 2 },
    nodes: [
      node("kv-cache", 1, [0, 1], { title: "KV cache", concepts: ["kv cache", "context"] }),
      node("vague", 2, [s, s], { title: "Something nearby", words: ["cache", "context"] }),
      node("context-window", 3, [1, 0], { title: "Context windows", concepts: ["context window", "context"] }),
    ],
  });
}

test("an exact technical term outranks a closer but vague semantic match", () => {
  const index = tinyIndex();
  // The query vector is nearly identical to "vague", and far from "kv-cache".
  const query = new Float32Array([0.8, 0.6]);
  const [first, second] = rank(index, "what is a KV cache", query);
  assert.equal(first.node.id, "kv-cache");
  assert.equal(second.node.id, "vague");
  assert.ok(second.semantic > first.semantic, "the vague node really was closer in meaning");
  assert.deepEqual(first.matched.map((m) => m.text), ["KV cache", "kv cache"]);
});

test("without a lexical match, meaning decides", () => {
  const index = tinyIndex();
  const results = rank(index, "something unrelated entirely", new Float32Array([1, 0]));
  assert.equal(results[0].node.id, "context-window");
  assert.equal(results[0].lexical, 0);
});

test("a common one-word concept counts for less than a rare one", () => {
  const index = tinyIndex();
  // "context" is a concept of two nodes and text of the third; "kv" is not a
  // phrase anywhere on its own, but "kv cache" is, in one node.
  const common = rank(index, "context", null).find((r) => r.node.id === "context-window");
  const rare = rank(index, "kv cache", null).find((r) => r.node.id === "kv-cache");
  assert.ok(rare.lexical > common.lexical, `${rare.lexical} vs ${common.lexical}`);
});

test("falls back to lexical ranking when there is no query vector", () => {
  const index = tinyIndex();
  const results = rank(index, "context window", null);
  assert.equal(results[0].node.id, "context-window");
  for (const r of results) {
    assert.equal(r.semantic, null);
    assert.equal(r.passage, null);
    assert.equal(r.semanticRank, undefined);
    assert.equal(r.score, r.lexical);
    assert.ok(r.score > 0);
  }
  assert.deepEqual(rank(index, "zebra", null), [], "nothing matched, nothing returned");
});

test("results carry the evidence the page explains", () => {
  const index = tinyIndex();
  const [result] = rank(index, "kv cache", new Float32Array([0, 1]));
  assert.deepEqual(Object.keys(result).sort(), [
    "lexical",
    "matched",
    "node",
    "passage",
    "score",
    "semantic",
    "semanticRank",
  ]);
  assert.equal(result.node.url, "/blog/ai/#kv-cache");
  assert.equal(result.passage.snippet, "kv-cache snippet");
  assert.equal(result.semanticRank, 1);
  assert.ok(Math.abs(result.score - (result.semantic + WEIGHTS.lexical * result.lexical)) < 1e-9);
});

test("ties are broken by article order, and the limit is respected", () => {
  const index = tinyIndex();
  const results = rank(index, "zebra", new Float32Array([Math.SQRT1_2, Math.SQRT1_2]), 2);
  assert.equal(results.length, 2);
  assert.equal(results[0].node.id, "vague");
});

test("a result links to the section its best passage is in", () => {
  const node = { url: "/blog/post/" };
  assert.equal(resultUrl({ node, passage: { anchor: "What-it-costs" } }), "/blog/post/#What-it-costs");
  assert.equal(resultUrl({ node, passage: { snippet: "no heading" } }), "/blog/post/");
  assert.equal(resultUrl({ node, passage: null }), "/blog/post/");
  // A node's URL already carries its own anchor.
  assert.equal(resultUrl({ node: { url: "/blog/ai/#rag" }, passage: null }), "/blog/ai/#rag");
});

// ---------- the real site ----------

const built = new URL("../public/search/", import.meta.url);
const indexFile = existsSync(built) && readdirSync(built).find((f) => /^index\.[0-9a-f]{16}\.json$/.test(f));
const skip = indexFile ? false : "public/search/ not built; run `cargo run --release` first";

function realIndex(kind = null) {
  const index = JSON.parse(readFileSync(new URL(indexFile, built), "utf8"));
  return prepareIndex(kind ? { ...index, nodes: index.nodes.filter((n) => n.kind === kind) } : index);
}

/** As /blog/ai/ask/ asks: the article's nodes only. */
function ask(question) {
  return rank(realIndex("ai"), question, model.encode(question).vector).map((r) => r.node.id);
}

/** As the Ask box at the top of a post asks: that post's sections only. */
function askPost(path, question) {
  const index = JSON.parse(readFileSync(new URL(indexFile, built), "utf8"));
  const post = index.nodes.find((n) => n.kind === "post" && n.id === path);
  const prepared = prepareIndex({ ...index, nodes: post.sections });
  return rank(prepared, question, model.encode(question).vector).map(resultUrl);
}

/** As the palette asks: the whole site. */
function searchSite(question) {
  return rank(realIndex(), question, model.encode(question).vector);
}

test("the palette finds posts as well as article sections", { skip }, () => {
  const top = (q) => searchSite(q)[0];
  assert.equal(top("FreeBSD on Hetzner Cloud").node.id, "blog/freebsd-on-hetzner");
  assert.equal(top("k3s platform kit").node.id, "blog/k3s-platform-kit");
  assert.equal(top("Trongate PHP framework").node.kind, "post");
  assert.equal(top("Trongate benchmarks").node.id, "blog/trongate/benchmarks");
  assert.equal(top("trongate.cloud CPU limit").node.id, "blog/trongate/benchmarks");
  assert.equal(top("What is stored in the KV cache?").node.id, "kv-cache");
  assert.equal(top("MCP").node.id, "mcp");
});

test("palette results deep-link into the section that matched", { skip }, () => {
  const linked = searchSite("what does the k3s cluster cost to run per month")
    .filter((r) => r.node.kind === "post")
    .map(resultUrl);
  assert.ok(linked.some((url) => url.includes("#")), linked.join(", "));
  for (const url of linked) assert.match(url, /^\/blog\/[^#]+\/(#[^#]+)?$/);

  assert.match(
    resultUrl(searchSite("is Valkey faster than files for PHP sessions")[0]),
    /^\/blog\/trongate\/benchmarks\/#/,
  );
});

test("exact technical terms find their node first", { skip }, () => {
  for (const [question, id] of [
    ["KV cache", "kv-cache"],
    ["What is stored in the KV cache?", "kv-cache"],
    ["QKV", "qkv"],
    ["MCP", "mcp"],
    ["RAG", "rag"],
    ["top-p", "sampling-controls"],
    ["temperature", "sampling-controls"],
    ["sliding window attention", "sparse-attention"],
  ]) {
    assert.equal(ask(question)[0], id, question);
  }
});

test("natural questions find the sections that answer them", { skip }, () => {
  const top3 = ask("Why doesn't a larger context window replace RAG?").slice(0, 3);
  for (const id of ["context-window", "rag", "retrieval-vs-attention"]) {
    assert.ok(top3.includes(id), `${id} in ${top3}`);
  }
  assert.equal(ask("Does the model learn from my conversation?")[0], "training");
  assert.equal(ask("Is MCP the same thing as RAG?")[0], "mcp");
  assert.equal(ask("does the model remember previous chats")[0], "external-memory");
});

test("a post's Ask box answers from that post's sections", { skip }, () => {
  const benchmarks = "/blog/trongate/benchmarks/";
  for (const [question, anchor] of [
    ["How do I deploy my app?", "You-push-we-deploy"],
    ["Do I need to manage servers?", "Nothing-to-manage"],
    ["Does it scale with me?", "Scales-with-you"],
    ["Is it built for Trongate?", "Built-for-Trongate"],
    ["Is it a proven platform?", "A-proven-platform-we-run-it-ourselves"],
    ["What did the benchmarks find?", "Benchmarks"],
    ["What is the lazy session fix?", "For-the-Trongate-team-the-session-finding"],
  ]) {
    assert.equal(askPost("blog/trongate/benchmarks", question)[0], `${benchmarks}#${anchor}`, question);
  }
  const recruiter = "/blog/trongate/recruiter/";
  for (const [question, anchor] of [
    ["What problem does it solve?", "The-problem"],
    ["How does cv_match work?", "What-cv_match-does"],
    ["Is it free?", "Free-or-with-your-own-key"],
    ["Which AI model does it use?", "Free-or-with-your-own-key"],
    ["Why build it on trongate.cloud?", "Built-on-trongatecloud"],
    ["How accurate is it?", "How-accurate-it-is"],
  ]) {
    assert.equal(askPost("blog/trongate/recruiter", question)[0], `${recruiter}#${anchor}`, question);
  }
});

test("every post's sections are in the index", { skip }, () => {
  const index = JSON.parse(readFileSync(new URL(indexFile, built), "utf8"));
  for (const post of index.nodes.filter((n) => n.kind === "post")) {
    assert.ok(post.sections?.length, post.id);
  }
});
