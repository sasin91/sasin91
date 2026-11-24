import { existsSync, readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, expect, test } from 'vitest'
import { cv, formatCvMonth, getCv } from '#shared/data/cv'
import type { AiArticleOutline, PostMeta } from '#shared/types/content'
import type { RawSearchIndex } from '#shared/retrieval'

const ROOT = process.cwd()

function flattenKeys(obj: Record<string, unknown>, prefix = ''): string[] {
  const out: string[] = []
  for (const [key, value] of Object.entries(obj)) {
    const full = prefix ? `${prefix}.${key}` : key
    if (value && typeof value === 'object' && !Array.isArray(value)) {
      out.push(...flattenKeys(value as Record<string, unknown>, full))
    } else {
      out.push(full)
    }
  }
  return out.sort()
}

describe('published content integrity', () => {
  test('all 9 blog posts exist with HTML bodies and valid metadata', () => {
    const posts = JSON.parse(
      readFileSync(resolve(ROOT, 'content/blog/meta.json'), 'utf8'),
    ) as PostMeta[]

    expect(posts).toHaveLength(9)
    for (const post of posts) {
      expect(existsSync(resolve(ROOT, `content/blog/${post.key}.html`))).toBe(true)
      expect(post.date).toMatch(/^\d{4}-\d{2}-\d{2}$/)
      expect(post.url).toBe(`/${post.path}/`)
      if (post.heroIsSvg) {
        expect(existsSync(resolve(ROOT, `content/blog/${post.key}.hero.html`))).toBe(true)
      }
    }
  })

  test('AI knowledge article includes all 32 nodes including the 3 newer ones', () => {
    const article = JSON.parse(
      readFileSync(resolve(ROOT, 'content/ai/article.json'), 'utf8'),
    ) as AiArticleOutline

    expect(article.parts).toHaveLength(6)
    expect(article.nodes).toHaveLength(32)

    const ids = new Set(article.nodes.map((n) => n.id))
    expect(ids.has('quantization')).toBe(true)
    expect(ids.has('local-inference')).toBe(true)
    expect(ids.has('hallucination')).toBe(true)

    for (const node of article.nodes) {
      expect(existsSync(resolve(ROOT, `content/ai/nodes/${node.id}.html`))).toBe(true)
      for (const link of [...node.buildsOn, ...node.seeAlso]) {
        expect(ids.has(link.id)).toBe(true)
      }
    }

    for (const path of article.learningPaths) {
      for (const nodeId of path.nodes) {
        expect(ids.has(nodeId)).toBe(true)
      }
    }
  })

  test('CV data and dates localize cleanly across en and da', () => {
    const enCv = getCv('en')
    const daCv = getCv('da')

    expect(enCv).toBe(cv)
    expect(formatCvMonth('2024-09', 'en')).toBe('September 2024')
    expect(formatCvMonth('2024-09', 'da')).toBe('september 2024')
    expect(formatCvMonth('2026-02', 'da')).toBe('februar 2026')
    expect(enCv.pdfPath).toBe('/cv_jonas_hansen_software_developer.pdf')
    expect(daCv.pdfPath).toBe('/da/cv_jonas_hansen_software_developer.pdf')
    expect(daCv.roles).toHaveLength(enCv.roles.length)
    expect(daCv.skills).toHaveLength(enCv.skills.length)
    expect(daCv.education).toHaveLength(enCv.education.length)

    for (const loc of ['en', 'da'] as const) {
      const data = getCv(loc)
      for (const role of data.roles) {
        expect(formatCvMonth(role.start, loc)).toBeTruthy()
        if (role.end) expect(formatCvMonth(role.end, loc)).toBeTruthy()
      }
      for (const edu of data.education) {
        expect(formatCvMonth(edu.start, loc)).toBeTruthy()
        if (edu.end) expect(formatCvMonth(edu.end, loc)).toBeTruthy()
      }
    }
  })

  test('en and da locale files define the exact same translation keys', () => {
    const en = JSON.parse(
      readFileSync(resolve(ROOT, 'i18n/locales/en.json'), 'utf8'),
    ) as Record<string, unknown>
    const da = JSON.parse(
      readFileSync(resolve(ROOT, 'i18n/locales/da.json'), 'utf8'),
    ) as Record<string, unknown>

    expect(flattenKeys(da)).toEqual(flattenKeys(en))
  })

  test('search index covers all 32 AI nodes and 9 posts', () => {
    const index = JSON.parse(
      readFileSync(resolve(ROOT, 'public/search/index.json'), 'utf8'),
    ) as RawSearchIndex

    expect(index.version).toBe(2)
    expect(index.nodes.filter((n) => n.kind === 'ai')).toHaveLength(32)
    expect(index.nodes.filter((n) => n.kind === 'post')).toHaveLength(9)
    expect(existsSync(resolve(ROOT, 'public/search/model.bin'))).toBe(true)
    expect(existsSync(resolve(ROOT, 'public/search/vocab.txt'))).toBe(true)
  })
})
