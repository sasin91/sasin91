<script setup lang="ts">
import { Button } from '@ui-sdk/vue/button'
import { Checkbox, CheckboxIndicator } from '@ui-sdk/vue/checkbox'
import {
  Collapsible,
  CollapsiblePanel,
  CollapsibleTrigger,
} from '@ui-sdk/vue/collapsible'
import { Field, FieldLabel } from '@ui-sdk/vue/field'
import { Input, InputClear, InputGroup } from '@ui-sdk/vue/input'
import { computed, ref, watch } from 'vue'
import {
  prepareIndex,
  rank,
  scopedIndex,
  type EncodedQuery,
  type LexicalHit,
  type PreparedSearchIndex,
  type RankedResult,
  type StaticModel,
} from '#shared/retrieval'
import { loadEmbeddingModel, loadRawIndex } from '~/utils/search-store'

const props = withDefaults(
  defineProps<{
    scope: string
    mode?: 'inline' | 'page'
    examples?: string[]
    placeholder?: string
    articleUrl?: string
  }>(),
  {
    mode: 'inline',
    examples: () => [],
    placeholder: undefined,
    articleUrl: '/blog/ai/',
  },
)

const i18n = typeof useI18n === 'function' ? useI18n() : null
const localePath = typeof useLocalePath === 'function' ? useLocalePath() : (p: string) => p

function tr(key: string, fallback: string, params?: Record<string, string | number>): string {
  if (i18n) {
    return params ? i18n.t(key, params) : i18n.t(key)
  }
  if (!params) return fallback
  return Object.entries(params).reduce(
    (acc, [k, v]) => acc.replaceAll(`{${k}}`, String(v)),
    fallback,
  )
}

function localizedUrl(url: string): string {
  const [pathPart, hashPart] = url.split('#')
  const base = localePath(pathPart || '/')
  return hashPart ? `${base}#${hashPart}` : base
}

const resolvedPlaceholder = computed(
  () => props.placeholder ?? tr('ask.placeholderInline', 'Ask this article a question'),
)

const FIELD_NAMES: Record<LexicalHit['field'], string> = {
  title: 'title',
  concept: 'concept',
  term: 'search term',
  keyword: 'keyword',
  text: 'text',
}

const query = ref('')
const useSemantic = ref(true)
const statusText = ref('')
const results = ref<RankedResult[]>([])
const hasSearched = ref(false)
const lastQuery = ref('')
const lastUsedSemantic = ref(false)
const lastEncoded = ref<EncodedQuery | null>(null)
const lastVector = ref<Float32Array | null>(null)

const scopedPrepared = ref<PreparedSearchIndex | null>(null)
const loadedModel = ref<StaticModel | null>(null)
let loadPromise: Promise<{ index: PreparedSearchIndex | null; model: StaticModel | null }> | null = null

const modelDim = computed(() => scopedPrepared.value?.model.dim ?? 128)
const modelName = computed(() => scopedPrepared.value?.model.name ?? 'potion-base-4M')

async function ensureLoaded(): Promise<{ index: PreparedSearchIndex | null; model: StaticModel | null }> {
  if (loadPromise) return loadPromise
  loadPromise = (async () => {
    let raw
    try {
      statusText.value = 'Loading the search index…'
      raw = await loadRawIndex()
      scopedPrepared.value = prepareIndex(scopedIndex(raw, props.scope))
    } catch (error) {
      console.error('Failed to load search index:', error)
      statusText.value = 'The search index could not be loaded. Please try again later.'
      loadPromise = null
      return { index: null, model: null }
    }

    try {
      const mb = (raw.model.bytes / 1e6).toFixed(1)
      statusText.value = `Loading the embedding model (${mb} MB, downloaded once)…`
      const model = await loadEmbeddingModel(raw)
      loadedModel.value = model
      statusText.value = 'Ready. Searching happens on this device.'
      return { index: scopedPrepared.value, model }
    } catch (error) {
      console.error('Failed to load embedding model:', error)
      statusText.value = 'The embedding model could not be loaded, so results use matching words only.'
      return { index: scopedPrepared.value, model: null }
    }
  })()
  return loadPromise
}

