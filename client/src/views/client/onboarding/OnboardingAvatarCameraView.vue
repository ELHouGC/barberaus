<template>
  <div
    class="fixed inset-0 z-50 flex h-dvh min-h-screen flex-col overflow-hidden bg-transparent text-white"
    role="dialog"
    aria-modal="true"
    aria-label="Take a profile photo"
  >
    <!-- Once captured, show the still image in place of the native live preview. -->
    <img
      v-if="capturedPreviewUrl"
      :src="capturedPreviewUrl"
      alt="Captured profile photo preview"
      class="pointer-events-none absolute inset-0 z-0 h-full w-full object-cover"
    />

    <!-- Keep the live or frozen preview visible while adding contrast for controls. -->
    <div
      aria-hidden="true"
      class="pointer-events-none absolute inset-0 z-[1] bg-gradient-to-b from-black/65 via-transparent to-black/75"
    />

    <header
      class="relative z-10 flex items-center justify-between px-5 pb-4 pt-[max(1rem,env(safe-area-inset-top))]"
    >
      <button
        type="button"
        class="rounded-full bg-black/45 px-4 py-2.5 text-sm font-semibold backdrop-blur-sm transition hover:bg-black/60 disabled:cursor-not-allowed disabled:opacity-60"
        :disabled="isCapturing"
        @click="handleClose"
      >
        Cancel
      </button>

      <h1 class="text-sm font-semibold drop-shadow">
        {{ capturedPreviewUrl ? 'Review your photo' : 'Take profile photo' }}
      </h1>

      <!-- Balance the title against the Cancel button. -->
      <div class="w-[68px]" aria-hidden="true" />
    </header>

    <section class="relative z-10 flex flex-1 items-center justify-center px-6 py-6">
      <div
        v-if="isStarting"
        class="rounded-2xl bg-black/60 px-5 py-4 text-center shadow-lg backdrop-blur-sm"
        role="status"
        aria-live="polite"
      >
        <span class="block text-sm font-semibold">Starting camera…</span>
        <span class="mt-1 block text-xs text-white/75">Please wait a moment.</span>
      </div>

      <div
        v-else-if="errorMessage"
        class="w-full max-w-sm rounded-2xl bg-black/80 p-5 text-center shadow-xl backdrop-blur-sm"
        role="alert"
        aria-live="assertive"
      >
        <h2 class="text-base font-semibold">Camera unavailable</h2>
        <p class="mt-2 text-sm leading-6 text-white/80">{{ errorMessage }}</p>
        <button
          type="button"
          class="mt-5 w-full rounded-xl bg-white px-4 py-3 text-sm font-bold text-black transition hover:bg-white/90 disabled:opacity-60"
          :disabled="isStarting || isCapturing"
          @click="startCamera"
        >
          Try again
        </button>
      </div>

      <div
        v-else-if="capturedPreviewUrl"
        class="rounded-full bg-black/45 px-4 py-2 text-sm font-medium shadow-lg backdrop-blur-sm"
        role="status"
        aria-live="polite"
      >
        Review your photo before continuing.
      </div>

      <div v-else-if="previewReady" class="flex w-full flex-col items-center">
        <div
          class="relative aspect-square w-[min(72vw,320px)] rounded-full border-2 border-white/90 shadow-[0_0_0_9999px_rgba(0,0,0,0.12)]"
          aria-hidden="true"
        >
          <div class="absolute inset-[12%] rounded-full border border-white/35" />
          <span class="absolute left-1/2 top-0 h-5 w-px -translate-x-1/2 bg-white/80" />
          <span class="absolute bottom-0 left-1/2 h-5 w-px -translate-x-1/2 bg-white/80" />
        </div>

        <p class="mt-7 max-w-xs text-center text-sm font-medium drop-shadow">
          Center your face in the frame and look toward the camera.
        </p>
        <p class="mt-2 text-center text-xs text-white/70">
          Make sure your face is well lit.
        </p>
      </div>
    </section>

    <footer
      class="relative z-10 flex min-h-32 flex-col items-center justify-start px-6 pb-[max(1.5rem,env(safe-area-inset-bottom))] pt-2"
    >
      <div v-if="capturedPreviewUrl" class="flex w-full max-w-sm gap-3">
        <button
          type="button"
          class="flex-1 rounded-xl border border-white/70 bg-black/45 px-4 py-3 text-sm font-semibold shadow-lg backdrop-blur-sm transition hover:bg-black/60 disabled:cursor-not-allowed disabled:opacity-60"
          :disabled="isStarting || isCapturing"
          @click="retakePhoto"
        >
          Retake
        </button>
        <button
          type="button"
          class="flex-1 rounded-xl bg-white px-4 py-3 text-sm font-bold text-black shadow-lg transition hover:bg-white/90 disabled:cursor-not-allowed disabled:opacity-60"
          :disabled="isStarting || isCapturing"
          @click="usePhoto"
        >
          Use photo
        </button>
      </div>

      <button
        v-else-if="previewReady && !errorMessage"
        type="button"
        class="group flex size-[76px] items-center justify-center rounded-full border-4 border-white/90 bg-white/20 shadow-lg transition hover:bg-white/30 active:scale-95 disabled:cursor-not-allowed disabled:opacity-50"
        :disabled="!previewReady || isCapturing"
        :aria-label="isCapturing ? 'Capturing photo' : 'Take photo'"
        @click="capturePhoto"
      >
        <span class="size-[58px] rounded-full bg-white transition group-hover:bg-white/95" />
      </button>

      <p
        v-if="isCapturing"
        class="mt-3 text-xs font-semibold drop-shadow"
        role="status"
        aria-live="polite"
      >
        Capturing photo…
      </p>
      <p
        v-else-if="captureErrorMessage"
        class="mt-3 max-w-xs text-center text-xs font-medium text-red-100 drop-shadow"
        role="alert"
        aria-live="assertive"
      >
        {{ captureErrorMessage }}
      </p>
      <p v-else-if="previewReady" class="mt-3 text-xs text-white/75 drop-shadow">
        Tap the button to take your photo
      </p>
    </footer>
  </div>
