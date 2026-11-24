import { render, screen, waitFor } from '@testing-library/vue'
import userEvent from '@testing-library/user-event'
import { createAgent } from '@ui-sdk/vue/agent'
import { Preferences } from '@ui-sdk/vue/preferences'
import { beforeEach, describe, expect, test, vi } from 'vitest'
import { defineComponent, h } from 'vue'
import AskArticle from '~/components/AskArticle.vue'
import LocaleSwitcher from '~/components/LocaleSwitcher.vue'
import SearchPalette from '~/components/SearchPalette.vue'
import ThemeSwitch from '~/components/ThemeSwitch.vue'
import { resetSearchCache } from '~/utils/search-store'
import { colorScheme } from '~/utils/theme'

const NuxtLinkStub = defineComponent({
  props: { to: { type: String, required: true } },
  setup(props, { slots }) {
    return () => h('a', { href: props.to }, slots.default?.())
  },
})

const MOCK_INDEX = {
  version: 2,
  model: {
    name: 'potion-base-4M',
    weights_url: '/search/model.bin?v=test',
    vocab_url: '/search/vocab.txt?v=test',
    bytes: 4_100_000,
    dim: 4,
  },
  nodes: [
    {
      kind: 'ai',
      context: 'How modern language models work',
      id: 'kv-cache',
      order: 1,
      url: '/blog/ai/#kv-cache',
      title: 'KV cache',
      summary: 'Keys and values already computed for earlier tokens are reused.',
      concepts: ['kv cache', 'key-value cache'],
      search_terms: ['kv-cache', 'prompt caching'],
      words: ['keys', 'values', 'cache', 'generation', 'tokens'],
      passages: [
        {
          snippet: 'Keys and values already computed for earlier tokens are reused.',
          vector: [1, 0, 0, 0],
        },
      ],
    },
    {
      kind: 'ai',
      context: 'How modern language models work',
      id: 'quantization',
      order: 2,
      url: '/blog/ai/#quantization',
      title: 'Quantization',
      summary: 'Quantization stores weights in fewer bits such as 8 or 4.',
      concepts: ['quantization', '4-bit'],
      search_terms: ['gguf', 'q4'],
      words: ['quantization', 'weights', 'bits', 'precision'],
      passages: [
        {
          snippet: 'Quantization stores weights in fewer bits such as 8 or 4.',
          vector: [0, 1, 0, 0],
        },
      ],
    },
  ],
}

const agent = createAgent()

beforeEach(() => {
  resetSearchCache()
  localStorage.clear()
  colorScheme.set('light', { reason: 'imperative' })
  vi.spyOn(console, 'error').mockImplementation(() => {})
  vi.stubGlobal(
    'fetch',
    vi.fn(async (input: RequestInfo | URL) => {
      const url = String(input)
      if (url.includes('/search/index.json')) {
        return new Response(JSON.stringify(MOCK_INDEX), {
          status: 200,
          headers: { 'content-type': 'application/json' },
        })
      }
      return new Response('Not found', { status: 404 })
    }),
  )
})

