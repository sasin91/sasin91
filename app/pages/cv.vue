<script setup lang="ts">
import { computed } from 'vue'
import { formatCvMonth, getCv, normalizeLocale } from '#shared/data/cv'

const { t, locale } = useI18n()
const currentLocale = computed(() => normalizeLocale(locale.value))
const cv = computed(() => getCv(currentLocale.value))

useSiteSeo({
  title: `${t('cv.title')} — ${cv.value.site.name}`,
  description: cv.value.intro[0] ?? cv.value.site.title,
  path: '/cv/',
  ogType: 'website',
})

function splitUpgradeFlourish(
  text: string,
): { before: string; mid: string; after: string } | null {
  for (const marker of [
    { token: 'version 6 to 7', mid: ' to ' },
    { token: 'version 6 til 7', mid: ' til ' },
  ]) {
    const idx = text.indexOf(marker.token)
    if (idx !== -1) {
      return {
        before: text.slice(0, idx),
        mid: marker.mid,
        after: text.slice(idx + marker.token.length),
      }
    }
  }
  return null
}
</script>

<template>
  <article class="cv">
    <p class="cv-download">
      <a :href="cv.pdfPath">{{ t('cv.downloadPdf') }}</a>
    </p>

    <header class="cv-header">
      <h1>{{ cv.site.name }}</h1>
      <p class="lede">{{ cv.site.title }}</p>
      <p class="cv-contact sep-list">
        <span>{{ cv.contact.town }}, {{ cv.contact.postcode }}</span>
        {{ ' ' }}
        <span><a :href="`tel:${cv.contact.phoneHref}`">{{ cv.contact.phone }}</a></span>
        {{ ' ' }}
        <span><a :href="`mailto:${cv.contact.email}`">{{ cv.contact.email }}</a></span>
        {{ ' ' }}
        <span><a :href="cv.site.links.github">{{ t('nav.github') }}</a></span>
        {{ ' ' }}
        <span><a :href="cv.site.links.linkedin">{{ t('nav.linkedin') }}</a></span>
      </p>
    </header>

    <p v-for="paragraph in cv.intro" :key="paragraph">{{ paragraph }}</p>

    <h2>{{ t('cv.experience') }}</h2>
    <article v-for="role in cv.roles" :key="`${role.company}-${role.start}`" class="cv-role">
      <h3>{{ role.title }} · {{ role.company }}</h3>
      <p class="meta">
        <time :datetime="role.start">{{ formatCvMonth(role.start, currentLocale) }}</time>
        <template v-if="role.end">
          – <time :datetime="role.end">{{ formatCvMonth(role.end, currentLocale) }}</time>
        </template>
        <template v-else> – {{ t('cv.present') }}</template>
        — {{ role.location }}
      </p>
      <p>{{ role.summary }}</p>
      <ul v-if="role.achievements.length > 0">
        <li v-for="achievement in role.achievements" :key="achievement">
          <template v-if="splitUpgradeFlourish(achievement)">
            {{ splitUpgradeFlourish(achievement)!.before }}version
            <span class="cv-jump cv-jump-six">6</span>{{ splitUpgradeFlourish(achievement)!.mid }}<span class="cv-jump cv-jump-seven">7</span>{{ splitUpgradeFlourish(achievement)!.after }}
          </template>
          <template v-else>{{ achievement }}</template>
        </li>
      </ul>
      <p v-if="role.note" class="cv-note">{{ role.note }}</p>
    </article>

    <h2>{{ t('cv.skills') }}</h2>
    <ul class="cv-skills">
      <li v-for="skill in cv.skills" :key="skill.name">{{ skill.name }}</li>
    </ul>

    <h2>{{ t('cv.education') }}</h2>
    <p v-if="cv.educationNote" class="lede">{{ cv.educationNote }}</p>
    <article
      v-for="education in cv.education"
      :key="`${education.school}-${education.start}`"
      class="cv-role"
    >
      <h3>{{ education.title }} · {{ education.school }}</h3>
      <p class="meta">
        <time :datetime="education.start">{{ formatCvMonth(education.start, currentLocale) }}</time>
        <template v-if="education.end">
          – <time :datetime="education.end">{{ formatCvMonth(education.end, currentLocale) }}</time>
        </template>
        <template v-else> – {{ t('cv.present') }}</template>
        — {{ education.location }}
      </p>
      <p v-if="education.note">{{ education.note }}</p>
    </article>
  </article>
</template>
