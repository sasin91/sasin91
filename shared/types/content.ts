export interface PostMeta {
  key: string
  path: string
  url: string
  title: string
  date: string
  dateIso: string
  dateLong: string
  dateRfc2822: string
  listedDate: string
  description: string
  hero: string | null
  heroAlt: string | null
  heroIsSvg: boolean
  card: string | null
  cardAlt: string | null
  series: string | null
  part: number | null
  ask: string[]
  hasSyntax: boolean
  hasDiagrams: boolean
}

export interface PostDetail extends PostMeta {
  body: string
  heroHtml: string | null
  ogImage: string | null
  ogImageAlt: string | null
}

export interface AiNodeLink {
  id: string
  title: string
}

export interface AiNodeMeta {
  id: string
  order: number
  partId: string | null
  title: string
  summary: string
  concepts: string[]
  searchTerms: string[]
  learningObjectives: string[]
  misconceptions: string[]
  buildsOn: AiNodeLink[]
  seeAlso: AiNodeLink[]
}

export interface AiNodeFull extends AiNodeMeta {
  body: string
}

export interface AiPartOutline {
  id: string
  title: string
  nodeIds: string[]
}

export interface AiLearningPath {
  id: string
  title: string
  nodes: string[]
}

export interface AiArticleOutline {
  path: string
  url: string
  title: string
  description: string
  published: string
  publishedIso: string
  publishedLong: string
  updated: string
  updatedIso: string
  updatedLong: string
  wasUpdated: boolean
  parts: AiPartOutline[]
  nodes: AiNodeMeta[]
  learningPaths: AiLearningPath[]
}

export interface AiArticleFull extends Omit<AiArticleOutline, 'nodes'> {
  nodes: AiNodeFull[]
}

export interface WritingEntry {
  url: string
  title: string
  dateIso: string
  dateLong: string
  description: string
  label: string | null
}
