export default defineEventHandler(async (event) => {
  const query = getQuery(event)
  if (query.outline === '1') {
    return getAiArticleOutline()
  }
  return getAiArticleFull()
})
