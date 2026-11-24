<script setup lang="ts">
import { computed } from 'vue'
import { getCv } from '#shared/data/cv'

const { t, locale } = useI18n()
const localePath = useLocalePath()
const route = useRoute()
const year = new Date().getFullYear()

const cv = computed(() => getCv(locale.value))
const normalizedPath = computed(() => route.path.replace(/^\/da(?=\/|$)/, '') || '/')

const isBlog = computed(
  () => normalizedPath.value === '/blog' || normalizedPath.value.startsWith('/blog/'),
)
const isAbout = computed(
  () => normalizedPath.value === '/about' || normalizedPath.value.startsWith('/about/'),
)
const isCv = computed(
  () => normalizedPath.value === '/cv' || normalizedPath.value.startsWith('/cv/'),
)
</script>

<template>
  <a class="skip-link" href="#content">{{ t('nav.skipToContent') }}</a>
  <div class="wrap">
    <header class="site-head">
      <NuxtLink class="brand" :to="localePath('/')">{{ cv.site.name }}</NuxtLink>
      <nav class="site-nav" :aria-label="t('nav.main')">
        <NuxtLink :to="localePath('/blog/')" :aria-current="isBlog ? 'page' : undefined">
          {{ t('nav.writing') }}
        </NuxtLink>
        <NuxtLink :to="localePath('/about/')" :aria-current="isAbout ? 'page' : undefined">
          {{ t('nav.about') }}
        </NuxtLink>
        <NuxtLink :to="localePath('/cv/')" :aria-current="isCv ? 'page' : undefined">
          {{ t('nav.cv') }}
        </NuxtLink>
        <a :href="cv.site.links.github">{{ t('nav.github') }}</a>
        <SearchPalette />
        <LocaleSwitcher />
        <ThemeSwitch />
      </nav>
    </header>

    <main id="content" tabindex="-1">
      <slot />
    </main>

    <footer class="site-foot">
      <span>&copy; {{ year }} {{ cv.site.name }}</span>
      <nav class="site-foot-links" :aria-label="t('nav.projects')">
        <a href="https://athletos.app">AthletOS</a>
        <a href="https://trongate.cloud">trongate.cloud</a>
      </nav>
      <a href="/rss.xml">RSS</a>
    </footer>
  </div>
</template>
