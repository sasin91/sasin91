import {
  fetchIndex,
  fetchModel,
  prepareIndex,
  type PreparedSearchIndex,
  type RawSearchIndex,
  type StaticModel,
} from '#shared/retrieval'

export const SEARCH_INDEX_URL = '/search/index.json'

let rawIndexPromise: Promise<RawSearchIndex> | null = null
let preparedSiteIndex: PreparedSearchIndex | null = null
let modelPromise: Promise<StaticModel> | null = null
let loadedModel: StaticModel | null = null

export function loadRawIndex(): Promise<RawSearchIndex> {
  if (!rawIndexPromise) {
    rawIndexPromise = fetchIndex(SEARCH_INDEX_URL).catch((error) => {
      rawIndexPromise = null
      throw error
    })
  }
  return rawIndexPromise
}

export async function loadSiteIndex(): Promise<PreparedSearchIndex> {
  if (preparedSiteIndex) return preparedSiteIndex
  const raw = await loadRawIndex()
  preparedSiteIndex = prepareIndex(raw)
  return preparedSiteIndex
}

export async function loadEmbeddingModel(raw?: RawSearchIndex): Promise<StaticModel> {
  if (loadedModel) return loadedModel
  if (!modelPromise) {
    modelPromise = (async () => {
      const index = raw ?? (await loadRawIndex())
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
  rawIndexPromise = null
  preparedSiteIndex = null
  modelPromise = null
  loadedModel = null
}
