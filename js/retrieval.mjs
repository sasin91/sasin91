// Retrieval for /ai/ask/: everything that turns a question into ranked
// knowledge nodes, with no DOM access, so it can be tested under `node --test`
// and inlined into the page unchanged (see `ask_script` in src/main.rs).
//
// Two independent signals are combined:
//
//   semantic  A static embedding model (Model2Vec potion-base): tokenize with
//             WordPiece, look up one int8 row per token, average, normalise.
//             The same steps run in src/embed.rs at build time to embed every
//             passage of the article, so a query and a passage land in the
//             same vector space. No neural network runs here; it is a table
//             lookup, which is why no ML runtime, WASM or WebGPU is needed.
//
//   lexical   Plain word matching against each node's title, concepts,
//             search terms and text, so that exact technical terms like
//             "KV cache" or "MCP" are never outranked by something vaguely
//             similar in meaning.
//
// The tokenizer must stay byte-for-byte compatible with src/embed.rs;
// models/potion-base-4M/wordpiece-fixture.json is checked by both test suites.

// ---------- WordPiece tokenizer (BERT uncased rules) ----------

const MAX_WORD_CHARS = 100;

const CONTROL = /\p{C}/u;
const WHITESPACE = /\s/u;
const MARK = /\p{Mn}/gu;
const PUNCTUATION = /\p{P}/u;

function isControl(ch) {
  return ch !== "\t" && ch !== "\n" && ch !== "\r" && CONTROL.test(ch);
}

// CJK ideographs are split into one token each, as BERT does.
function isCjk(cp) {
  return (
    (cp >= 0x4e00 && cp <= 0x9fff) ||
    (cp >= 0x3400 && cp <= 0x4dbf) ||
    (cp >= 0x20000 && cp <= 0x2a6df) ||
    (cp >= 0x2a700 && cp <= 0x2b73f) ||
    (cp >= 0x2b740 && cp <= 0x2b81f) ||
    (cp >= 0x2b820 && cp <= 0x2ceaf) ||
    (cp >= 0xf900 && cp <= 0xfaff) ||
    (cp >= 0x2f800 && cp <= 0x2fa1f)
  );
}

// ASCII symbols such as $ + < = > ^ ` | ~ are not Unicode punctuation, but
// BERT treats them as punctuation anyway.
function isPunctuation(ch) {
  const cp = ch.codePointAt(0);
  return (
    (cp >= 33 && cp <= 47) ||
    (cp >= 58 && cp <= 64) ||
    (cp >= 91 && cp <= 96) ||
    (cp >= 123 && cp <= 126) ||
    PUNCTUATION.test(ch)
  );
}

/** Clean, split CJK, strip accents, lowercase: the BERT normaliser. */
export function normalize(text) {
  let out = "";
  for (const ch of text) {
    const cp = ch.codePointAt(0);
    if (cp === 0 || cp === 0xfffd || isControl(ch)) continue;
    if (WHITESPACE.test(ch)) out += " ";
    else if (isCjk(cp)) out += ` ${ch} `;
    else out += ch;
  }
  return out.normalize("NFD").replace(MARK, "").toLowerCase();
}

/** Split normalised text on whitespace, isolating every punctuation mark. */
export function preTokenize(normalized) {
  const words = [];
  for (const chunk of normalized.split(WHITESPACE)) {
    let word = "";
    for (const ch of chunk) {
      if (isPunctuation(ch)) {
        if (word) words.push(word);
        words.push(ch);
        word = "";
      } else {
        word += ch;
      }
    }
    if (word) words.push(word);
  }
  return words;
}

/**
 * Greedy longest-match-first WordPiece. Returns the token strings; a word
 * that cannot be fully covered by the vocabulary becomes a single [UNK].
 */
export function wordPiece(word, vocab) {
  const chars = Array.from(word);
  if (chars.length > MAX_WORD_CHARS) return ["[UNK]"];

  const pieces = [];
  let start = 0;
  while (start < chars.length) {
    let end = chars.length;
    let piece = null;
    while (start < end) {
      const candidate = (start > 0 ? "##" : "") + chars.slice(start, end).join("");
      if (vocab.has(candidate)) {
        piece = candidate;
        break;
      }
      end--;
    }
    if (piece === null) return ["[UNK]"];
    pieces.push(piece);
    start = end;
  }
  return pieces;
}

// ---------- Static embedding model ----------

