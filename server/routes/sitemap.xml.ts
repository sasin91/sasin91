import { cv } from '#shared/data/cv'

export default defineEventHandler(async (event) => {
  const [posts, article] = await Promise.all([getPostsMeta('en'), getAiArticleOutline('en')])

  const urls: { loc: string; lastmod?: string }[] = [
    { loc: `${cv.siteUrl}/` },
    { loc: `${cv.siteUrl}/da/` },
    { loc: `${cv.siteUrl}/blog/` },
    { loc: `${cv.siteUrl}/da/blog/` },
    { loc: `${cv.siteUrl}/about/` },
    { loc: `${cv.siteUrl}/da/about/` },
    { loc: `${cv.siteUrl}/cv/` },
    { loc: `${cv.siteUrl}/da/cv/` },
    { loc: `${cv.siteUrl}${article.url}`, lastmod: article.updatedIso },
    { loc: `${cv.siteUrl}/da${article.url}`, lastmod: article.updatedIso },
    { loc: `${cv.siteUrl}${article.url}timeline/`, lastmod: article.updatedIso },
    { loc: `${cv.siteUrl}/da${article.url}timeline/`, lastmod: article.updatedIso },
    { loc: `${cv.siteUrl}${article.url}ask/`, lastmod: article.updatedIso },
    { loc: `${cv.siteUrl}/da${article.url}ask/`, lastmod: article.updatedIso },
    ...posts.flatMap((post) => [
      { loc: `${cv.siteUrl}${post.url}`, lastmod: post.dateIso },
      { loc: `${cv.siteUrl}/da${post.url}`, lastmod: post.dateIso },
    ]),
  ]

  const body = urls
    .map((entry) =>
      entry.lastmod
        ? `  <url><loc>${entry.loc}</loc><lastmod>${entry.lastmod}</lastmod></url>`
        : `  <url><loc>${entry.loc}</loc></url>`,
    )
    .join('\n')

  const xml = `<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
${body}
</urlset>
`

  setHeader(event, 'content-type', 'application/xml; charset=utf-8')
  return xml
})
