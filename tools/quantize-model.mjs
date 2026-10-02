// Converts a Model2Vec static embedding model, as published on Hugging Face,
// into the two files the site ships for the Ask view:
//
//   vocab.txt         one token per line; the line number is the token ID
//   embeddings.q8.bin [vocab x f32 scale][vocab x dim x i8 weight], little-endian
//
// Each row is quantised on its own: scale = max(|row|) / 127, weight =
// round(value / scale). That cuts the file to a quarter of the float32
// original while keeping every row's direction almost exactly, which is all
// cosine similarity looks at.
//
// Run once when changing models, then commit the output. The build never
// downloads anything; see models/README.md for the provenance of the model
// that is committed.
//
//   node tools/quantize-model.mjs <downloaded-model-dir> <output-dir>
//
// The input directory needs model.safetensors and vocab.txt from the pinned
// revision of the model repository.

import { createHash } from "node:crypto";
import { mkdirSync, readFileSync, writeFileSync, copyFileSync } from "node:fs";
import { join } from "node:path";

const [input, output] = process.argv.slice(2);
if (!input || !output) {
  console.error("usage: node tools/quantize-model.mjs <model-dir> <output-dir>");
  process.exit(2);
}

const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");

// safetensors: an 8-byte little-endian header length, a JSON header naming
// each tensor's dtype, shape and byte range, then the raw tensor bytes.
const file = readFileSync(join(input, "model.safetensors"));
const headerLength = Number(file.readBigUInt64LE(0));
const header = JSON.parse(file.subarray(8, 8 + headerLength).toString("utf8"));
const tensor = header.embeddings;
if (!tensor || tensor.dtype !== "F32" || tensor.shape.length !== 2) {
  throw new Error(`expected one F32 [vocab, dim] tensor named "embeddings", got ${JSON.stringify(header)}`);
}
const [vocab, dim] = tensor.shape;
const start = 8 + headerLength + tensor.data_offsets[0];
const floats = new Float32Array(file.buffer.slice(file.byteOffset + start, file.byteOffset + start + vocab * dim * 4));

const vocabText = readFileSync(join(input, "vocab.txt"), "utf8");
const tokens = vocabText.split("\n");
if (tokens.at(-1) === "") tokens.pop();
if (tokens.length !== vocab) {
  throw new Error(`vocab.txt has ${tokens.length} tokens but the tensor has ${vocab} rows`);
}

const out = new ArrayBuffer(vocab * 4 + vocab * dim);
const scales = new Float32Array(out, 0, vocab);
const weights = new Int8Array(out, vocab * 4, vocab * dim);

let worst = 1;
let total = 0;
for (let row = 0; row < vocab; row++) {
  const values = floats.subarray(row * dim, (row + 1) * dim);
  let max = 0;
  for (const v of values) max = Math.max(max, Math.abs(v));
  const scale = max / 127 || 1;
  scales[row] = scale;

  let dot = 0;
  let a = 0;
  let b = 0;
  for (let i = 0; i < dim; i++) {
    const q = Math.round(values[i] / scale);
    weights[row * dim + i] = q;
    const back = q * scale;
    dot += values[i] * back;
    a += values[i] * values[i];
    b += back * back;
  }
  const cosine = a && b ? dot / Math.sqrt(a * b) : 1;
  worst = Math.min(worst, cosine);
  total += cosine;
}

mkdirSync(output, { recursive: true });
const bytes = new Uint8Array(out);
writeFileSync(join(output, "embeddings.q8.bin"), bytes);
copyFileSync(join(input, "vocab.txt"), join(output, "vocab.txt"));

console.log(`rows ${vocab}, dim ${dim}`);
console.log(`row cosine float32 vs int8: mean ${(total / vocab).toFixed(6)}, worst ${worst.toFixed(6)}`);
console.log(`model.safetensors sha256 ${sha256(file)}`);
console.log(`vocab.txt         sha256 ${sha256(Buffer.from(vocabText))}`);
console.log(`embeddings.q8.bin sha256 ${sha256(bytes)} (${bytes.length} bytes)`);
