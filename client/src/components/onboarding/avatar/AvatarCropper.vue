<template>
  <div class="absolute inset-0 z-50 flex flex-col bg-app-bg text-tx-primary">
    <div class="flex h-16 items-center justify-between px-4 border-b border-border-subtle">
      <button
        aria-label="Close photo editor"
        class="grid size-10 place-items-center rounded-full bg-app-surface border border-border-subtle"
        @click="$emit('cancel')"
        type="button"
      >
        ✕
      </button>
      <h2 class="font-bold text-tx-primary">Adjust Profile Photo</h2>
      <button
        v-if="hasExistingAvatar"
        class="px-1 text-sm font-bold text-error-text"
        @click="$emit('remove')"
        type="button"
      >
        Remove
      </button>
      <span v-else class="w-10" />
    </div>

    <div class="flex flex-1 flex-col items-center justify-center overflow-hidden px-4">
      <p class="mb-6 text-sm text-tx-secondary">Drag to reposition and slide to zoom</p>
      
      <div
        class="relative size-[260px] touch-none overflow-hidden rounded-full border-2 border-brand bg-app-surface shadow-2xl"
        @pointerdown="handlePointerDown"
        @pointermove="handlePointerMove"
        @pointerup="handlePointerUp"
        @pointercancel="handlePointerUp"
      >
        <img
          ref="imageRef"
          :src="imageSrc"
          alt="Crop Preview"
          @load="onImageLoad"
          class="pointer-events-none absolute left-1/2 top-1/2 max-w-none select-none"
          :style="{
            height: isLandscape ? '260px' : `${260 / aspectRatio}px`,
            width: isLandscape ? `${260 * aspectRatio}px` : '260px',
            transform: `translate(calc(-50% + ${offset.x}px), calc(-50% + ${offset.y}px)) scale(${zoom})`,
          }"
        />
      </div>

      <div class="mt-10 flex w-full max-w-sm items-center gap-4">
        <span class="text-xl text-tx-secondary">−</span>
        <input
          aria-label="Photo zoom"
          class="w-full accent-brand"
          max="3"
          min="1"
          step="0.05"
          type="range"
          v-model.number="zoom"
        />
        <span class="text-xl text-tx-secondary">+</span>
      </div>
    </div>

    <div class="grid grid-cols-2 gap-3 border-t border-border-subtle p-4 pb-[max(20px,env(safe-area-inset-bottom))] bg-app-surface">
      <button
        class="h-12 rounded-xl border border-border-subtle bg-app-bg text-sm font-semibold text-tx-primary hover:bg-border-subtle transition"
        @click="$emit('reselect')"
        type="button"
      >
        Retake / Reselect
      </button>
      <button
        class="h-12 rounded-xl bg-brand text-sm font-bold text-app-bg hover:bg-brand-hover transition"
        @click="handleSaveLocally"
        type="button"
      >
        Save &amp; Apply
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import type { CropCoordinates } from '@/transport/profile.contracts'

const props = defineProps<{
  imageSrc: string
  hasExistingAvatar: boolean
}>()

const emit = defineEmits<{
  (e: 'save', payload: { cropData: CropCoordinates; previewUrl: string }): void
  (e: 'cancel'): void
  (e: 'remove'): void
  (e: 'reselect'): void
}>()

const imageRef = ref<HTMLImageElement | null>(null)
const zoom = ref(1)
const offset = ref({ x: 0, y: 0 })
const naturalSize = ref({ width: 0, height: 0 })
const displaySize = { width: 260, height: 260 }
const containerSize = 260

const aspectRatio = computed(() => {
  if (!naturalSize.value.height) return 1
  return naturalSize.value.width / naturalSize.value.height
})

const isLandscape = computed(() => aspectRatio.value >= 1)

const renderedWidth = computed(() => {
  if (!naturalSize.value.width) return containerSize
  return isLandscape.value ? containerSize * aspectRatio.value * zoom.value : containerSize * zoom.value
})

const renderedHeight = computed(() => {
  if (!naturalSize.value.height) return containerSize
  return isLandscape.value ? containerSize * zoom.value : containerSize * (1 / aspectRatio.value) * zoom.value
})

// Immediately clamp offset whenever zoom changes to prevent gaps on zoom-out
watch(zoom, () => {
  const maxOffsetX = Math.max(0, (renderedWidth.value - containerSize) / 2)
  const maxOffsetY = Math.max(0, (renderedHeight.value - containerSize) / 2)

  offset.value = {
    x: Math.min(maxOffsetX, Math.max(-maxOffsetX, offset.value.x)),
    y: Math.min(maxOffsetY, Math.max(-maxOffsetY, offset.value.y)),
  }
})

