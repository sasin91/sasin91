<script setup lang="ts">
import { computed } from 'vue'
import { getCv, normalizeLocale } from '#shared/data/cv'
import type { PostMeta, WritingEntry } from '#shared/types/content'

const { t, locale } = useI18n()
const localePath = useLocalePath()
const currentLocale = computed(() => normalizeLocale(locale.value))
const cv = computed(() => getCv(currentLocale.value))

const { data } = await useFetch<{ posts: PostMeta[]; writing: WritingEntry[] }>('/api/posts', {
  key: () => `blog-listing-${currentLocale.value}`,
  query: { lang: currentLocale },
})

useSiteSeo({
  title: `${t('blog.title')} — ${cv.value.site.name}`,
  description: t('blog.lede'),
  path: '/blog/',
  ogType: 'website',
})
</script>

<template>
  <h1>{{ t('blog.title') }}</h1>
  <p class="lede">{{ t('blog.lede') }}</p>

  <ul class="post-list">
    <li v-for="entry in data?.writing ?? []" :key="entry.url">
      <div class="meta">
        <time :datetime="entry.dateIso">{{ entry.dateLong }}</time>
        <template v-if="entry.label"> · {{ entry.label }}</template>
      </div>
      <h2>
        <NuxtLink :to="localePath(entry.url)">{{ entry.title }}</NuxtLink>
      </h2>
      <p>{{ entry.description }}</p>
    </li>
  </ul>
</template>