</template>

<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref } from 'vue'
import {
  capture,
  startPreview,
  stopPreview,
} from 'tauri-plugin-camera-kessdev'
import { readFile } from '@tauri-apps/plugin-fs';

const emit = defineEmits<{
  (event: 'captured', file: File): void
  (event: 'close'): void
}>()

type CapturedPhoto = {
  path?: string
  dataUrl?: string
  blob?: Blob
  format?: string
}

const isStarting = ref(false)
const isCapturing = ref(false)
const previewReady = ref(false)
const errorMessage = ref<string | null>(null)
const captureErrorMessage = ref<string | null>(null)
const capturedFile = ref<File | null>(null)
const capturedPreviewUrl = ref<string | null>(null)

let disposed = false
let previewSessionRequested = false
let previewStartTask: Promise<unknown> | null = null
let previewStopTask: Promise<void> | null = null

/**
 * Starts the native preview. Permission is handled by the onboarding route
 * before navigating here, so this component owns preview/capture lifecycle only.
 */
async function startCamera(): Promise<void> {
  if (disposed || isStarting.value || isCapturing.value) return

  isStarting.value = true
  previewReady.value = false
  errorMessage.value = null
  captureErrorMessage.value = null

  // A retry starts a new preview lifecycle after the previous one was released.
  previewStartTask = null
  previewStopTask = null
  previewSessionRequested = true

  try {
    previewStartTask = startPreview({ camera: 'front', windowed: true })
    await previewStartTask

    if (disposed) {
      await releaseCamera()
      return
    }

    previewReady.value = true
  } catch (error) {
    console.error('[avatar-camera] Failed to start preview:', error)
    await releaseCamera()

    if (!disposed) {
      errorMessage.value =
        'The camera could not be started. Check camera access and try again.'
    }
  } finally {
    if (!disposed) isStarting.value = false
  }
}

/**
 * Idempotent cleanup. If unmount happens while startPreview is still pending,
 * wait for it to settle before asking the native plugin to stop the preview.
 */
function releaseCamera(): Promise<void> {
  if (previewStopTask) return previewStopTask

  previewStopTask = (async () => {
    if (previewStartTask) {
      try {
        await previewStartTask
      } catch {
        // Still make a best-effort stop request after a failed startup.
      }
    }

    if (!previewSessionRequested) return
    previewSessionRequested = false
    previewReady.value = false

    try {
      await stopPreview()
    } catch (error) {
      console.error('[avatar-camera] Failed to stop preview:', error)
    }
  })()

  return previewStopTask
}

async function photoToFile(photo: CapturedPhoto): Promise<File> {
  let blob: Blob

  if (photo.blob instanceof Blob) {
    blob = photo.blob
  } else if (photo.dataUrl) {
    const response = await fetch(photo.dataUrl)
    if (!response.ok) throw new Error('Could not read the captured image data.')
    blob = await response.blob()
  } else if (photo.path) {
  // Read the native temporary file directly through Tauri.
  const bytes = await readFile(photo.path)
  blob = new Blob([bytes as BlobPart])}
   else {
    throw new Error('The camera did not return a readable image.')
  }

  const format = photo.format?.toLowerCase()
  const fallbackMimeType = format === 'png' ? 'image/png' : 'image/jpeg'
  const mimeType = blob.type.startsWith('image/') ? blob.type : fallbackMimeType
  const extension = mimeType === 'image/png' ? 'png' : 'jpg'
  const timestamp = new Date().toISOString().replace(/[:.]/g, '-')

  return new File([blob], `avatar-${timestamp}.${extension}`, {
    type: mimeType,
    lastModified: Date.now(),
  })
}

async function capturePhoto(): Promise<void> {
  if (!previewReady.value || isCapturing.value || disposed) return

  isCapturing.value = true
  captureErrorMessage.value = null

  try {
    const photo = (await capture({ format: 'jpeg', quality: 0.92 })) as CapturedPhoto
    if (disposed) return

    const file = await photoToFile(photo)
    if (disposed) return

    // Freeze the screen on the captured image before offering the next action.
    await releaseCamera()
    if (disposed) return

    clearCapturedPreview()
    capturedFile.value = file
    capturedPreviewUrl.value = URL.createObjectURL(file)
  } catch (error) {
    console.error('[avatar-camera] Failed to capture photo:', error)
    if (!disposed) {
      captureErrorMessage.value =
        'Could not capture that photo. Please try again.'
    }
  } finally {
    if (!disposed) isCapturing.value = false
  }
}

function clearCapturedPreview(): void {
  if (capturedPreviewUrl.value) {
    URL.revokeObjectURL(capturedPreviewUrl.value)
  }

  capturedPreviewUrl.value = null
  capturedFile.value = null
}

function usePhoto(): void {
  const file = capturedFile.value
  if (!file || disposed) return

  // The parent will pass the accepted file into the existing cropper flow.
  emit('captured', file)
  clearCapturedPreview()
}

async function retakePhoto(): Promise<void> {
  if (!capturedFile.value || isStarting.value || isCapturing.value || disposed) return

  clearCapturedPreview()
  await startCamera()
}

async function handleClose(): Promise<void> {
  if (isCapturing.value) return

  await releaseCamera()
  if (!disposed) emit('close')
}

onMounted(() => {
  void startCamera()
})

onBeforeUnmount(() => {
  disposed = true
  clearCapturedPreview()
  void releaseCamera()
})
</script>
