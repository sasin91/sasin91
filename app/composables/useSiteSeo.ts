import { getCv, normalizeLocale } from '#shared/data/cv'

export interface SiteSeoOptions {
  title: string
  description: string
  path: string
  ogType?: 'website' | 'article'
  image?: string | null
  imageAlt?: string | null
}

export function useSiteSeo(options: SiteSeoOptions) {
  const { locale } = useI18n()
  const localePath = useLocalePath()
  const currentLocale = normalizeLocale(locale.value)
  const cv = getCv(currentLocale)
  const localizedPath = localePath(options.path)
  const url = `${cv.siteUrl}${localizedPath.endsWith('/') ? localizedPath : `${localizedPath}/`}`

  useHead({
    title: options.title,
    link: [{ rel: 'canonical', href: url }],
  })

  useSeoMeta({
    description: options.description,
    ogType: options.ogType ?? 'website',
    ogSiteName: cv.site.name,
    ogTitle: options.title,
    ogDescription: options.description,
    ogUrl: url,
    ogLocale: currentLocale === 'da' ? 'da_DK' : 'en_GB',
    ogImage: options.image ?? undefined,
    ogImageAlt: options.image && options.imageAlt ? options.imageAlt : undefined,
    twitterCard: options.image ? 'summary_large_image' : 'summary',
  })
}
