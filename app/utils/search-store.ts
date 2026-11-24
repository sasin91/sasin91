import type { LocaleCode } from '#shared/data/cv'
import {
  fetchIndex,
  fetchModel,
  prepareIndex,
  type PreparedSearchIndex,
  type RawSearchIndex,
  type StaticModel,
} from '#shared/retrieval'

export function searchIndexUrl(locale: LocaleCode = 'en'): string {
  return locale === 'da' ? '/search/index.da.json' : '/search/index.json'
}

const rawIndexPromises: Partial<Record<LocaleCode, Promise<RawSearchIndex>>> = {}
const preparedSiteIndexes: Partial<Record<LocaleCode, PreparedSearchIndex>> = {}
let modelPromise: Promise<StaticModel> | null = null
let loadedModel: StaticModel | null = null

export function loadRawIndex(locale: LocaleCode = 'en'): Promise<RawSearchIndex> {
  if (!rawIndexPromises[locale]) {
    rawIndexPromises[locale] = fetchIndex(searchIndexUrl(locale)).catch((error) => {
      delete rawIndexPromises[locale]
      throw error
    })
  }
  return rawIndexPromises[locale]!
}

export async function loadSiteIndex(locale: LocaleCode = 'en'): Promise<PreparedSearchIndex> {
  if (preparedSiteIndexes[locale]) return preparedSiteIndexes[locale]!
  const raw = await loadRawIndex(locale)
  preparedSiteIndexes[locale] = prepareIndex(raw)
  return preparedSiteIndexes[locale]!
}

export async function loadEmbeddingModel(
  raw?: RawSearchIndex,
  locale: LocaleCode = 'en',
): Promise<StaticModel> {
  if (loadedModel) return loadedModel
  if (!modelPromise) {
    modelPromise = (async () => {
      const index = raw ?? (await loadRawIndex(locale))
      const model = await fetchModel(index.model)
      loadedModel = model
      return model
    })().catch((error) => {
      modelPromise = null
      throw error
    })
  }
  return modelPromise
}

export function getLoadedModel(): StaticModel | null {
  return loadedModel
}

export function resetSearchCache() {
  delete rawIndexPromises.en
  delete rawIndexPromises.da
  delete preparedSiteIndexes.en
  delete preparedSiteIndexes.da
  modelPromise = null
  loadedModel = null
}
