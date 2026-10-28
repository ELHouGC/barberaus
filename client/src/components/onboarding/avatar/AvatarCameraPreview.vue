
<template>
  <div
    class="camera-preview-screen fixed inset-0 z-50 flex flex-col bg-transparent text-white"
  >
    <!-- Header -->
    <header class="flex items-center justify-between p-4">
      <button
        type="button"
        class="rounded-full bg-black/40 px-4 py-2 text-sm font-semibold"
        :disabled="isStarting"
        @click="close"
      >
        Cancel
      </button>

      <span class="text-sm font-semibold drop-shadow">
        Take profile photo
      </span>

      <!-- Spacer to balance the header -->
      <div class="w-[68px]" />
    </header>

    <!-- Status / errors -->
    <div class="flex flex-1 items-center justify-center px-6">
      <div
        v-if="isStarting"
        class="rounded-xl bg-black/60 px-4 py-3 text-sm"
        role="status"
      >
        Starting camera…
      </div>

      <div
        v-else-if="errorMessage"
        class="max-w-sm rounded-xl bg-black/75 p-4 text-center"
        role="alert"
      >
        <p>{{ errorMessage }}</p>

        <button
          type="button"
          class="mt-4 rounded-lg bg-white px-4 py-2 text-sm font-semibold text-black"
          @click="close"
        >
          Close camera
        </button>
      </div>
    </div>

    <!-- Bottom controls; capture is M2 -->
    <footer
      class="flex justify-center px-6 pb-[max(32px,env(safe-area-inset-bottom))] pt-4"
    >
      <p class="rounded-full bg-black/50 px-4 py-2 text-xs">
        Position your face in the camera
      </p>
    </footer>
  </div>
</template>

<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref } from 'vue'
import {
  checkPermissions,
  requestPermissions,
  startPreview,
  stopPreview,
} from 'tauri-plugin-camera-kessdev'

const emit = defineEmits<{
  (event: 'close'): void
}>()

const isStarting = ref(true)
const errorMessage = ref<string | null>(null)

let disposed = false
let previewStarted = false

async function initializeCamera() {
  try {
    let permission = await checkPermissions()

    if (permission !== 'granted') {
      await requestPermissions()
      permission = await checkPermissions()
    }

    if (permission !== 'granted') {
      throw new Error('Camera permission was not granted.')
    }

    if (disposed) return
    console.log('[camera] requesting preview')
    await startPreview({
      camera: 'front',
      windowed: true,
    })
    console.log('[camera] preview started')

    previewStarted = true

    // The view may have been closed while preview startup was pending.
    if (disposed) {
      await stopPreview()
      previewStarted = false
    }
  } catch (error) {
    if (!disposed) {
      errorMessage.value =
        error instanceof Error
          ? error.message
          : 'Unable to start the camera.'
    }
  } finally {
    if (!disposed) {
      isStarting.value = false
    }
  }
}

async function close() {
  await releaseCamera()
  emit('close')
}

async function releaseCamera() {
  if (!previewStarted) return

  previewStarted = false

  try {
    await stopPreview()
  } catch (error) {
    console.error('Failed to stop camera preview:', error)
  }
}

onMounted(() => {
  void initializeCamera()
})

onBeforeUnmount(() => {
  disposed = true
  void releaseCamera()
})
</script>
