import { resolve } from 'node:path'
import vue from '@vitejs/plugin-vue'
import { defineConfig } from 'vitest/config'

export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: {
      '~': resolve(import.meta.dirname, 'app'),
      '@': resolve(import.meta.dirname, 'app'),
      '#shared': resolve(import.meta.dirname, 'shared'),
    },
  },
  test: {
    environment: 'jsdom',
    setupFiles: ['./tests/setup.ts'],
    css: false,
  },
})
