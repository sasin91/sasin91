<script setup lang="ts">
import { computed } from 'vue'
import { getCv, normalizeLocale } from '#shared/data/cv'
import type { AiArticleFull, AiNodeFull } from '#shared/types/content'

const { t, locale } = useI18n()
const localePath = useLocalePath()
const currentLocale = computed(() => normalizeLocale(locale.value))
const cv = computed(() => getCv(currentLocale.value))

const { data: article } = await useFetch<AiArticleFull>('/api/ai-article')

if (!article.value) {
  throw createError({ statusCode: 500, statusMessage: 'Could not load AI article', fatal: true })
}

const nodesById = computed(() => {
  const map = new Map<string, AiNodeFull>()
  for (const node of article.value?.nodes ?? []) {
    map.set(node.id, node)
  }
  return map
})

const partsWithNodes = computed(() =>
  (article.value?.parts ?? []).map((part) => ({
    ...part,
    nodes: part.nodeIds
      .map((id) => nodesById.value.get(id))
      .filter((node): node is AiNodeFull => Boolean(node)),
  })),
)

const hasDiagrams = computed(() =>
  (article.value?.nodes ?? []).some((node) => node.body.includes('<figure class="diagram"')),
)

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
  title: `${article.value.title} — ${cv.value.site.name}`,
  description: article.value.description,
  path: article.value.url,
  ogType: 'article',
})
</script>

<template>
  <article v-if="article" class="prose knowledge">
    <h1>{{ article.title }}</h1>
    <p class="lede">{{ article.description }}</p>
    <div class="meta">
      <time :datetime="article.publishedIso">
        {{ formatDate(article.publishedIso, article.publishedLong) }}
      </time>
      <template v-if="article.wasUpdated">
        · {{ t('ai.updated') }}
        <time :datetime="article.updatedIso">
          {{ formatDate(article.updatedIso, article.updatedLong) }}
        </time>
      </template>
      · {{ t('ai.sectionsCount', { count: article.nodes.length }) }}
    </div>
    <p v-if="currentLocale === 'da'" class="article-lang-note">
      {{ t('blog.englishOnlyNote') }}
    </p>

    <KnowledgeViewNav current="read" />

    <section v-for="part in partsWithNodes" :key="part.id" class="knowledge-part">
      <h2 :id="part.id">{{ part.title }}</h2>

      <section v-for="node in part.nodes" :id="node.id" :key="node.id" class="node">
        <div class="node-head">
          <h3>{{ node.title }}</h3>
          <a class="node-anchor" :href="`#${node.id}`">
            <span aria-hidden="true">#</span>
            <span class="visually-hidden">
              {{ t('ai.linkToSection', { title: node.title }) }}
            </span>
          </a>
        </div>

        <div class="node-body" v-html="node.body" />

        <div v-if="node.misconceptions.length > 0" class="node-misreadings">
          <p>{{ t('ai.oftenMisread') }}</p>
          <ul>
            <li v-for="belief in node.misconceptions" :key="belief">“{{ belief }}”</li>
          </ul>
        </div>

        <p v-if="node.buildsOn.length > 0 || node.seeAlso.length > 0" class="node-links">
          <span v-if="node.buildsOn.length > 0">
            {{ t('ai.buildsOn') }}
            <template v-for="(link, idx) in node.buildsOn" :key="link.id">
              <a :href="`#${link.id}`">{{ link.title }}</a><template v-if="idx < node.buildsOn.length - 1">, </template>
            </template>
          </span>
          <span v-if="node.seeAlso.length > 0">
            {{ t('ai.seeAlso') }}
            <template v-for="(link, idx) in node.seeAlso" :key="link.id">
              <a :href="`#${link.id}`">{{ link.title }}</a><template v-if="idx < node.seeAlso.length - 1">, </template>
            </template>
          </span>
        </p>
      </section>
    </section>

    <p class="knowledge-coda">
      {{ t('ai.codaPrefix') }}
      <NuxtLink :to="localePath(`${article.url}ask/`)">{{ t('ai.codaAskLink') }}</NuxtLink>
      {{ t('ai.codaMiddle') }}
      <a href="#rag">{{ t('ai.codaRagLink') }}</a>{{ t('ai.codaSuffix') }}
    </p>

    <DiagramLightbox v-if="hasDiagrams" content-key="blog/ai" />
  </article>
</template>
