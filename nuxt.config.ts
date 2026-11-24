import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'

const postsMeta = JSON.parse(
  readFileSync(resolve(import.meta.dirname, 'content/blog/meta.json'), 'utf8'),
) as Array<{ url: string }>

const prerenderRoutes = [
  '/',
  '/da/',
  '/about/',
  '/da/about/',
  '/cv/',
  '/da/cv/',
  '/blog/',
  '/da/blog/',
  '/blog/ai/',
  '/da/blog/ai/',
  '/blog/ai/timeline/',
  '/da/blog/ai/timeline/',
  '/blog/ai/ask/',
  '/da/blog/ai/ask/',
  '/rss.xml',
  '/sitemap.xml',
  '/cv_jonas_hansen_software_developer.pdf',
  '/da/cv_jonas_hansen_software_developer.pdf',
  ...postsMeta.flatMap((post) => [post.url, `/da${post.url}`]),
]

export default defineNuxtConfig({
  compatibilityDate: '2026-11-24',
  devtools: { enabled: true },
  modules: ['@ui-sdk/vue', '@nuxtjs/i18n'],
  css: [
    '@ui-sdk/core/web.css',
    '~/assets/css/site.css',
    '~/assets/css/syntax.css',
  ],
  i18n: {
    defaultLocale: 'en',
    strategy: 'prefix_except_default',
    baseUrl: 'https://sasin91.xyz',
    detectBrowserLanguage: false,
    locales: [
      { code: 'en', language: 'en-GB', name: 'English', file: 'en.json' },
      { code: 'da', language: 'da-DK', name: 'Dansk', file: 'da.json' },
    ],
  },
  routeRules: {
    '/': { prerender: true },
    '/da': { prerender: true },
    '/about': { prerender: true },
    '/da/about': { prerender: true },
    '/cv': { prerender: true },
    '/da/cv': { prerender: true },
    '/blog/**': { prerender: true },
    '/da/blog/**': { prerender: true },
    '/rss.xml': { prerender: true },
    '/sitemap.xml': { prerender: true },
    '/cv_jonas_hansen_software_developer.pdf': { prerender: true },
    '/da/cv_jonas_hansen_software_developer.pdf': { prerender: true },
    '/search/**': {
      headers: { 'cache-control': 'public, max-age=31536000, immutable' },
    },
  },
  nitro: {
    preset: 'cloudflare-module',
    cloudflare: {
      nodeCompat: true,
    },
    prerender: {
      crawlLinks: false,
      routes: prerenderRoutes,
    },
    serverAssets: [
      {
        baseName: 'content',
        dir: resolve(import.meta.dirname, 'content'),
      },
    ],
  },
  app: {
    head: {
      link: [
        { rel: 'icon', href: '/favicon.png', sizes: 'any' },
        { rel: 'apple-touch-icon', href: '/apple-touch-icon.png' },
        {
          rel: 'alternate',
          type: 'application/rss+xml',
          title: 'Jonas Hansen',
          href: '/rss.xml',
        },
      ],
    },
  },
})
