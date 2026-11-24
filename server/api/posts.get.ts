import { normalizeLocale } from '#shared/data/cv'

export default defineEventHandler(async (event) => {
  const query = getQuery(event)
  const locale = normalizeLocale(typeof query.lang === 'string' ? query.lang : 'en')
  const [posts, writing] = await Promise.all([getPostsMeta(locale), getWritingListing(locale)])
  return { posts, writing }
})