watch(
  () => props.scope,
  () => {
    loadPromise = null
    scopedPrepared.value = null
    results.value = []
    hasSearched.value = false
    statusText.value = ''
    query.value = ''
    lastQuery.value = ''
  },
)

async function runAsk(rawQuery: string) {
  const q = rawQuery.trim()
  if (!q) {
    statusText.value = 'Type a question first.'
    return
  }
  lastQuery.value = q
  const { index, model } = await ensureLoaded()
  if (!index) return

  const encoded = model ? model.encode(q) : null
  const wantsMeaning = props.mode === 'inline' || useSemantic.value
  const vector = wantsMeaning && encoded ? encoded.vector : null
  const ranked = rank(index, q, vector)

  lastEncoded.value = encoded
  lastVector.value = vector
  lastUsedSemantic.value = Boolean(vector)
  results.value = ranked
  hasSearched.value = true

  const how = vector
    ? 'by meaning and matching words'
    : model
      ? 'by matching words only'
      : 'by matching words only, because the embedding model could not be loaded'
  statusText.value = ranked.length
    ? `${ranked.length} sections found ${how}.`
    : 'No section matched those words. Try describing the idea differently.'
}

function onSubmit() {
  return runAsk(query.value)
}

function runExample(example: string) {
  query.value = example
  return runAsk(example)
}

function onSemanticChange(next: boolean | 'indeterminate') {
  useSemantic.value = next === true
  if (lastQuery.value) {
    void runAsk(lastQuery.value)
  }
}

function ordinal(n: number): string {
  const suffix =
    n % 100 >= 11 && n % 100 <= 13 ? 'th' : ({ 1: 'st', 2: 'nd', 3: 'rd' }[n % 10] ?? 'th')
  return `${n}${suffix}`
}

function describeMatches(matched: LexicalHit[]): string {
  if (!matched.length) return 'No exact words in common; this is a match by meaning only.'
  return matched.map((m) => `“${m.text}” (${FIELD_NAMES[m.field]})`).join(', ')
}

const vectorPreview = computed(() => {
  const vector = lastVector.value
  if (!vector) return null
  const shown = Array.from(vector.slice(0, 32))
  const peak = Math.max(...shown.map(Math.abs)) || 1
  return {
    length: vector.length,
    preview: shown.slice(0, 3).map((v) => v.toFixed(3)).join(', '),
    bars: shown.map((v, idx) => ({
      id: idx,
      negative: v < 0,
      height: `${Math.max(4, (Math.abs(v) / peak) * 100)}%`,
    })),
  }
})

const pipelineCompareText = computed(() => {
  if (!hasSearched.value || !scopedPrepared.value) return ''
  const passages = scopedPrepared.value.nodes.reduce((sum, node) => sum + node.passages.length, 0)
  if (lastVector.value && results.value.length) {
    const best = [...results.value].sort((a, b) => (a.semanticRank ?? 999) - (b.semanticRank ?? 999))[0]!
    return `Compared with ${passages} passage vectors from ${scopedPrepared.value.nodes.length} sections. Closest in meaning among the results: “${best.node.title}”.`
  }
  return 'Skipped: ranking by matching words only.'
})

const pipelineWordsText = computed(() => {
  if (!hasSearched.value) return ''
  const matched = new Set(results.value.flatMap((r) => r.matched.map((m) => m.text)))
  return matched.size
    ? `Matched: ${[...matched].map((t) => `“${t}”`).join(', ')}.`
    : 'No exact words or phrases matched.'
})
</script>

