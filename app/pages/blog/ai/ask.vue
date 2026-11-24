<script setup lang="ts">
import { computed } from 'vue'
import { getCv, normalizeLocale } from '#shared/data/cv'
import type { AiArticleOutline } from '#shared/types/content'

const ASK_EXAMPLES = [
  "Why doesn't a larger context window replace RAG?",
  'What is stored in the KV cache?',
  'Is MCP the same thing as RAG?',
  'Does the model learn from my conversation?',
  'top-p',
]

const { t, locale } = useI18n()
const localePath = useLocalePath()
const cv = computed(() => getCv(normalizeLocale(locale.value)))

const { data: article } = await useFetch<AiArticleOutline>('/api/ai-article?outline=1')

if (!article.value) {
  throw createError({ statusCode: 500, statusMessage: 'Could not load AI article', fatal: true })
}

useSiteSeo({
  title: `${t('ai.ask')}: ${article.value.title} — ${cv.value.site.name}`,
  description:
    'Ask a question and get the most relevant sections of the article, found by semantic search that runs entirely in your browser.',
  path: `${article.value.url}ask/`,
  ogType: 'website',
})
</script>

<template>
  <template v-if="article">
    <h1>{{ t('ai.askTitle') }}</h1>
    <p class="lede">
      {{ t('ai.askLedePrefix') }}
      <NuxtLink :to="localePath(article.url)">{{ article.title }}</NuxtLink>
      {{ t('ai.askLedeSuffix') }}
    </p>

    <KnowledgeViewNav current="ask" />

    <AskArticle
      mode="page"
      scope="ai"
      :examples="ASK_EXAMPLES"
      :placeholder="ASK_EXAMPLES[0]"
      :article-url="article.url"
    />
  </template>
</template>
