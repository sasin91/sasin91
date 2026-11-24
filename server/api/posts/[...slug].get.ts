import { normalizeLocale } from '#shared/data/cv'

export default defineEventHandler(async (event) => {
  const slugParam = getRouterParam(event, 'slug') ?? ''
  const query = getQuery(event)
  const locale = normalizeLocale(typeof query.lang === 'string' ? query.lang : 'en')
  const post = await getPostByPath(`blog/${slugParam}`, locale)
  if (!post) {
    throw createError({ statusCode: 404, statusMessage: 'Post not found' })
  }
  return post
})