<template>
  <div :class="['ask-container', mode === 'inline' ? 'ask-inline' : '']">
    <form class="ask" role="search" @submit.prevent="onSubmit">
      <Field name="q" class="ask-field">
        <FieldLabel :class="mode === 'inline' ? 'visually-hidden' : 'ask-label'">
          {{
            mode === 'inline'
              ? tr('ask.labelInline', 'Ask this article')
              : tr('ask.labelPage', 'Your question')
          }}
        </FieldLabel>
        <div class="ask-row">
          <InputGroup class="ask-input-group">
            <Input
              v-model="query"
              class="ask-input"
              type="search"
              autocomplete="off"
              spellcheck="false"
              :placeholder="resolvedPlaceholder"
              @focus.once="ensureLoaded"
            />
            <InputClear class="ask-clear" :aria-label="tr('ask.clearQuestion', 'Clear question')">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" aria-hidden="true">
                <path stroke-linecap="round" stroke-linejoin="round" d="M6 18 18 6M6 6l12 12" />
              </svg>
            </InputClear>
          </InputGroup>
          <Button type="button" class="ask-submit" @click="onSubmit">
            {{ tr('ask.submit', 'Ask') }}
          </Button>
        </div>
      </Field>

      <Field v-if="mode === 'page'" class="ask-option-field">
        <Checkbox :checked="useSemantic" @checked-change="onSemanticChange">
          <CheckboxIndicator />
        </Checkbox>
        <FieldLabel class="ask-option-label">
          {{ tr('ask.semanticLabel', 'Rank by meaning as well as by matching words') }}
        </FieldLabel>
      </Field>

      <p class="ask-status" role="status" aria-live="polite">{{ statusText }}</p>

      <div v-if="examples.length > 0" class="ask-examples">
        <p>{{ mode === 'inline' ? tr('ask.tryInline', 'Try:') : tr('ask.tryPage', 'Or try:') }}</p>
        <ul>
          <li v-for="example in examples" :key="example">
            <Button type="button" class="ask-example-btn" @click="runExample(example)">
              {{ example }}
            </Button>
          </li>
        </ul>
      </div>

      <section v-if="hasSearched" class="ask-results" aria-labelledby="ask-results-title">
        <h2 id="ask-results-title" :class="mode === 'inline' ? 'visually-hidden' : ''">
          {{ tr('ask.relevantSections', 'Relevant sections') }}
        </h2>
        <ol v-if="results.length > 0">
          <li v-for="result in results" :key="result.node.id">
            <h3>
              <NuxtLink :to="localizedUrl(result.node.url)">{{ result.node.title }}</NuxtLink>
            </h3>
            <p>{{ result.node.summary }}</p>
            <Collapsible>
              <CollapsibleTrigger class="ask-why-trigger">
                <span>
                  {{ tr('ask.whyMatched', 'Why “{title}” matched', { title: result.node.title }) }}
                </span>
                <svg class="ask-why-chevron" viewBox="0 0 20 20" fill="currentColor" aria-hidden="true">
                  <path
                    fill-rule="evenodd"
                    clip-rule="evenodd"
                    d="M5.22 8.22a.75.75 0 0 1 1.06 0L10 11.94l3.72-3.72a.75.75 0 1 1 1.06 1.06l-4.25 4.25a.75.75 0 0 1-1.06 0L5.22 9.28a.75.75 0 0 1 0-1.06Z"
                  />
                </svg>
              </CollapsibleTrigger>
              <CollapsiblePanel class="ask-why-panel">
                <dl>
                  <template v-if="lastUsedSemantic && result.semanticRank !== undefined && result.passage">
                    <dt>{{ tr('ask.meaning', 'Meaning') }}</dt>
                    <dd>
                      {{ ordinal(result.semanticRank) }} closest of {{ scopedPrepared?.nodes.length ?? 0 }} sections.
                      Closest passage: <q>{{ result.passage.snippet }}</q>
                    </dd>
                  </template>
                  <dt>{{ tr('ask.words', 'Words') }}</dt>
                  <dd>{{ describeMatches(result.matched) }}</dd>
                  <dt>{{ tr('ask.numbers', 'Numbers') }}</dt>
                  <dd>
                    <template v-if="lastUsedSemantic && result.semantic !== null">
                      cosine similarity {{ result.semantic.toFixed(2) }},
                    </template>
                    word score {{ result.lexical.toFixed(2) }}, combined {{ result.score.toFixed(2) }}.
                    These only order the results; they are not a confidence.
                  </dd>
                </dl>
              </CollapsiblePanel>
            </Collapsible>
          </li>
        </ol>
        <p v-if="mode === 'inline'" class="ask-credit">
          {{
            tr(
              'ask.inlineCredit',
              'Retrieval over this article, run on your device: it finds sections, it does not write answers.',
            )
          }}
          <NuxtLink :to="localizedUrl('/blog/ai/ask/#ask-how-title')">
            {{ tr('ask.howItWorks', 'How it works') }}
          </NuxtLink>.
        </p>
      </section>
    </form>

    <section v-if="mode === 'page'" class="ask-how" aria-labelledby="ask-how-title">
      <h2 id="ask-how-title">{{ tr('ask.howTitle', 'How this works') }}</h2>
      <p>
        This page is a small retrieval system of the kind described in
        <NuxtLink :to="localizedUrl(`${articleUrl}#rag`)">Retrieval-augmented generation</NuxtLink>.
        Every step happens in your browser; your question is never sent anywhere.
      </p>
      <ol class="pipeline">
        <li>
          <strong>Tokenize.</strong> Your question is split into the WordPiece tokens of the model's
          vocabulary (<NuxtLink :to="localizedUrl(`${articleUrl}#tokenization`)">tokenization</NuxtLink>).
          <output v-if="hasSearched" class="pipeline-output">
            <template v-if="lastEncoded">
              {{ lastEncoded.tokens.length }} tokens{{
                lastEncoded.unknown ? `, ${lastEncoded.unknown} not in the vocabulary and skipped` : ''
              }}:
              <span class="pipeline-tokens">
                <span
                  v-for="(token, idx) in lastEncoded.tokens"
                  :key="`${idx}-${token}`"
                  :class="['pipeline-token', token === '[UNK]' ? 'is-unknown' : '']"
                >
                  {{ token }}
                </span>
              </span>
            </template>
            <template v-else>Skipped: the model is not loaded.</template>
          </output>
        </li>
        <li>
          <strong>Embed.</strong> Each token selects one row of a static embedding table, and the rows
          are averaged into a single vector of {{ modelDim }} numbers
          (<NuxtLink :to="localizedUrl(`${articleUrl}#embeddings`)">embeddings</NuxtLink>).
          <output v-if="hasSearched" class="pipeline-output">
            <template v-if="vectorPreview">
              <span class="pipeline-bars" aria-hidden="true">
                <span
                  v-for="bar in vectorPreview.bars"
                  :key="bar.id"
                  :class="{ 'is-negative': bar.negative }"
                  :style="{ height: bar.height }"
                />
              </span>
              {{ vectorPreview.length }} numbers, starting {{ vectorPreview.preview }}, …
            </template>
            <template v-else-if="!loadedModel">Skipped: the model is not loaded.</template>
            <template v-else-if="!useSemantic">Skipped: ranking by meaning is switched off.</template>
            <template v-else>Skipped: none of the question's tokens are in the vocabulary.</template>
          </output>
        </li>
        <li>
          <strong>Compare.</strong> That vector is compared with vectors computed for every passage of
          the article ahead of time, by cosine similarity.
          <output v-if="pipelineCompareText" class="pipeline-output">{{ pipelineCompareText }}</output>
        </li>
        <li>
          <strong>Match words.</strong> Exact words and phrases from your question are matched against
          each section's title, concepts and text, so a precise term like <q>KV cache</q> is never
          outranked by something only vaguely similar.
          <output v-if="pipelineWordsText" class="pipeline-output">{{ pipelineWordsText }}</output>
        </li>
        <li>
          <strong>Rank.</strong> The two signals are combined, and the best sections are listed above,
          each linking to its place in the article.
        </li>
      </ol>
      <p class="ask-credit">
        Model: {{ modelName }} by Minish Lab (MIT licence), a static embedding table of {{ modelDim }}
        dimensions, quantised to 8 bits. It is served from this site and downloaded once, when you first
        use the form.
      </p>
    </section>
  </div>
</template>
