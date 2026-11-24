<script setup lang="ts">
import { computed } from 'vue'
import { getCv, normalizeLocale } from '#shared/data/cv'
import type { AiArticleOutline, AiNodeMeta } from '#shared/types/content'

const { t, locale } = useI18n()
const localePath = useLocalePath()
const currentLocale = computed(() => normalizeLocale(locale.value))
const cv = computed(() => getCv(currentLocale.value))

const { data: article } = await useFetch<AiArticleOutline>('/api/ai-article', {
  key: () => `ai-article-outline-${currentLocale.value}`,
  query: { outline: '1', lang: currentLocale },
})

if (!article.value) {
  throw createError({ statusCode: 500, statusMessage: 'Could not load AI article', fatal: true })
}

const nodesById = computed(() => {
  const map = new Map<string, AiNodeMeta>()
  for (const node of article.value?.nodes ?? []) {
    map.set(node.id, node)
  }
  return map
})

const partsWithNodes = computed(() =>
  (article.value?.parts ?? []).map((part) => {
    const nodes = part.nodeIds
      .map((id) => nodesById.value.get(id))
      .filter((node): node is AiNodeMeta => Boolean(node))
    return {
      ...part,
      start: nodes[0]?.order ?? 1,
      nodes,
    }
  }),
)

useSiteSeo({
  title: `${t('ai.timeline')}: ${article.value.title} — ${cv.value.site.name}`,
  description: `${article.value.description} ${t('ai.timelineLede')}`,
  path: `${article.value.url}timeline/`,
  ogType: 'website',
})
</script>

<template>
  <template v-if="article">
    <h1>{{ article.title }}</h1>
    <p class="lede">{{ t('ai.timelineLede') }}</p>

    <KnowledgeViewNav current="timeline" />

    <section v-for="part in partsWithNodes" :key="part.id" class="timeline-part">
      <h2>{{ part.title }}</h2>
      <ol class="timeline" :start="part.start">
        <li v-for="node in part.nodes" :key="node.id">
          <NuxtLink :to="`${localePath(article.url)}#${node.id}`">
            <span class="timeline-title">{{ node.title }}</span>
            <span class="timeline-summary">{{ node.summary }}</span>
          </NuxtLink>
        </li>
      </ol>
    </section>
  </template>
</template>
