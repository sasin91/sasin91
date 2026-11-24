<script setup lang="ts">
import type { NuxtError } from '#app'
import { Preferences } from '@ui-sdk/vue/preferences'
import { colorScheme } from '~/utils/theme'

const props = defineProps<{
  error: NuxtError
}>()

const { t } = useI18n()
const localePath = useLocalePath()
const isNotFound = computed(() => props.error?.statusCode === 404)
</script>

<template>
  <Preferences :preferences="[colorScheme]">
    <NuxtLayout>
      <h1>{{ isNotFound ? t('error.notFoundTitle') : t('error.genericTitle') }}</h1>
      <p class="lede">
        {{
          isNotFound
            ? t('error.notFoundLede')
            : (error?.statusMessage || t('error.genericLede'))
        }}
      </p>
      <p>
        <NuxtLink :to="localePath('/')">{{ t('error.goHome') }}</NuxtLink>
        {{ t('error.orBrowse') }}
        <NuxtLink :to="localePath('/blog/')">{{ t('nav.writing') }}</NuxtLink>.
      </p>
    </NuxtLayout>
  </Preferences>
</template>
