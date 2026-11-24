<script setup lang="ts">
import {
  createDialogController,
  Dialog,
  DialogBackdrop,
  DialogClose,
  DialogContent,
  DialogTitle,
} from '@ui-sdk/vue/dialog'
import { nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'

const props = defineProps<{
  contentKey?: string
}>()

const controller = createDialogController()
const open = ref(false)
const activeLabel = ref('Enlarged diagram')
const stageEl = ref<HTMLElement | null>(null)

let originFigure: HTMLElement | null = null
let movedSvg: SVGSVGElement | null = null
const cleanups: Array<() => void> = []

function restoreSvg() {
  if (movedSvg && originFigure) {
    originFigure.insertBefore(movedSvg, originFigure.firstChild)
  }
  originFigure = null
  movedSvg = null
}

function openFigure(figure: HTMLElement) {
  const svg = figure.querySelector('svg')
  if (!svg) return
  restoreSvg()
  originFigure = figure
  movedSvg = svg
  activeLabel.value = figure.getAttribute('aria-label') || 'Diagram'
  open.value = true
  nextTick(() => {
    if (stageEl.value && movedSvg) {
      stageEl.value.replaceChildren(movedSvg)
    }
  })
}

function onOpenChange(next: boolean) {
  if (!next) {
    restoreSvg()
  }
}

function wireFigures() {
  for (const cleanup of cleanups) cleanup()
  cleanups.length = 0

  if (typeof document === 'undefined') return
  const figures = document.querySelectorAll<HTMLElement>('.prose figure.diagram')
  figures.forEach((figure) => {
    if (figure.querySelector('.lightbox-trigger')) return
    const button = document.createElement('button')
    button.type = 'button'
    button.className = 'lightbox-trigger'
    button.setAttribute(
      'aria-label',
      `Enlarge diagram: ${figure.getAttribute('aria-label') || 'diagram'}`,
    )
    const handler = () => openFigure(figure)
    button.addEventListener('click', handler)
    figure.classList.add('is-zoomable')
    figure.appendChild(button)
    cleanups.push(() => {
      button.removeEventListener('click', handler)
      button.remove()
      figure.classList.remove('is-zoomable')
    })
  })
}

onMounted(() => {
  wireFigures()
})

watch(
  () => props.contentKey,
  () => {
    restoreSvg()
    open.value = false
    nextTick(wireFigures)
  },
)

onBeforeUnmount(() => {
  restoreSvg()
  for (const cleanup of cleanups) cleanup()
})
</script>

<template>
  <Dialog
    v-model:open="open"
    :controller="controller"
    @open-change="onOpenChange"
  >
    <DialogBackdrop class="lightbox-backdrop" />
    <DialogContent class="lightbox" keep-mounted>
      <DialogTitle class="visually-hidden">{{ activeLabel }}</DialogTitle>
      <DialogClose class="lightbox-close" aria-label="Close diagram">
        <svg
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="1.7"
          focusable="false"
          aria-hidden="true"
        >
          <path stroke-linecap="round" stroke-linejoin="round" d="M6 18 18 6M6 6l12 12" />
        </svg>
      </DialogClose>
      <div ref="stageEl" class="lightbox-stage" />
    </DialogContent>
  </Dialog>
</template>
