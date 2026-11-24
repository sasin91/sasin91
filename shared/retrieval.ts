const MAX_WORD_CHARS = 100
const CONTROL = /\p{C}/u
const WHITESPACE = /\s/u
const MARK = /\p{Mn}/gu
const PUNCTUATION = /\p{P}/u

export interface RawPassage {
  snippet: string
  vector: number[]
  anchor?: string
  heading?: string
}

export interface PreparedPassage {
  snippet: string
  vector: Float32Array
  anchor?: string
  heading?: string
}

export interface RawSearchNode {
  kind: string
  context: string
  id: string
  order: number
  url: string
  title: string
  summary: string
  concepts: string[]
  search_terms: string[]
  words: string[]
  passages: RawPassage[]
  sections?: RawSearchNode[]
}

export interface SearchModelInfo {
  name: string
  weights_url: string
  vocab_url: string
  bytes: number
  dim: number
}

export interface RawSearchIndex {
  version: number
  model: SearchModelInfo
  nodes: RawSearchNode[]
}

export interface PhraseEntry {
  text: string
  field: 'title' | 'concept' | 'term'
  words: string[]
}

export interface PreparedSearchNode extends Omit<RawSearchNode, 'passages'> {
  phrases: PhraseEntry[]
  keywords: Set<string>
  text: Set<string>
  passages: PreparedPassage[]
}

export interface PreparedSearchIndex {
  version: number
  model: SearchModelInfo
  nodes: PreparedSearchNode[]
  frequency: Map<string, number>
}

export interface LexicalHit {
  text: string
  field: 'title' | 'concept' | 'term' | 'keyword' | 'text'
}

export interface RankedResult {
  node: PreparedSearchNode
  score: number
  semantic: number | null
  lexical: number
  matched: LexicalHit[]
  passage: PreparedPassage | null
  semanticRank?: number
}

export interface EncodedQuery {
  tokens: string[]
  unknown: number
  vector: Float32Array | null
}

function isControl(ch: string): boolean {
  return ch !== '\t' && ch !== '\n' && ch !== '\r' && CONTROL.test(ch)
}

function isCjk(cp: number): boolean {
  return (
    (cp >= 0x4e00 && cp <= 0x9fff) ||
    (cp >= 0x3400 && cp <= 0x4dbf) ||
    (cp >= 0x20000 && cp <= 0x2a6df) ||
    (cp >= 0x2a700 && cp <= 0x2b73f) ||
    (cp >= 0x2b740 && cp <= 0x2b81f) ||
    (cp >= 0x2b820 && cp <= 0x2ceaf) ||
    (cp >= 0xf900 && cp <= 0xfaff) ||
    (cp >= 0x2f800 && cp <= 0x2fa1f)
  )
}

// ASCII symbols such as $ + < = > ^ ` | ~ are not Unicode punctuation, but BERT splits on them.
function isPunctuation(ch: string): boolean {
  const cp = ch.codePointAt(0) ?? 0
  return (
    (cp >= 33 && cp <= 47) ||
    (cp >= 58 && cp <= 64) ||
    (cp >= 91 && cp <= 96) ||
    (cp >= 123 && cp <= 126) ||
    PUNCTUATION.test(ch)
  )
}

export function normalize(text: string): string {
  let out = ''
  for (const ch of text) {
    const cp = ch.codePointAt(0) ?? 0
    if (cp === 0 || cp === 0xfffd || isControl(ch)) continue
    if (WHITESPACE.test(ch)) out += ' '
    else if (isCjk(cp)) out += ` ${ch} `
    else out += ch
  }
  return out.normalize('NFD').replace(MARK, '').toLowerCase()
}

export function preTokenize(normalized: string): string[] {
  const out: string[] = []
  for (const chunk of normalized.split(WHITESPACE)) {
    let word = ''
    for (const ch of chunk) {
      if (isPunctuation(ch)) {
        if (word) out.push(word)
        out.push(ch)
        word = ''
      } else {
        word += ch
      }
    }
    if (word) out.push(word)
  }
  return out
}

export function wordPiece(word: string, vocab: Map<string, number>): string[] {
  const chars = Array.from(word)
  if (chars.length > MAX_WORD_CHARS) return ['[UNK]']

  const pieces: string[] = []
  let start = 0
  while (start < chars.length) {
    let end = chars.length
    let piece: string | null = null
    while (start < end) {
      const candidate = (start > 0 ? '##' : '') + chars.slice(start, end).join('')
      if (vocab.has(candidate)) {
        piece = candidate
        break
      }
      end--
    }
    if (piece === null) return ['[UNK]']
    pieces.push(piece)
    start = end
  }
  return pieces
}

