import { createHash } from 'node:crypto'
import { readFileSync, writeFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { JSDOM } from 'jsdom'
import { StaticModel, words } from '../shared/retrieval.ts'

const ROOT = process.cwd()
const MODEL_DIR = resolve(ROOT, 'models/potion-base-4M')
const CONTENT_DIR = resolve(ROOT, 'content')
const OUT_FILE = resolve(ROOT, 'public/search/index.json')
const SNIPPET_CHARS = 160

function collapse(text) {
  return text.trim().split(/\s+/).filter(Boolean).join(' ')
}

function snippet(text) {
  if (text.length <= SNIPPET_CHARS) return text
  let cut = text.slice(0, SNIPPET_CHARS)
  const space = cut.lastIndexOf(' ')
  if (space > 0) cut = cut.slice(0, space)
  return `${cut.replace(/[,;:.]+$/, '')}…`
}

function blockText(node) {
  const tag = node.tagName.toLowerCase()
  if (['pre', 'script', 'style', 'video', 'math'].includes(tag)) return ''
  if (tag === 'figure') return collapse(node.getAttribute('aria-label') ?? '')
  if (node.getAttribute('class') === 'codeblock') return ''
  const copy = node.cloneNode(true)
  for (const bad of ['pre', 'math', 'script']) {
    for (const el of copy.querySelectorAll(bad)) el.remove()
  }
  for (const img of copy.querySelectorAll('img')) {
    img.replaceWith(img.getAttribute('alt') ?? '')
  }
  for (const fig of copy.querySelectorAll('figure')) {
    fig.replaceWith(fig.getAttribute('aria-label') ?? '')
  }
  return collapse(copy.textContent ?? '')
}

function headingText(section) {
  for (const child of section.children) {
    if (/^h[1-6]$/i.test(child.tagName)) return collapse(child.textContent ?? '')
  }
  return ''
}

function extractPassages(html) {
  const dom = new JSDOM(`<body>${html}</body>`)
  const passages = []
  let section = null

  function collect(parent) {
    for (const child of parent.children) {
      const tag = child.tagName.toLowerCase()
      if (tag === 'section' || tag === 'div') {
        if (child.hasAttribute('id')) {
          section = { id: child.getAttribute('id'), heading: headingText(child) }
        }
        collect(child)
        continue
      }
      if (/^h[1-6]$/.test(tag)) {
        if (child.hasAttribute('id')) {
          section = { id: child.getAttribute('id'), heading: collapse(child.textContent ?? '') }
        }
        continue
      }
      const text = blockText(child)
      if (!text) continue
      const prev = passages.at(-1)
      if (prev && prev.text.endsWith(':') && prev.section === section) {
        prev.text = `${prev.text} ${text}`
        continue
      }
      passages.push({ text, section })
    }
  }

  collect(dom.window.document.body)
  return passages
}

function embedPassage(model, text, snip, section) {
  const { vector } = model.encode(text)
  if (!vector) return null
  const rounded = Array.from(vector, (v) => Math.round(v * 10000) / 10000)
  const out = { snippet: snip, vector: rounded }
  if (section) {
    out.anchor = section.id
    out.heading = section.heading
  }
  return out
}

function lexicalWords(list) {
  return [...new Set(words(list.map((p) => p.text).join(' ')))].sort()
}

function buildPostSections(model, post, body) {
  const groups = []
  for (const passage of body) {
    const last = groups.at(-1)
    if (last && last.section === passage.section) {
      last.passages.push(passage)
    } else {
      groups.push({ section: passage.section, passages: [passage] })
    }
  }

  return groups.map((group, idx) => {
    const firstText = group.passages[0].text
    const sum = snippet(firstText)
    const section = group.section
    const id = section ? section.id : 'top'
    const title = section ? section.heading : 'Introduction'
    const url = section ? `${post.url}#${section.id}` : post.url
    const headPassages = section
      ? [embedPassage(model, `${title}. ${firstText}`, sum, section)].filter(Boolean)
      : []
    const bodyPassages = group.passages
      .map((p) => embedPassage(model, p.text, snippet(p.text), p.section))
      .filter(Boolean)

    return {
      kind: 'section',
      context: post.title,
      id,
      order: idx + 1,
      url,
      title,
      summary: sum,
      concepts: [],
      search_terms: [],
      words: lexicalWords(group.passages),
      passages: [...headPassages, ...bodyPassages],
    }
  })
}

const vocabText = readFileSync(resolve(MODEL_DIR, 'vocab.txt'), 'utf8')
const weightsBuf = readFileSync(resolve(MODEL_DIR, 'embeddings.q8.bin'))
const arrayBuffer = weightsBuf.buffer.slice(
  weightsBuf.byteOffset,
  weightsBuf.byteOffset + weightsBuf.byteLength,
)
const model = new StaticModel(vocabText, arrayBuffer)
const version = createHash('md5').update(`${vocabText}:${weightsBuf.byteLength}`).digest('hex').slice(0, 16)
const modelInfo = {
  name: 'potion-base-4M',
  weights_url: `/search/model.bin?v=${version}`,
  vocab_url: `/search/vocab.txt?v=${version}`,
  bytes: weightsBuf.byteLength + Buffer.byteLength(vocabText, 'utf8'),
  dim: model.dim,
}

const article = JSON.parse(readFileSync(resolve(CONTENT_DIR, 'ai/article.json'), 'utf8'))
const posts = JSON.parse(readFileSync(resolve(CONTENT_DIR, 'blog/meta.json'), 'utf8'))
const documents = []

for (const node of article.nodes) {
  const html = readFileSync(resolve(CONTENT_DIR, `ai/nodes/${node.id}.html`), 'utf8')
  const body = extractPassages(html)
  const headText = `${node.title}. ${node.summary} ${node.concepts.join(', ')}. ${node.searchTerms.join(', ')}.`
  const passages = [
    embedPassage(model, headText, node.summary, null),
    ...body.map((p) => embedPassage(model, p.text, snippet(p.text), p.section)),
  ].filter(Boolean)

  documents.push({
    kind: 'ai',
    context: article.title,
    id: node.id,
    order: node.order,
    url: `${article.url}#${node.id}`,
    title: node.title,
    summary: node.summary,
    concepts: node.concepts,
    search_terms: node.searchTerms,
    words: lexicalWords(body),
    passages,
  })
}

const nodeCount = article.nodes.length
posts.forEach((post, index) => {
  const html = readFileSync(resolve(CONTENT_DIR, `blog/${post.key}.html`), 'utf8')
  const body = extractPassages(html)
  const passages = [
    embedPassage(model, `${post.title}. ${post.description}`, post.description, null),
    ...body.map((p) => embedPassage(model, p.text, snippet(p.text), p.section)),
  ].filter(Boolean)

  const doc = {
    kind: 'post',
    context: 'Writing',
    id: post.path,
    order: nodeCount + index + 1,
    url: post.url,
    title: post.title,
    summary: post.description,
    concepts: [],
    search_terms: [],
    words: lexicalWords(body),
    passages,
  }
  const sections = buildPostSections(model, post, body)
  if (sections.length > 0) doc.sections = sections
  documents.push(doc)
})

const payload = JSON.stringify({ version: 2, model: modelInfo, nodes: documents })
writeFileSync(OUT_FILE, payload)
console.log(`Built search index: ${documents.length} documents (${payload.length} bytes).`)
