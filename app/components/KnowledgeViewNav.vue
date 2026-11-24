<script setup lang="ts">
defineProps<{
  current: 'read' | 'timeline' | 'ask'
}>()

const { t } = useI18n()
const localePath = useLocalePath()

const views = computed(() => [
  { key: 'read', label: t('ai.read'), to: localePath('/blog/ai/') },
  { key: 'timeline', label: t('ai.timeline'), to: localePath('/blog/ai/timeline/') },
  { key: 'ask', label: t('ai.ask'), to: localePath('/blog/ai/ask/') },
] as const)
</script>

<template>
  <nav class="view-nav" :aria-label="t('ai.viewsLabel')">
    <NuxtLink
      v-for="view in views"
      :key="view.key"
      :to="view.to"
      :aria-current="current === view.key ? 'page' : undefined"
    >
      {{ view.label }}
    </NuxtLink>
  </nav>
</template>