function unit(vector: Float32Array): Float32Array {
  let length = 0
  for (const v of vector) length += v * v
  length = Math.sqrt(length)
  if (length > 0) {
    for (let i = 0; i < vector.length; i++) {
      vector[i] = vector[i]! / length
    }
  }
  return vector
}

export class StaticModel {
  readonly dim: number
  readonly vocab: Map<string, number>
  readonly scales: Float32Array
  readonly weights: Int8Array

  constructor(vocabText: string, buffer: ArrayBuffer) {
    const tokens = vocabText.split('\n')
    if (tokens.at(-1) === '') tokens.pop()
    const rows = tokens.length
    const dim = (buffer.byteLength - rows * 4) / rows
    if (!Number.isInteger(dim) || dim <= 0) {
      throw new Error(`model file of ${buffer.byteLength} bytes does not fit ${rows} tokens`)
    }

    this.dim = dim
    this.vocab = new Map(tokens.map((token, id) => [token, id]))
    this.scales = new Float32Array(buffer, 0, rows)
    this.weights = new Int8Array(buffer, rows * 4, rows * dim)
  }

  tokenize(text: string): string[] {
    return preTokenize(normalize(text)).flatMap((word) => wordPiece(word, this.vocab))
  }

  encode(text: string): EncodedQuery {
    const tokens = this.tokenize(text)
    const sum = new Float32Array(this.dim)
    let known = 0
    for (const token of tokens) {
      if (token === '[UNK]') continue
      const id = this.vocab.get(token)
      if (id === undefined) continue
      const scale = this.scales[id]!
      const row = id * this.dim
      for (let i = 0; i < this.dim; i++) {
        sum[i] = sum[i]! + this.weights[row + i]! * scale
      }
      known++
    }
    return { tokens, unknown: tokens.length - known, vector: known ? unit(sum) : null }
  }
}

export function cosine(a: ArrayLike<number>, b: ArrayLike<number>): number {
  let dot = 0
  for (let i = 0; i < a.length; i++) dot += (a[i] ?? 0) * (b[i] ?? 0)
  return dot
}

const STOPWORDS = new Set(
  (
    'a about after all also an and any are as at be because been before but by can could ' +
    'did do does doesn don each for from had has have how i if in into is isn it its just ' +
    'me more most my no not of on or other our out over s so some such t than that the their ' +
    'them then there these they this those through to too under up us very was way we were ' +
    'what when where which while who why will with would you your'
  ).split(' '),
)

export function words(text: string): string[] {
  return text.toLowerCase().match(/[\p{Alphabetic}\p{N}]+/gu) ?? []
}

export function contentWords(text: string): string[] {
  return [...new Set(words(text).filter((w) => !STOPWORDS.has(w)))]
}

function containsPhrase(haystack: string[], phrase: string[]): boolean {
  outer: for (let i = 0; i + phrase.length <= haystack.length; i++) {
    for (let j = 0; j < phrase.length; j++) {
      if (haystack[i + j] !== phrase[j]) continue outer
    }
    return true
  }
  return false
}

export const WEIGHTS = {
  title: 1.0,
  concept: 0.7,
  term: 0.6,
  keyword: 0.25,
  text: 0.1,
  lexical: 0.5,
} as const

export function prepareIndex(index: RawSearchIndex): PreparedSearchIndex {
  const nodes: PreparedSearchNode[] = index.nodes.map((node) => {
    const phrases: PhraseEntry[] = [
      { text: node.title, field: 'title' as const },
      ...node.concepts.map((text) => ({ text, field: 'concept' as const })),
      ...node.search_terms.map((text) => ({ text, field: 'term' as const })),
    ]
      .map((phrase) => ({ ...phrase, words: words(phrase.text) }))
      .filter((phrase) => phrase.words.some((w) => !STOPWORDS.has(w)))

    const keywords = new Set(phrases.flatMap((phrase) => phrase.words))
    const text = new Set([...words(node.summary), ...node.words])

    return {
      ...node,
      phrases,
      keywords,
      text,
      passages: node.passages.map((p) => ({ ...p, vector: Float32Array.from(p.vector) })),
    }
  })

  const frequency = new Map<string, number>()
  for (const node of nodes) {
    for (const word of new Set([...node.keywords, ...node.text])) {
      frequency.set(word, (frequency.get(word) ?? 0) + 1)
    }
  }

  return { ...index, nodes, frequency }
}

