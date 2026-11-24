<script setup lang="ts">
import { Preferences } from '@ui-sdk/vue/preferences'
import { colorScheme } from '~/utils/theme'

const localeHead = useLocaleHead({ seo: true })

useHead(() => ({
  htmlAttrs: localeHead.value.htmlAttrs,
  link: localeHead.value.link,
  meta: localeHead.value.meta,
}))
</script>

<template>
  <Preferences :preferences="[colorScheme]">
    <NuxtLayout>
      <NuxtPage />
    </NuxtLayout>
  </Preferences>
</template>
