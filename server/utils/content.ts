import { cv, normalizeLocale, type LocaleCode } from '#shared/data/cv'
import type {
  AiArticleFull,
  AiArticleOutline,
  PostDetail,
  PostMeta,
  WritingEntry,
} from '#shared/types/content'

const ARTICLE_LABEL: Record<LocaleCode, string> = {
  en: 'Long read, with a timeline and Ask',
  da: 'Lang artikel med tidslinje og Spørg',
}

const cachedPosts: Partial<Record<LocaleCode, PostMeta[]>> = {}
const cachedArticleOutline: Partial<Record<LocaleCode, AiArticleOutline>> = {}
const utf8Decoder = new TextDecoder()

function contentStorage() {
  return useStorage('assets:content')
}

async function readJson<T>(key: string): Promise<T | null> {
  const value = await contentStorage().getItem(key)
  if (value === null || value === undefined) return null
  if (typeof value === 'string') {
    return JSON.parse(value) as T
  }
  return value as T
}

async function readText(key: string): Promise<string | null> {
  const raw = await contentStorage().getItemRaw(key)
  if (raw === null || raw === undefined) return null
  if (typeof raw === 'string') return raw
  if (raw instanceof Uint8Array) {
    return utf8Decoder.decode(raw)
  }
  return String(raw)
}

function isRaster(path: string): boolean {
  return ['.png', '.jpg', '.jpeg', '.webp'].some((ext) => path.endsWith(ext))
}

function resolveOgImage(post: PostMeta): { ogImage: string | null; ogImageAlt: string | null } {
  if (post.card && isRaster(post.card)) {
    return { ogImage: `${cv.siteUrl}${post.card}`, ogImageAlt: post.cardAlt }
  }
  if (post.hero && isRaster(post.hero)) {
    return { ogImage: `${cv.siteUrl}${post.hero}`, ogImageAlt: post.heroAlt }
  }
  return { ogImage: null, ogImageAlt: null }
}

function escapeAttr(value: string): string {
  return value.replaceAll('&', '&amp;').replaceAll('"', '&quot;').replaceAll('<', '&lt;')
}

export async function getPostsMeta(locale: LocaleCode = 'en'): Promise<PostMeta[]> {
  const loc = normalizeLocale(locale)
  if (!cachedPosts[loc]) {
    const localized = loc === 'da' ? await readJson<PostMeta[]>('blog:meta.da.json') : null
    const data = localized ?? (await readJson<PostMeta[]>('blog:meta.json'))
    if (!data) throw new Error('Missing content asset: blog:meta.json')
    cachedPosts[loc] = data
  }
  return cachedPosts[loc]!
}

export async function getPostByPath(
  rawPath: string,
  locale: LocaleCode = 'en',
): Promise<PostDetail | null> {
  const loc = normalizeLocale(locale)
  const normalized = rawPath.replace(/^\/+|\/+$/g, '')
  const posts = await getPostsMeta(loc)
  const meta = posts.find((post) => post.path === normalized)
  if (!meta) return null

  const localizedBody = loc === 'da' ? await readText(`blog:da:${meta.key}.html`) : null
  const body = localizedBody ?? (await readText(`blog:${meta.key}.html`)) ?? ''

  let heroHtml = meta.heroIsSvg ? await readText(`blog:${meta.key}.hero.html`) : null
  if (heroHtml && loc === 'da' && meta.heroAlt) {
    heroHtml = heroHtml.replace(/aria-label="[^"]*"/, `aria-label="${escapeAttr(meta.heroAlt)}"`)
  }
  const { ogImage, ogImageAlt } = resolveOgImage(meta)

  return {
    ...meta,
    body,
    heroHtml,
    ogImage,
    ogImageAlt,
  }
}

export async function getAiArticleOutline(locale: LocaleCode = 'en'): Promise<AiArticleOutline> {
  const loc = normalizeLocale(locale)
  if (!cachedArticleOutline[loc]) {
    const localized = loc === 'da' ? await readJson<AiArticleOutline>('ai:article.da.json') : null
    const data = localized ?? (await readJson<AiArticleOutline>('ai:article.json'))
    if (!data) throw new Error('Missing content asset: ai:article.json')
    cachedArticleOutline[loc] = data
  }
  return cachedArticleOutline[loc]!
}

export async function getAiArticleFull(locale: LocaleCode = 'en'): Promise<AiArticleFull> {
  const loc = normalizeLocale(locale)
  const outline = await getAiArticleOutline(loc)
  const nodes = await Promise.all(
    outline.nodes.map(async (node) => {
      const localized = loc === 'da' ? await readText(`ai:da:nodes:${node.id}.html`) : null
      const body = localized ?? (await readText(`ai:nodes:${node.id}.html`)) ?? ''
      return { ...node, body }
    }),
  )
  return { ...outline, nodes }
}

export async function getWritingListing(locale: LocaleCode = 'en'): Promise<WritingEntry[]> {
  const loc = normalizeLocale(locale)
  const [posts, article] = await Promise.all([getPostsMeta(loc), getAiArticleOutline(loc)])
  const entries: WritingEntry[] = []
  let placed = false

  for (const post of posts) {
    if (!placed && post.listedDate < article.published) {
      entries.push({
        url: article.url,
        title: article.title,
        dateIso: article.publishedIso,
        dateLong: article.publishedLong,
        description: article.description,
        label: ARTICLE_LABEL[loc],
      })
      placed = true
    }
    entries.push({
      url: post.url,
      title: post.title,
      dateIso: post.dateIso,
      dateLong: post.dateLong,
      description: post.description,
      label: null,
    })
  }

  if (!placed) {
    entries.push({
      url: article.url,
      title: article.title,
      dateIso: article.publishedIso,
      dateLong: article.publishedLong,
      description: article.description,
      label: ARTICLE_LABEL[loc],
    })
  }

  return entries
}
