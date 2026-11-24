<script setup lang="ts">
import { computed } from 'vue'
import { getCv, normalizeLocale } from '#shared/data/cv'
import type { PostDetail } from '#shared/types/content'

const { t, locale } = useI18n()
const currentLocale = computed(() => normalizeLocale(locale.value))
const cv = computed(() => getCv(currentLocale.value))
const route = useRoute()

const slugPath = computed(() => {
  const raw = route.params.slug
  const segments = Array.isArray(raw) ? raw : raw ? [raw] : []
  return segments.filter(Boolean).join('/')
})

const { data: post, error } = await useFetch<PostDetail>(() => `/api/posts/${slugPath.value}`)

if (error.value || !post.value) {
  throw createError({ statusCode: 404, statusMessage: 'Post not found', fatal: true })
}

const formattedDate = computed(() => {
  if (!post.value) return ''
  if (currentLocale.value === 'en') return post.value.dateLong
  const [year, month, day] = post.value.dateIso.split('-').map(Number)
  if (!year || !month || !day) return post.value.dateLong
  return new Intl.DateTimeFormat('da-DK', {
    day: 'numeric',
    month: 'long',
    year: 'numeric',
    timeZone: 'UTC',
  }).format(new Date(Date.UTC(year, month - 1, day)))
})

useSiteSeo({
  title: `${post.value.title} — ${cv.value.site.name}`,
  description: post.value.description,
  path: post.value.url,
  ogType: 'article',
  image: post.value.ogImage,
  imageAlt: post.value.ogImageAlt,
})
</script>

<template>
  <article v-if="post" class="prose">
    <div v-if="post.heroHtml" class="prose-hero" v-html="post.heroHtml" />
    <img v-else-if="post.hero" :src="post.hero" :alt="post.heroAlt ?? ''" />

    <h1>{{ post.title }}</h1>
    <div class="meta">
      <time :datetime="post.dateIso">{{ formattedDate }}</time>
    </div>
    <p v-if="currentLocale === 'da'" class="article-lang-note">
      {{ t('blog.englishOnlyNote') }}
    </p>

    <AskArticle
      mode="inline"
      :scope="`post:${post.path}`"
      :examples="post.ask"
      :placeholder="t('ask.placeholderInline')"
    />

    <div class="prose-body" v-html="post.body" />

    <DiagramLightbox v-if="post.hasDiagrams" :content-key="post.path" />
  </article>
</template>
