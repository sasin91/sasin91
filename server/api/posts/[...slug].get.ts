export default defineEventHandler(async (event) => {
  const slugParam = getRouterParam(event, 'slug') ?? ''
  const post = await getPostByPath(`blog/${slugParam}`)
  if (!post) {
    throw createError({ statusCode: 404, statusMessage: 'Post not found' })
  }
  return post
})