function onImageLoad(e: Event) {
  const img = e.target as HTMLImageElement
  naturalSize.value = { width: img.naturalWidth, height: img.naturalHeight }
}

let pointers = new Map<number, { x: number; y: number }>()
let lastPointer: { x: number; y: number } | null = null

let pinchStartDistance = 0
let pinchStartZoom = 1

// Keep these limits consistent with the existing slider.
const MIN_ZOOM = 1
const MAX_ZOOM = 3

function getPointerDistance() {
  const [a, b] = [...pointers.values()]

  if (!a || !b) return 0

  return Math.hypot(b.x - a.x, b.y - a.y)
}
function handlePointerDown(event: PointerEvent) {
  ;(event.currentTarget as HTMLElement).setPointerCapture(event.pointerId)

  pointers.set(event.pointerId, {
    x: event.clientX,
    y: event.clientY,
  })

  if (pointers.size === 2) {
    pinchStartDistance = getPointerDistance()
    pinchStartZoom = zoom.value
    lastPointer = null
  } else if (pointers.size === 1) {
    lastPointer = { x: event.clientX, y: event.clientY }
  }
}

function handlePointerMove(event: PointerEvent) {
  if (!pointers.has(event.pointerId)) return

  pointers.set(event.pointerId, {
    x: event.clientX,
    y: event.clientY,
  })

  // Two fingers: pinch to zoom.
  if (pointers.size === 2) {
    const distance = getPointerDistance()

    if (pinchStartDistance > 0 && distance > 0) {
      zoom.value = Math.min(
        MAX_ZOOM,
        Math.max(
          MIN_ZOOM,
          pinchStartZoom * (distance / pinchStartDistance),
        ),
      )
    }

    return
  }

  // One finger: move the image.
  if (pointers.size === 1 && lastPointer) {
    const dx = event.clientX - lastPointer.x
    const dy = event.clientY - lastPointer.y

    const maxOffsetX = Math.max(
      0,
      (renderedWidth.value - containerSize) / 2,
    )

    const maxOffsetY = Math.max(
      0,
      (renderedHeight.value - containerSize) / 2,
    )

    offset.value = {
      x: Math.min(
        maxOffsetX,
        Math.max(-maxOffsetX, offset.value.x + dx),
      ),
      y: Math.min(
        maxOffsetY,
        Math.max(-maxOffsetY, offset.value.y + dy),
      ),
    }

    lastPointer = { x: event.clientX, y: event.clientY }
  }
}

function handlePointerUp(event: PointerEvent) {
  pointers.delete(event.pointerId)

  if (pointers.size === 1) {
    // Allow one-finger dragging to resume smoothly after pinching.
    const remaining = [...pointers.values()][0]
    lastPointer = remaining ? { ...remaining } : null
  } else {
    lastPointer = null
  }

  if (pointers.size < 2) {
    pinchStartDistance = 0
  }
}

function handleSaveLocally() {
  const image = imageRef.value
  if (!image?.naturalWidth || !image.naturalHeight) return

  const canvas = document.createElement('canvas')
  canvas.width = 320
  canvas.height = 320
  const ctx = canvas.getContext('2d')
  if (ctx) {
    const guideSize = 260
    const baseScale = Math.max(guideSize / image.naturalWidth, guideSize / image.naturalHeight)
    const outputScale = canvas.width / guideSize
    ctx.translate(canvas.width / 2 + offset.value.x * outputScale, canvas.height / 2 + offset.value.y * outputScale)
    ctx.scale(baseScale * zoom.value * outputScale, baseScale * zoom.value * outputScale)
    ctx.drawImage(image, -image.naturalWidth / 2, -image.naturalHeight / 2)
  }
  const previewUrl = canvas.toDataURL('image/jpeg', 0.92)

  const natW = naturalSize.value.width
  const natH = naturalSize.value.height
  const baseScale = Math.max(displaySize.width / natW, displaySize.height / natH)
  const currentScale = baseScale * zoom.value

  const visibleWidthInNatural = displaySize.width / currentScale
  const visibleHeightInNatural = displaySize.height / currentScale

  const centerX = natW / 2 - (offset.value.x / currentScale)
  const centerY = natH / 2 - (offset.value.y / currentScale)

  let x = Math.round(centerX - visibleWidthInNatural / 2)
  let y = Math.round(centerY - visibleHeightInNatural / 2)
  let width = Math.round(visibleWidthInNatural)
  let height = Math.round(visibleHeightInNatural)

  x = Math.max(0, Math.min(x, natW - width))
  y = Math.max(0, Math.min(y, natH - height))
  width = Math.min(width, natW - x)
  height = Math.min(height, natH - y)

  const cropData: CropCoordinates = { x, y, width, height }
  emit('save', { cropData, previewUrl })
}
</script>