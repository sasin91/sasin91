<script setup lang="ts">
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandGroupLabel,
  CommandInput,
  CommandItem,
  CommandList,
} from '@ui-sdk/vue/command'
import {
  Dialog,
  DialogBackdrop,
  DialogContent,
  DialogShortcut,
  DialogTitle,
  DialogTrigger,
} from '@ui-sdk/vue/dialog'
import { useShortcut } from '@ui-sdk/vue/shortcuts'
import { computed, ref, watch } from 'vue'
import { getCv, normalizeLocale } from '#shared/data/cv'
import { rank, resultUrl, type PreparedSearchIndex, type StaticModel } from '#shared/retrieval'
import { getLoadedModel, loadEmbeddingModel, loadSiteIndex } from '~/utils/search-store'
import { toggleTheme } from '~/utils/theme'

interface SiteCommand {
  id: string
  title: string
  hint?: string
  url?: string
  external?: boolean
  run?: () => void
}

interface SearchHit {
  id: string
  title: string
  context: string
  snippet: string
  url: string
}

const props = withDefaults(
  defineProps<{
    navigate?: (url: string, external?: boolean) => void
  }>(),
  {
    navigate: undefined,
  },
)

const i18n = typeof useI18n === 'function' ? useI18n() : null
const localePath = typeof useLocalePath === 'function' ? useLocalePath() : (p: string) => p
const switchLocalePath = typeof useSwitchLocalePath === 'function' ? useSwitchLocalePath() : null

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

const RESULT_LIMIT = 8
const RELATIVE_CUTOFF = 0.65

const currentLocale = computed(() => normalizeLocale(i18n?.locale.value))
const currentCv = computed(() => getCv(currentLocale.value))

const commands = computed<SiteCommand[]>(() => {
  const otherLocale = currentLocale.value === 'da' ? 'en' : 'da'
  const list: SiteCommand[] = [
    { id: 'cmd:home', title: tr('palette.cmd.home', 'Home'), url: localePath('/') },
    {
      id: 'cmd:blog',
      title: tr('palette.cmd.writing', 'Writing'),
      hint: tr('palette.cmd.writingHint', 'Every post'),
      url: localePath('/blog/'),
    },
    {
      id: 'cmd:ai',
      title: tr('palette.cmd.ai', 'How modern language models work'),
      hint: tr('palette.cmd.aiHint', 'The AI article'),
      url: localePath('/blog/ai/'),
    },
    {
      id: 'cmd:timeline',
      title: tr('palette.cmd.timeline', 'Timeline'),
      hint: tr('palette.cmd.timelineHint', 'The AI article, one line per idea'),
      url: localePath('/blog/ai/timeline/'),
    },
    {
      id: 'cmd:ask',
      title: tr('palette.cmd.ask', 'Ask this article'),
      hint: tr('palette.cmd.askHint', 'Retrieval over the AI article'),
      url: localePath('/blog/ai/ask/'),
    },
    { id: 'cmd:about', title: tr('palette.cmd.about', 'About'), url: localePath('/about/') },
    { id: 'cmd:cv', title: tr('palette.cmd.cv', 'CV'), url: localePath('/cv/') },
    {
      id: 'cmd:pdf',
      title: tr('palette.cmd.pdf', 'Download CV'),
      hint: tr('palette.cmd.pdfHint', 'PDF'),
      url: currentCv.value.pdfPath,
      external: true,
    },
    {
      id: 'cmd:theme',
      title: tr('palette.cmd.theme', 'Toggle dark theme'),
      hint: tr('palette.cmd.themeHint', 'Switch between light and dark'),
      run: toggleTheme,
    },
  ]

  if (switchLocalePath) {
    list.push({
      id: 'cmd:locale',
      title: tr('palette.cmd.switchLanguage', 'Skift til dansk'),
      hint: tr('palette.cmd.switchLanguageHint', 'Switch language to Danish'),
      url: switchLocalePath(otherLocale),
    })
  }

  return list
})

const open = ref(false)
const query = ref('')
const index = ref<PreparedSearchIndex | null>(null)
const model = ref<StaticModel | null>(getLoadedModel())
const failed = ref(false)
const modelLoading = ref(false)
let loadingPromise: Promise<void> | null = null

