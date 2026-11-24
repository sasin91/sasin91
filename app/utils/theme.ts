import { createColorScheme, createLocalStorage } from '@ui-sdk/core'

export const colorScheme = createColorScheme({
  global: true,
  storage: createLocalStorage('theme'),
})

export function setThemeWithTransition(next: 'light' | 'dark') {
  const apply = () => {
    colorScheme.set(next, { reason: 'press' })
  }

  if (
    typeof document !== 'undefined' &&
    'startViewTransition' in document &&
    typeof document.startViewTransition === 'function' &&
    !window.matchMedia('(prefers-reduced-motion: reduce)').matches
  ) {
    document.startViewTransition(apply)
  } else {
    apply()
  }
}

export function toggleTheme() {
  const current = colorScheme.get().resolved
  setThemeWithTransition(current === 'dark' ? 'light' : 'dark')
}
