<script setup lang="ts">
import { computed } from 'vue'
import { getCv, normalizeLocale } from '#shared/data/cv'

const { t, locale } = useI18n()
const localePath = useLocalePath()
const cv = computed(() => getCv(normalizeLocale(locale.value)))

useSiteSeo({
  title: `${t('about.title')} — ${cv.value.site.name}`,
  description: cv.value.about[0] ?? cv.value.site.title,
  path: '/about/',
  ogType: 'website',
})
</script>

<template>
  <h1>{{ t('about.title') }}</h1>
  <p class="lede">{{ cv.site.title }}. {{ cv.site.availableNote }}.</p>

  <p v-for="paragraph in cv.about" :key="paragraph">{{ paragraph }}</p>

  <p>
    <NuxtLink :to="localePath('/cv/')">{{ t('about.readFullCv') }}</NuxtLink>{{ t('about.orTakePdf') }}
    <a :href="cv.pdfPath">{{ t('about.pdf') }}</a>.
  </p>
</template>
