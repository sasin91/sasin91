<script setup lang="ts">
import { computed } from 'vue'
import { getCv, normalizeLocale } from '#shared/data/cv'
import type { PostMeta, WritingEntry } from '#shared/types/content'

const { t, locale } = useI18n()
const localePath = useLocalePath()
const currentLocale = computed(() => normalizeLocale(locale.value))
const cv = computed(() => getCv(currentLocale.value))

const { data } = await useFetch<{ posts: PostMeta[]; writing: WritingEntry[] }>('/api/posts')

function formatDate(iso: string, fallback: string): string {
  if (currentLocale.value === 'en') return fallback
  const [year, month, day] = iso.split('-').map(Number)
  if (!year || !month || !day) return fallback
  return new Intl.DateTimeFormat('da-DK', {
    day: 'numeric',
    month: 'long',
    year: 'numeric',
    timeZone: 'UTC',
  }).format(new Date(Date.UTC(year, month - 1, day)))
}

useSiteSeo({
  title: `${cv.value.site.name} — ${cv.value.site.title}`,
  description: `${cv.value.site.title} in ${cv.value.contact.town}. ${cv.value.site.stack.join(', ')}.`,
  path: '/',
  ogType: 'website',
})
</script>

<template>
  <section class="intro">
    <h1>{{ cv.site.name }}</h1>
    <p class="lede">{{ cv.site.title }}</p>
    <p class="stack sep-list">
      <template v-for="(item, idx) in cv.site.stack" :key="item">
        <span>{{ item }}</span><template v-if="idx < cv.site.stack.length - 1">{{ ' ' }}</template>
      </template>
    </p>
    <p v-if="cv.site.available" class="available">
      {{ cv.site.availableNote }} —
      <a :href="cv.site.links.email">{{ t('home.getInTouch') }}</a>
    </p>

    <div class="social">
      <a :href="cv.site.links.github">{{ t('nav.github') }}</a>
      <a :href="cv.site.links.linkedin">{{ t('nav.linkedin') }}</a>
      <a :href="cv.site.links.email">{{ t('nav.email') }}</a>
    </div>
  </section>

  <h2>{{ t('home.writingHeading') }}</h2>
  <ul class="post-list">
    <li v-for="entry in data?.writing ?? []" :key="entry.url">
      <div class="meta">
        <time :datetime="entry.dateIso">{{ formatDate(entry.dateIso, entry.dateLong) }}</time>
        <template v-if="entry.label"> · {{ t('blog.articleLabel') }}</template>
      </div>
      <h3>
        <NuxtLink :to="localePath(entry.url)">{{ entry.title }}</NuxtLink>
      </h3>
      <p>{{ entry.description }}</p>
    </li>
  </ul>
</template>
