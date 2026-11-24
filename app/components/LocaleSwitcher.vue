<script setup lang="ts">
import {
  SegmentedControl,
  SegmentedControlIndicator,
  SegmentedControlItem,
} from '@ui-sdk/vue/segmented-control'
import { computed, ref, watch } from 'vue'
import type { LocaleCode } from '#shared/data/cv'

const props = withDefaults(
  defineProps<{
    modelLocale?: LocaleCode
    onSelectLocale?: (next: LocaleCode) => void
  }>(),
  {
    modelLocale: undefined,
    onSelectLocale: undefined,
  },
)

const i18n = typeof useI18n === 'function' ? useI18n() : null
const switchLocalePath = typeof useSwitchLocalePath === 'function' ? useSwitchLocalePath() : null

const selected = ref<string | null>(
  props.modelLocale ?? (i18n?.locale.value as LocaleCode | undefined) ?? 'en',
)

watch(
  () => props.modelLocale ?? (i18n?.locale.value as LocaleCode | undefined),
  (next) => {
    if (next && next !== selected.value) {
      selected.value = next
    }
  },
)

function onValueChange(next: string | null) {
  if (next !== 'en' && next !== 'da') return
  selected.value = next
  if (props.onSelectLocale) {
    props.onSelectLocale(next)
    return
  }
  if (switchLocalePath && typeof navigateTo === 'function') {
    const target = switchLocalePath(next)
    if (target) {
      void navigateTo(target)
    }
  }
}

const label = computed(() => (i18n ? i18n.t('nav.language') : 'Language'))
</script>

<template>
  <SegmentedControl
    :value="selected"
    class="locale-switch"
    :aria-label="label"
    @value-change="onValueChange"
  >
    <SegmentedControlIndicator class="locale-switch-indicator" />
    <SegmentedControlItem value="en" class="locale-switch-item">EN</SegmentedControlItem>
    <SegmentedControlItem value="da" class="locale-switch-item">DA</SegmentedControlItem>
  </SegmentedControl>
</template>