export class StaticModel {
  /**
   * @param {string} vocabText vocab.txt: one token per line, line number = ID
   * @param {ArrayBuffer} buffer embeddings.q8.bin: f32 scales, then i8 rows
   */
  constructor(vocabText, buffer) {
    const tokens = vocabText.split("\n");
    if (tokens.at(-1) === "") tokens.pop();
    const rows = tokens.length;
    const dim = (buffer.byteLength - rows * 4) / rows;
    if (!Number.isInteger(dim) || dim <= 0) {
      throw new Error(`model file of ${buffer.byteLength} bytes does not fit ${rows} tokens`);
    }

    this.dim = dim;
    this.vocab = new Map(tokens.map((token, id) => [token, id]));
    this.scales = new Float32Array(buffer, 0, rows);
    this.weights = new Int8Array(buffer, rows * 4, rows * dim);
  }

  /** WordPiece tokens of `text`, as strings, including any [UNK]. */
  tokenize(text) {
    return preTokenize(normalize(text)).flatMap((word) => wordPiece(word, this.vocab));
  }

  /**
   * Mean of the known tokens' rows, normalised to unit length. Unknown
   * tokens are dropped, as Model2Vec does. `vector` is null when no token
   * was known, since a zero vector is similar to nothing.
   */
  encode(text) {
    const tokens = this.tokenize(text);
    const sum = new Float32Array(this.dim);
    let known = 0;
    for (const token of tokens) {
      if (token === "[UNK]") continue;
      const id = this.vocab.get(token);
      const scale = this.scales[id];
      const row = id * this.dim;
      for (let i = 0; i < this.dim; i++) sum[i] += this.weights[row + i] * scale;
      known++;
    }
    return { tokens, unknown: tokens.length - known, vector: known ? unit(sum) : null };
  }
}

function unit(vector) {
  let length = 0;
  for (const v of vector) length += v * v;
  length = Math.sqrt(length);
  if (length > 0) for (let i = 0; i < vector.length; i++) vector[i] /= length;
  return vector;
}

/** Cosine similarity of two unit vectors, which is just their dot product. */
export function cosine(a, b) {
  let dot = 0;
  for (let i = 0; i < a.length; i++) dot += a[i] * b[i];
  return dot;
}

// ---------- Lexical matching ----------

// Words that say nothing about which node a question is about. Kept short on
// purpose: a word missing from here only adds a little noise, while a
// technical word wrongly listed here could never be matched.
const STOPWORDS = new Set(
  (
    "a about after all also an and any are as at be because been before but by can could " +
    "did do does doesn don each for from had has have how i if in into is isn it its just " +
    "me more most my no not of on or other our out over s so some such t than that the their " +
    "them then there these they this those through to too under up us very was way we were " +
    "what when where which while who why will with would you your"
  ).split(" "),
);

/** Lowercase runs of letters and digits; must match `lexical_words` in src/search_index.rs. */
export function words(text) {
  return text.toLowerCase().match(/[\p{Alphabetic}\p{N}]+/gu) ?? [];
}

/** The words of a query worth matching on: no stopwords, no repeats. */
export function contentWords(text) {
  return [...new Set(words(text).filter((w) => !STOPWORDS.has(w)))];
}

function containsPhrase(haystack, phrase) {
  outer: for (let i = 0; i + phrase.length <= haystack.length; i++) {
    for (let j = 0; j < phrase.length; j++) {
      if (haystack[i + j] !== phrase[j]) continue outer;
    }
    return true;
  }
  return false;
}

// How much each kind of evidence is worth. A whole phrase from a node's own
// metadata appearing in the question is the strongest signal there is; a
// single word turning up somewhere in the body is the weakest.
export const WEIGHTS = {
  title: 1.0, // the node's title, as a phrase
  concept: 0.7, // one of its concepts, as a phrase
  term: 0.6, // one of its search terms, as a phrase
  keyword: 0.25, // a word from the title, concepts or search terms
  text: 0.1, // a word from the summary or body
  // How far lexical evidence can move a result relative to cosine
  // similarity. Cosines between unrelated passages of a static model still
  // sit around 0.2-0.4, so the gap between a good and a vague semantic match
  // is often under 0.2: an exact concept match (lexical around 0.5) is meant
  // to clear that gap.
  lexical: 0.5,
};

/**
 * Prepares the static index (index.<hash>.json, written by src/search_index.rs) for
 * ranking: phrase lists, word sets and document frequencies.
 */
export function prepareIndex(index) {
  const nodes = index.nodes.map((node) => {
    const phrases = [
      { text: node.title, field: "title" },
      ...node.concepts.map((text) => ({ text, field: "concept" })),
      ...node.search_terms.map((text) => ({ text, field: "term" })),
    ]
      .map((phrase) => ({ ...phrase, words: words(phrase.text) }))
      .filter((phrase) => phrase.words.some((w) => !STOPWORDS.has(w)));

    const keywords = new Set(phrases.flatMap((phrase) => phrase.words));
    const text = new Set([...words(node.summary), ...node.words]);

    return {
      ...node,
      phrases,
      keywords,
      text,
      passages: node.passages.map((p) => ({ ...p, vector: Float32Array.from(p.vector) })),
    };
  });

  const frequency = new Map();
  for (const node of nodes) {
    for (const word of new Set([...node.keywords, ...node.text])) {
      frequency.set(word, (frequency.get(word) ?? 0) + 1);
    }
  }

  return { ...index, nodes, frequency };
}