function saveData(): boolean {
  if (typeof navigator === 'undefined') return false
  const connection = (navigator as Navigator & { connection?: { saveData?: boolean } }).connection
  return Boolean(connection?.saveData)
}

let loadedLocale: string | null = null

function ensureLoaded() {
  const loc = currentLocale.value
  if (loadingPromise && loadedLocale === loc) return
  loadedLocale = loc
  loadingPromise = (async () => {
    try {
      index.value = await loadSiteIndex(loc)
      failed.value = false
    } catch (error) {
      console.error('Failed to load search index:', error)
      failed.value = true
      loadingPromise = null
      return
    }
    if (saveData() || model.value) return
    modelLoading.value = true
    try {
      model.value = await loadEmbeddingModel(undefined, loc)
    } catch (error) {
      // Word matching remains available if the embedding model cannot be loaded.
      console.error('Failed to load embedding model:', error)
    } finally {
      modelLoading.value = false
    }
  })()
}

watch([open, currentLocale], ([isOpen]) => {
  if (isOpen) {
    ensureLoaded()
  }
})

useShortcut('/', () => {
  open.value = true
}, { description: 'Search the site' })

useShortcut('ctrl+space', () => {
  open.value = !open.value
}, { description: 'Toggle search palette' })

const trimmedQuery = computed(() => query.value.trim())

const matchingCommands = computed<SiteCommand[]>(() => {
  const q = trimmedQuery.value.toLowerCase()
  if (!q) return commands.value
  return commands.value
    .filter((c) => `${c.title} ${c.hint ?? ''}`.toLowerCase().includes(q))
    .slice(0, 3)
})

const contentResults = computed<SearchHit[]>(() => {
  const q = trimmedQuery.value
  if (!q || !index.value) return []
  const vector = model.value ? model.value.encode(q).vector : null
  const ranked = rank(index.value, q, vector, RESULT_LIMIT)
  const floor = (ranked[0]?.score ?? 0) * RELATIVE_CUTOFF
  return ranked
    .filter((r) => r.score >= floor)
    .map((result) => {
      const heading = result.passage?.heading
      const context =
        result.node.kind === 'post' && heading
          ? `${result.node.context} › ${heading}`
          : result.node.context
      const url = localizedUrl(resultUrl(result))
      return {
        id: `hit:${result.node.kind}:${result.node.id}:${result.passage?.anchor ?? ''}`,
        title: result.node.title,
        context,
        snippet: result.passage?.snippet ?? result.node.summary,
        url,
      }
    })
})

const hasAnyItems = computed(() => contentResults.value.length > 0 || matchingCommands.value.length > 0)

const statusMessage = computed(() => {
  if (failed.value) {
    return tr('palette.statusFailed', 'The search index could not be loaded. Pages are still listed.')
  }
  if (!trimmedQuery.value) {
    return tr(
      'palette.statusIdle',
      'Type to search posts and the AI article. Everything runs on this device.',
    )
  }
  if (!index.value) {
    return tr('palette.statusLoadingIndex', 'Loading the search index…')
  }
  const count = contentResults.value.length
  const found =
    count === 0
      ? tr('palette.statusNone', 'No results')
      : count === 1
        ? tr('palette.statusCountOne', '1 result')
        : tr('palette.statusCountMany', '{count} results', { count })
  if (model.value) {
    return tr('palette.statusHybrid', '{found}, ranked by meaning and matching words.', { found })
  }
  if (saveData()) {
    return tr(
      'palette.statusSaveData',
      '{found}, by matching words (data saver is on, so the meaning model was not loaded).',
      { found },
    )
  }
  if (modelLoading.value) {
    return tr('palette.statusModelLoading', '{found} by matching words; the meaning model is loading.', {
      found,
    })
  }
  return tr(
    'palette.statusLexicalOnly',
    '{found}, by matching words only (the meaning model could not be loaded).',
    { found },
  )
})

function goTo(url: string, external = false) {
  if (props.navigate) {
    props.navigate(url, external)
    return
  }
  if (external) {
    window.location.assign(url)
    return
  }
  if (typeof navigateTo === 'function') {
    void navigateTo(url)
  } else {
    window.location.assign(url)
  }
}

