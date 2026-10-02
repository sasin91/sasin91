# Embedding model for /ai/ask/

`potion-base-4M/` is the model behind the AI article's Ask view. The build uses
it to embed every passage of the article (`src/embed.rs`), and the reader's
browser downloads the same files to embed their question
(`js/retrieval.mjs`). Nothing is fetched from a model registry at build or run
time; the files are committed so the site is reproducible and works offline.

## What it is

[potion-base-4M](https://huggingface.co/minishlab/potion-base-4M) by Minish
Lab (Stephan Tulkens and Thomas van Dongen), a
[Model2Vec](https://github.com/MinishLab/model2vec) *static* embedding model
distilled from BAAI/bge-base-en-v1.5. It is a table with one 128-dimensional
vector per WordPiece token. A text's embedding is the mean of its tokens'
vectors, normalised. There is no transformer to run, so the browser needs no
ML runtime, WASM or GPU: embedding a question is a few hundred table lookups.

Licence: MIT, as declared on the model card. The vocabulary is the
bge-base-en-v1.5 / BERT uncased WordPiece vocabulary.

## Files

| File | What | Size |
| --- | --- | --- |
| `vocab.txt` | One token per line; the line number is the token ID. Unmodified from the source. | 219,690 B |
| `embeddings.q8.bin` | 29,528 little-endian f32 row scales, then 29,528 × 128 int8 weights | 3,897,696 B |
| `wordpiece-fixture.json` | Reference tokenizations of awkward inputs, produced by Hugging Face's `@huggingface/tokenizers` from the model's own `tokenizer.json`. Both test suites must reproduce it exactly. | 4.7 kB |

Int8 quantisation is per row (`scale = max|row| / 127`). Against the original
float32 rows, the mean cosine similarity is 0.999973 and the worst row is
0.999886, so ranking is unaffected.

## Provenance

Source: `minishlab/potion-base-4M`, revision
`9b3cff412d30be9ae8603fe10224c224f3401869`.

| Source file | sha256 |
| --- | --- |
| `model.safetensors` | `8a7140edd17ffab30ddcff1135eda127df57abfae856203dbd7b3d061295c31a` |
| `vocab.txt` | `1394523a67ddd404a825428018c0582a6998bcfa044ecbcbf1f4d71adb94c61c` |
| `tokenizer.json` (fixture only) | `e67e803f624fb4d67dea1c730d06e1067e1b14d830e2c2202569e3ef0f70bb50` |

`embeddings.q8.bin` sha256:
`245786ba08d83dc772a41d8ae0acdc619695a158a130f3a26db2a6a0b51b17e0`.

To regenerate, download `model.safetensors` and `vocab.txt` from that revision
into a directory, then:

```sh
node tools/quantize-model.mjs <download-dir> models/potion-base-4M
```

## Why this model

Measured on this article, 28 tuning questions plus 15 held-out ones, top
result correct:

| Model | Download | Hybrid, tuned set | Hybrid, held out | Meaning only, held out |
| --- | --- | --- | --- | --- |
| potion-base-4M (int8) | 3.9 MB | 28/28 | 14/15 | 12/15 |
| potion-base-8M (int8) | 7.7 MB | 27/28 | 13/15 | 12/15 |

The larger model did not help on a corpus this size, at twice the download.

The trade-off being accepted: a static model has no context. "bank" gets
one vector whether it is a river or an account, and word order is ignored.
The lexical half of the ranking (`js/retrieval.mjs`) makes up for that on
this site's technical vocabulary.

Changing to another *static* model means converting it with
`tools/quantize-model.mjs` and changing `AI_MODEL_DIR` and `AI_MODEL_NAME` in
`src/main.rs`; the index is rebuilt with it automatically.

## When to switch to a transformer model

Switch when either of these shows up in real queries:

- **The content grows** to the point where the static model's misses are
  regular rather than occasional.
- **Failures the static model causes:** the wrong sense of a word, or a
  paraphrase that shares no words with the passage that answers it (so the
  lexical half cannot rescue it either).

Then move to a small contextual embedding model, all-MiniLM-L6-v2 or
bge-small-en-v1.5, run in the browser with Transformers.js (ONNX Runtime Web
underneath). Expect roughly 23–34 MB of int8 model plus a WASM runtime of
several megabytes, and 10–80 ms per query instead of under one. Those
figures are approximate; measure before deciding, with the same tuning and
held-out questions as the table above.

What changes, and what does not:

| Changes | Stays |
| --- | --- |
| The in-browser query embedding: `StaticModel` in `js/retrieval.mjs` is replaced by a Transformers.js pipeline, lazy-loaded the same way | The hybrid ranking (`rank`, `lexicalMatch`, the weights) |
| The build-time passage embedding in `src/embed.rs`, which must use the identical model. Either a Rust inference crate (`ort` or `candle`) or a small Node build step | The palette, the Ask page and their loading and fallback behaviour |
| The index's vectors, and `model` in the index header (dimension 384 instead of 128) | The index format: documents, passages, snippets, anchors |
| `models/`, and the tokenizer fixture tests | The knowledge node and post content, and everything that validates it |

Contextual models also cap input length (about 256–512 tokens), which
paragraph-sized passages already respect.
