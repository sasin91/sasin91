import { normalizeLocale } from '#shared/data/cv'

export default defineEventHandler(async (event) => {
  const query = getQuery(event)
  const locale = normalizeLocale(typeof query.lang === 'string' ? query.lang : 'en')
  if (query.outline === '1') {
    return getAiArticleOutline(locale)
  }
  return getAiArticleFull(locale)
})