function onAction(value: string) {
  const cmd = matchingCommands.value.find((c) => c.id === value)
  if (cmd) {
    open.value = false
    query.value = ''
    if (cmd.run) {
      cmd.run()
    } else if (cmd.url) {
      goTo(cmd.url, cmd.external)
    }
    return
  }
  const hit = contentResults.value.find((r) => r.id === value)
  if (hit) {
    open.value = false
    query.value = ''
    goTo(hit.url)
  }
}
</script>

<template>
  <Dialog v-model:open="open">
    <DialogTrigger class="site-search" shortcut="mod+k">
      <svg viewBox="0 0 20 20" fill="currentColor" aria-hidden="true" focusable="false">
        <path
          fill-rule="evenodd"
          clip-rule="evenodd"
          d="M9 3.5a5.5 5.5 0 1 0 0 11 5.5 5.5 0 0 0 0-11ZM2 9a7 7 0 1 1 12.452 4.391l3.328 3.329a.75.75 0 1 1-1.06 1.06l-3.329-3.328A7 7 0 0 1 2 9Z"
        />
      </svg>
      <span class="site-search-label">{{ tr('nav.search', 'Search') }}</span>
      <DialogShortcut class="site-search-key" />
    </DialogTrigger>
    <DialogBackdrop class="palette-backdrop" />
    <DialogContent class="palette">
      <DialogTitle class="visually-hidden">{{ tr('palette.title', 'Search the site') }}</DialogTitle>
      <Command
        v-model:query="query"
        class="palette-command"
        :filter="false"
        @action="onAction"
      >
        <div class="palette-search">
          <svg viewBox="0 0 20 20" fill="currentColor" aria-hidden="true" focusable="false">
            <path
              fill-rule="evenodd"
              clip-rule="evenodd"
              d="M9 3.5a5.5 5.5 0 1 0 0 11 5.5 5.5 0 0 0 0-11ZM2 9a7 7 0 1 1 12.452 4.391l3.328 3.329a.75.75 0 1 1-1.06 1.06l-3.329-3.328A7 7 0 0 1 2 9Z"
            />
          </svg>
          <CommandInput
            class="palette-input"
            :aria-label="tr('palette.inputLabel', 'Search posts and the AI article')"
            :placeholder="tr('palette.placeholder', 'Search posts and the AI article…')"
            autocomplete="off"
            spellcheck="false"
          />
        </div>
        <CommandEmpty v-if="!hasAnyItems" class="palette-empty">
          {{
            trimmedQuery
              ? tr('palette.nothingFound', 'Nothing found for “{query}”.', { query: trimmedQuery })
              : ''
          }}
        </CommandEmpty>
        <CommandList class="palette-list">
          <CommandGroup v-if="contentResults.length > 0" class="palette-group">
            <CommandGroupLabel class="palette-group-label">
              {{ tr('palette.groupResults', 'Results') }}
            </CommandGroupLabel>
            <CommandItem
              v-for="entry in contentResults"
              :key="entry.id"
              :value="entry.id"
              class="palette-option"
            >
              <span class="palette-title">{{ entry.title }}</span>
              <span v-if="entry.context" class="palette-context">{{ entry.context }}</span>
              <span v-if="entry.snippet" class="palette-snippet">{{ entry.snippet }}</span>
            </CommandItem>
          </CommandGroup>
          <CommandGroup v-if="matchingCommands.length > 0" class="palette-group">
            <CommandGroupLabel class="palette-group-label">
              {{
                trimmedQuery
                  ? tr('palette.groupPages', 'Pages')
                  : tr('palette.groupGoTo', 'Go to')
              }}
            </CommandGroupLabel>
            <CommandItem
              v-for="entry in matchingCommands"
              :key="entry.id"
              :value="entry.id"
              class="palette-option"
            >
              <span class="palette-title">{{ entry.title }}</span>
              <span v-if="entry.hint" class="palette-context">{{ entry.hint }}</span>
            </CommandItem>
          </CommandGroup>
        </CommandList>
        <div class="palette-foot">
          <span role="status" class="palette-status">{{ statusMessage }}</span>
          <span class="palette-keys" aria-hidden="true">
            <kbd>↑</kbd><kbd>↓</kbd> {{ tr('palette.keyMove', 'move') }}
            <kbd>↵</kbd> {{ tr('palette.keyOpen', 'open') }}
            <kbd>esc</kbd> {{ tr('palette.keyClose', 'close') }}
          </span>
        </div>
      </Command>
    </DialogContent>
  </Dialog>
</template>
