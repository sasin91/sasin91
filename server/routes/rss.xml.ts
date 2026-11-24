import { cv } from '#shared/data/cv'

function xmlEscape(value: string): string {
  return value
    .replaceAll('&', '&amp;')
    .replaceAll('<', '&lt;')
    .replaceAll('>', '&gt;')
    .replaceAll('"', '&quot;')
    .replaceAll("'", '&apos;')
}

export default defineEventHandler(async (event) => {
  const posts = await getPostsMeta()
  const lastBuildDate = posts[0]?.dateRfc2822 ?? new Date().toUTCString()

  const items = posts
    .map(
      (post) => `    <item>
      <title>${xmlEscape(post.title)}</title>
      <link>${xmlEscape(cv.siteUrl + post.url)}</link>
      <guid isPermaLink="true">${xmlEscape(cv.siteUrl + post.url)}</guid>
      <pubDate>${xmlEscape(post.dateRfc2822)}</pubDate>
      <description>${xmlEscape(post.description)}</description>
    </item>`,
    )
    .join('\n')

  const xml = `<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0" xmlns:atom="http://www.w3.org/2005/Atom">
  <channel>
    <title>${xmlEscape(cv.site.name)}</title>
    <link>${xmlEscape(cv.siteUrl)}/</link>
    <description>Notes on things I built and what broke on the way.</description>
    <language>en</language>
    <lastBuildDate>${xmlEscape(lastBuildDate)}</lastBuildDate>
    <atom:link href="${xmlEscape(cv.siteUrl)}/rss.xml" rel="self" type="application/rss+xml" />
${items}
  </channel>
</rss>
`

  setHeader(event, 'content-type', 'application/rss+xml; charset=utf-8')
  return xml
})
