import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, expect, test } from 'vitest'
import {
  cosine,
  prepareIndex,
  rank,
  resultUrl,
  scopedIndex,
  StaticModel,
  type RawSearchIndex,
} from '#shared/retrieval'

const ROOT = process.cwd()
const MODEL_DIR = resolve(ROOT, 'models/potion-base-4M')

function loadModel(): StaticModel {
  const vocab = readFileSync(resolve(MODEL_DIR, 'vocab.txt'), 'utf8')
  const buf = readFileSync(resolve(MODEL_DIR, 'embeddings.q8.bin'))
  const ab = buf.buffer.slice(buf.byteOffset, buf.byteOffset + buf.byteLength)
  return new StaticModel(vocab, ab)
}

describe('retrieval engine', () => {
  test('WordPiece tokenizer matches the reference fixture byte for byte', () => {
    const model = loadModel()
    const fixture = JSON.parse(
      readFileSync(resolve(MODEL_DIR, 'wordpiece-fixture.json'), 'utf8'),
    ) as { cases?: Array<{ input: string; tokens: string[] }> } | Array<{ text: string; tokens: string[] }>

    const cases = Array.isArray(fixture) ? fixture : (fixture.cases ?? [])
    expect(cases.length).toBeGreaterThan(0)

    for (const entry of cases) {
      const text = 'text' in entry ? entry.text : entry.input
      expect(model.tokenize(text)).toEqual(entry.tokens)
    }
  })

  test('StaticModel produces unit vectors and returns null when all tokens are unknown', () => {
    const model = loadModel()
    const encoded = model.encode('How does the KV cache work?')
    expect(encoded.vector).not.toBeNull()
    expect(encoded.vector!.length).toBe(model.dim)
    expect(cosine(encoded.vector!, encoded.vector!)).toBeCloseTo(1, 4)

    const empty = model.encode('')
    expect(empty.vector).toBeNull()
  })

  test('hybrid ranking finds the right AI knowledge nodes and blog post sections', () => {
    const model = loadModel()
    const raw = JSON.parse(
      readFileSync(resolve(ROOT, 'public/search/index.json'), 'utf8'),
    ) as RawSearchIndex

    const aiIndex = prepareIndex(scopedIndex(raw, 'ai'))
    const kvQuery = 'What is stored in the KV cache?'
    const kvResults = rank(aiIndex, kvQuery, model.encode(kvQuery).vector)
    expect(kvResults[0]?.node.id).toBe('kv-cache')

    const quantQuery = 'How does 4-bit quantization shrink model weights?'
    const quantResults = rank(aiIndex, quantQuery, model.encode(quantQuery).vector)
    expect(quantResults[0]?.node.id).toBe('quantization')

    const halluQuery = 'Why do language models make up fake references?'
    const halluResults = rank(aiIndex, halluQuery, model.encode(halluQuery).vector)
    expect(halluResults[0]?.node.id).toBe('hallucination')

    const postIndex = prepareIndex(scopedIndex(raw, 'post:blog/trongate/recruiter'))
    const postQuery = 'How does cv_match work?'
    const postResults = rank(postIndex, postQuery, model.encode(postQuery).vector)
    expect(postResults.length).toBeGreaterThan(0)
    expect(resultUrl(postResults[0]!)).toContain('/blog/trongate/recruiter/')
  })
})
