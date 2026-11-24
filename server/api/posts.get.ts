export default defineEventHandler(async () => {
  const [posts, writing] = await Promise.all([getPostsMeta(), getWritingListing()])
  return { posts, writing }
})
