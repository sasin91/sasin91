<script setup lang="ts">
import { computed } from 'vue'
import { getCv, normalizeLocale, type LocaleCode } from '#shared/data/cv'
import type { AiArticleOutline } from '#shared/types/content'

const ASK_EXAMPLES: Record<LocaleCode, string[]> = {
  en: [
    "Why doesn't a larger context window replace RAG?",
    'What is stored in the KV cache?',
    'Is MCP the same thing as RAG?',
    'Does the model learn from my conversation?',
    'top-p',
  ],
  da: [
    'Hvorfor erstatter et større kontekstvindue ikke RAG?',
    'Hvad gemmes i KV-cachen?',
    'Er MCP det samme som RAG?',
    'Lærer modellen af min samtale?',
    'top-p',
  ],
}

const { t, locale } = useI18n()
const localePath = useLocalePath()
const currentLocale = computed(() => normalizeLocale(locale.value))
const cv = computed(() => getCv(currentLocale.value))
const examples = computed(() => ASK_EXAMPLES[currentLocale.value])

const { data: article } = await useFetch<AiArticleOutline>('/api/ai-article', {
  key: () => `ai-article-ask-outline-${currentLocale.value}`,
  query: { outline: '1', lang: currentLocale },
})

if (!article.value) {
  throw createError({ statusCode: 500, statusMessage: 'Could not load AI article', fatal: true })
}

useSiteSeo({
  title: `${t('ai.ask')}: ${article.value.title} — ${cv.value.site.name}`,
  description:
    currentLocale.value === 'da'
      ? 'Stil et spørgsmål og få de mest relevante afsnit fra artiklen, fundet med semantisk søgning direkte i din browser.'
      : 'Ask a question and get the most relevant sections of the article, found by semantic search that runs entirely in your browser.',
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
      :examples="examples"
      :placeholder="examples[0]"
      :article-url="article.url"
    />
  </template>
</template>