describe('ui-sdk interactive components', () => {
  test('ThemeSwitch toggles color scheme with keyboard Space and updates aria-checked', async () => {
    const user = userEvent.setup()
    render({
      components: { Preferences, ThemeSwitch },
      setup: () => ({ prefs: [colorScheme] }),
      template: '<Preferences :preferences="prefs"><ThemeSwitch /></Preferences>',
    })

    const toggle = screen.getByRole('switch', { name: 'Dark theme' })
    expect(toggle.getAttribute('aria-checked')).toBe('false')
    expect(toggle.getAttribute('aria-disabled')).toBeNull()

    toggle.focus()
    await user.keyboard(' ')
    expect(toggle.getAttribute('aria-checked')).toBe('true')
    expect(colorScheme.get().value).toBe('dark')

    await user.keyboard(' ')
    expect(toggle.getAttribute('aria-checked')).toBe('false')
    expect(colorScheme.get().value).toBe('light')
  })

  test('SearchPalette opens, searches content, navigates on selection, and closes on Escape', async () => {
    const user = userEvent.setup()
    const navigate = vi.fn()

    render(SearchPalette, {
      props: { navigate },
      global: { stubs: { NuxtLink: NuxtLinkStub } },
    })

    const trigger = screen.getByRole('button', { name: /Search/ })
    await user.click(trigger)

    const input = await screen.findByRole('combobox', { name: 'Search posts and the AI article' })
    await user.type(input, 'KV cache')

    await waitFor(() => {
      expect(screen.getByText('KV cache')).toBeTruthy()
    })

    await user.click(screen.getByText('KV cache'))
    expect(navigate).toHaveBeenCalledWith('/blog/ai/#kv-cache', false)

    await user.click(trigger)
    await screen.findByRole('combobox', { name: 'Search posts and the AI article' })
    await user.keyboard('{Escape}')
    await waitFor(() => {
      expect(document.activeElement).toBe(trigger)
    })
  })

  test('AskArticle searches, reveals Why matched collapsible, and toggles semantic ranking', async () => {
    const user = userEvent.setup()

    render(AskArticle, {
      props: {
        mode: 'page',
        scope: 'ai',
        examples: ['What is stored in the KV cache?'],
        placeholder: 'Ask a question',
      },
      global: { stubs: { NuxtLink: NuxtLinkStub } },
    })

    await user.click(screen.getByRole('button', { name: 'What is stored in the KV cache?' }))

    await waitFor(() => {
      expect(screen.getByRole('link', { name: 'KV cache' })).toBeTruthy()
    })

    const whyTrigger = screen.getByRole('button', { name: /Why “KV cache” matched/ })
    await user.click(whyTrigger)
    expect(screen.getByText(/word score/)).toBeTruthy()

    const semanticCheckbox = screen.getByRole('checkbox', {
      name: 'Rank by meaning as well as by matching words',
    })
    expect(semanticCheckbox.getAttribute('aria-checked')).toBe('true')
    semanticCheckbox.focus()
    await user.keyboard(' ')
    expect(semanticCheckbox.getAttribute('aria-checked')).toBe('false')
  })

  test('LocaleSwitcher switches between EN and DA with keyboard arrows and clicks', async () => {
    const user = userEvent.setup()
    const onSelectLocale = vi.fn()

    render(LocaleSwitcher, {
      props: { modelLocale: 'en', onSelectLocale },
    })

    const enRadio = screen.getByRole('radio', { name: 'EN' })
    const daRadio = screen.getByRole('radio', { name: 'DA' })
    expect(enRadio.getAttribute('aria-checked')).toBe('true')
    expect(daRadio.getAttribute('aria-checked')).toBe('false')

    enRadio.focus()
    await user.keyboard('{ArrowRight}')
    expect(onSelectLocale).toHaveBeenCalledWith('da')
    expect(daRadio.getAttribute('aria-checked')).toBe('true')

    await user.click(enRadio)
    expect(onSelectLocale).toHaveBeenCalledWith('en')
    expect(enRadio.getAttribute('aria-checked')).toBe('true')
  })

  test('an agent can discover every named control and operate the interface', async () => {
    const onSelectLocale = vi.fn()
    render(
      {
        components: { Preferences, ThemeSwitch, LocaleSwitcher, SearchPalette, AskArticle },
        setup: () => ({
          prefs: [colorScheme],
          examples: ['What is stored in the KV cache?'],
          onSelectLocale,
        }),
        template: `
          <Preferences :preferences="prefs">
            <SearchPalette />
            <LocaleSwitcher model-locale="en" :on-select-locale="onSelectLocale" />
            <ThemeSwitch />
            <AskArticle
              mode="page"
              scope="ai"
              :examples="examples"
              placeholder="Ask a question"
            />
          </Preferences>
        `,
      },
      {
        global: { stubs: { NuxtLink: NuxtLinkStub } },
      },
    )

    const components = agent
      .snapshot()
      .filter((entry) => !entry.offPage && entry.component !== 'more')

    for (const entry of components) {
      expect(entry.label, `${entry.component} ${entry.ref} has no name`).toBeTruthy()
    }

    const labels = components.map((entry) => entry.label)
    for (const expected of [
      'Language',
      'Dark theme',
      'Your question',
      'Ask',
      'Rank by meaning as well as by matching words',
      'What is stored in the KV cache?',
    ]) {
      expect(labels, `${expected} is missing from agent snapshot`).toContain(expected)
    }

    const result = await agent.act({
      steps: [
        { ref: 'Language', action: 'setValue', args: ['DA'] },
        { ref: 'Dark theme', action: 'setChecked', args: [true] },
        { ref: 'Your question', action: 'setValue', args: ['quantization'] },
        { ref: 'Ask', action: 'press' },
      ],
    })

    expect(result.ok, result.error?.message).toBe(true)
    expect(onSelectLocale).toHaveBeenCalledWith('da')
    expect(colorScheme.get().value).toBe('dark')
    await waitFor(() => {
      expect(screen.getByRole('link', { name: 'Quantization' })).toBeTruthy()
    })
  })
})
