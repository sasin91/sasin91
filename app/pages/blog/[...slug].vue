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

const { data: post, error } = await useFetch<PostDetail>(() => `/api/posts/${slugPath.value}`, {
  key: () => `post-${currentLocale.value}-${slugPath.value}`,
  query: { lang: currentLocale },
})

if (error.value || !post.value) {
  throw createError({ statusCode: 404, statusMessage: 'Post not found', fatal: true })
}

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
      <time :datetime="post.dateIso">{{ post.dateLong }}</time>
    </div>

    <AskArticle
      mode="inline"
      :scope="`post:${post.path}`"
      :examples="post.ask"
      :placeholder="t('ask.placeholderInline')"
    />

    <div class="prose-body" v-html="post.body" />

    <DiagramLightbox v-if="post.hasDiagrams" :content-key="`${currentLocale}-${post.path}`" />
  </article>
</template>