function rarity(prepared: PreparedSearchIndex, word: string): number {
  const n = prepared.nodes.length
  const df = prepared.frequency.get(word) ?? 0
  if (df === 0) return 0
  return n < 2 ? 1 : Math.log(n / df) / Math.log(n)
}

export function lexicalMatch(
  prepared: PreparedSearchIndex,
  node: PreparedSearchNode,
  query: string,
): { score: number; matched: LexicalHit[] } {
  const queryWords = words(query)
  const matched: LexicalHit[] = []
  let raw = 0

  const coveredByPhrase = new Set<string>()
  for (const phrase of node.phrases) {
    if (!containsPhrase(queryWords, phrase.words)) continue
    const specificity =
      phrase.words.length > 1
        ? 1 + 0.5 * (phrase.words.length - 1)
        : 0.4 + 0.6 * rarity(prepared, phrase.words[0]!)
    raw += WEIGHTS[phrase.field] * specificity
    matched.push({ text: phrase.text, field: phrase.field })
    for (const w of phrase.words) coveredByPhrase.add(w)
  }

  for (const word of contentWords(query)) {
    if (coveredByPhrase.has(word)) continue
    const r = rarity(prepared, word)
    if (node.keywords.has(word)) {
      raw += WEIGHTS.keyword * r
      matched.push({ text: word, field: 'keyword' })
    } else if (node.text.has(word)) {
      raw += WEIGHTS.text * r
      matched.push({ text: word, field: 'text' })
    }
  }

  return { score: 1 - Math.exp(-raw), matched }
}

export function rank(
  prepared: PreparedSearchIndex,
  query: string,
  queryVector: Float32Array | null,
  limit = 5,
): RankedResult[] {
  const scored: RankedResult[] = prepared.nodes.map((node) => {
    const lexical = lexicalMatch(prepared, node, query)
    let semantic: number | null = null
    let passage: PreparedPassage | null = null
    if (queryVector) {
      for (const p of node.passages) {
        const similarity = cosine(queryVector, p.vector)
        if (semantic === null || similarity > semantic) {
          semantic = similarity
          passage = p
        }
      }
    }
    const score = queryVector && semantic !== null ? semantic + WEIGHTS.lexical * lexical.score : lexical.score
    return { node, score, semantic, lexical: lexical.score, matched: lexical.matched, passage }
  })

  if (queryVector) {
    const byMeaning = [...scored].sort((a, b) => (b.semantic ?? -Infinity) - (a.semantic ?? -Infinity))
    byMeaning.forEach((result, i) => {
      result.semanticRank = i + 1
    })
  }

  return scored
    .filter((result) => result.score > 0)
    .sort((a, b) => b.score - a.score || a.node.order - b.node.order)
    .slice(0, limit)
}

export function resultUrl(result: RankedResult): string {
  const anchor = result.passage?.anchor
  return anchor ? `${result.node.url.split('#')[0]}#${anchor}` : result.node.url
}

export function scopedIndex(index: RawSearchIndex, scope: string): RawSearchIndex {
  if (scope.startsWith('post:')) {
    const id = scope.slice(5)
    const post = index.nodes.find((n) => n.kind === 'post' && n.id === id)
    if (!post?.sections) throw new Error(`no sections for ${scope}`)
    return { ...index, nodes: post.sections }
  }
  return { ...index, nodes: index.nodes.filter((n) => n.kind === scope) }
}

async function fetchOk(url: string): Promise<Response> {
  const response = await fetch(url)
  if (!response.ok) throw new Error(`${url}: ${response.status}`)
  return response
}

export async function fetchIndex(url: string, kind: string | null = null): Promise<RawSearchIndex> {
  const index = (await fetchOk(url).then((r) => r.json())) as RawSearchIndex
  return kind ? { ...index, nodes: index.nodes.filter((n) => n.kind === kind) } : index
}

export async function fetchModel(info: SearchModelInfo): Promise<StaticModel> {
  const [vocab, weights] = await Promise.all([
    fetchOk(info.vocab_url).then((r) => r.text()),
    fetchOk(info.weights_url).then((r) => r.arrayBuffer()),
  ])
  return new StaticModel(vocab, weights)
}
