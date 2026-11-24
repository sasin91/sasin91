import { cv } from '#shared/data/cv'
import type {
  AiArticleFull,
  AiArticleOutline,
  PostDetail,
  PostMeta,
  WritingEntry,
} from '#shared/types/content'

const ARTICLE_LABEL = 'Long read, with a timeline and Ask'

let cachedPosts: PostMeta[] | null = null
let cachedArticleOutline: AiArticleOutline | null = null

function contentStorage() {
  return useStorage('assets:content')
}

async function readJson<T>(key: string): Promise<T> {
  const value = await contentStorage().getItem(key)
  if (value === null || value === undefined) {
    throw new Error(`Missing content asset: ${key}`)
  }
  if (typeof value === 'string') {
    return JSON.parse(value) as T
  }
  return value as T
}

const utf8Decoder = new TextDecoder()

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

export async function getPostsMeta(): Promise<PostMeta[]> {
  if (!cachedPosts) {
    cachedPosts = await readJson<PostMeta[]>('blog:meta.json')
  }
  return cachedPosts
}

export async function getPostByPath(rawPath: string): Promise<PostDetail | null> {
  const normalized = rawPath.replace(/^\/+|\/+$/g, '')
  const posts = await getPostsMeta()
  const meta = posts.find((post) => post.path === normalized)
  if (!meta) return null

  const body = (await readText(`blog:${meta.key}.html`)) ?? ''
  const heroHtml = meta.heroIsSvg ? await readText(`blog:${meta.key}.hero.html`) : null
  const { ogImage, ogImageAlt } = resolveOgImage(meta)

  return {
    ...meta,
    body,
    heroHtml,
    ogImage,
    ogImageAlt,
  }
}

export async function getAiArticleOutline(): Promise<AiArticleOutline> {
  if (!cachedArticleOutline) {
    cachedArticleOutline = await readJson<AiArticleOutline>('ai:article.json')
  }
  return cachedArticleOutline
}

export async function getAiArticleFull(): Promise<AiArticleFull> {
  const outline = await getAiArticleOutline()
  const nodes = await Promise.all(
    outline.nodes.map(async (node) => ({
      ...node,
      body: (await readText(`ai:nodes:${node.id}.html`)) ?? '',
    })),
  )
  return { ...outline, nodes }
}

export async function getWritingListing(): Promise<WritingEntry[]> {
  const [posts, article] = await Promise.all([getPostsMeta(), getAiArticleOutline()])
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
        label: ARTICLE_LABEL,
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
      label: ARTICLE_LABEL,
    })
  }

  return entries
}