/**
 * Rarity of a word across nodes, from 0 (in every node) towards 1 (in one).
 * A word every node shares cannot tell nodes apart.
 */
function rarity(prepared, word) {
  const n = prepared.nodes.length;
  const df = prepared.frequency.get(word) ?? 0;
  if (df === 0) return 0;
  return n < 2 ? 1 : Math.log(n / df) / Math.log(n);
}

/** Lexical evidence for one node: a score in [0, 1) and what matched. */
export function lexicalMatch(prepared, node, query) {
  const queryWords = words(query);
  const matched = [];
  let raw = 0;

  const coveredByPhrase = new Set();
  for (const phrase of node.phrases) {
    if (!containsPhrase(queryWords, phrase.words)) continue;
    // Longer phrases are more specific: "context window" says more than
    // "context" does. A one-word phrase is only as specific as the word is
    // rare: "mcp" names one node, "context" half of them.
    const specificity =
      phrase.words.length > 1
        ? 1 + 0.5 * (phrase.words.length - 1)
        : 0.4 + 0.6 * rarity(prepared, phrase.words[0]);
    raw += WEIGHTS[phrase.field] * specificity;
    matched.push({ text: phrase.text, field: phrase.field });
    for (const w of phrase.words) coveredByPhrase.add(w);
  }

  for (const word of contentWords(query)) {
    if (coveredByPhrase.has(word)) continue;
    const r = rarity(prepared, word);
    if (node.keywords.has(word)) {
      raw += WEIGHTS.keyword * r;
      matched.push({ text: word, field: "keyword" });
    } else if (node.text.has(word)) {
      raw += WEIGHTS.text * r;
      matched.push({ text: word, field: "text" });
    }
  }

  return { score: 1 - Math.exp(-raw), matched };
}

/**
 * Ranks every node for `query`.
 *
 * With a query vector: score = best passage cosine + WEIGHTS.lexical x lexical.
 * Without one (model unavailable, or no known tokens): score = lexical.
 *
 * Each result carries the evidence behind it, for the "why this matched"
 * view: the best passage, its cosine, the node's rank by meaning alone, and
 * the matched words and phrases.
 */
export function rank(prepared, query, queryVector, limit = 5) {
  const scored = prepared.nodes.map((node) => {
    const lexical = lexicalMatch(prepared, node, query);
    let semantic = null;
    let passage = null;
    if (queryVector) {
      for (const p of node.passages) {
        const similarity = cosine(queryVector, p.vector);
        if (semantic === null || similarity > semantic) {
          semantic = similarity;
          passage = p;
        }
      }
    }
    const score = queryVector ? semantic + WEIGHTS.lexical * lexical.score : lexical.score;
    return { node, score, semantic, lexical: lexical.score, matched: lexical.matched, passage };
  });

  if (queryVector) {
    const byMeaning = [...scored].sort((a, b) => b.semantic - a.semantic);
    byMeaning.forEach((result, i) => (result.semanticRank = i + 1));
  }

  return scored
    .filter((result) => result.score > 0)
    .sort((a, b) => b.score - a.score || a.node.order - b.node.order)
    .slice(0, limit);
}

/**
 * Where a result should link: the section of the page its best passage sits
 * in, when that passage is under a heading (posts), otherwise the document's
 * own URL (which for a knowledge node already points at the node).
 */
export function resultUrl(result) {
  const anchor = result.passage?.anchor;
  return anchor ? `${result.node.url.split("#")[0]}#${anchor}` : result.node.url;
}

// ---------- loading (shared by /ai/ask/ and the search palette) ----------

async function fetchOk(url) {
  const response = await fetch(url);
  if (!response.ok) throw new Error(`${url}: ${response.status}`);
  return response;
}

/** The raw index JSON, optionally narrowed to one kind of document. */
export async function fetchIndex(url, kind = null) {
  const index = await fetchOk(url).then((r) => r.json());
  return kind ? { ...index, nodes: index.nodes.filter((n) => n.kind === kind) } : index;
}

/** The embedding model the index describes. Rejects if either file fails. */
export async function fetchModel(info) {
  const [vocab, weights] = await Promise.all([
    fetchOk(info.vocab_url).then((r) => r.text()),
    fetchOk(info.weights_url).then((r) => r.arrayBuffer()),
  ]);
  return new StaticModel(vocab, weights);
}
