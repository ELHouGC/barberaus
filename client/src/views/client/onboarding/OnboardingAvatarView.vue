<template>
  <!-- Keep this parent route mounted while its nested camera route is active. -->
  <template v-if="!isCameraRoute">
    <!-- Header -->
    <OnboardingStepHeader
      step-label="Step 1 of 3"
      show-skip
      @skip="handleSkip"
    />

    <!-- Main Content -->
    <div
      class="flex flex-1 flex-col items-center justify-center px-4 py-8 text-center"
    >
      <div class="mx-auto mb-8 max-w-xs">
        <h1 class="text-2xl font-bold tracking-tight text-tx-primary">
          {{ displayPreviewUrl ? 'Looking Sharp!' : 'Add a Profile Photo' }}
        </h1>

        <p class="mt-2 text-sm leading-6 text-tx-secondary">
          {{
            displayPreviewUrl
              ? 'Tap your photo anytime to replace or adjust it.'
              : 'Barbers use this to recognize you when you arrive for your cut.'
          }}
        </p>
      </div>

      <!-- Avatar Preview -->
      <AvatarPicker
        :imageUrl="displayPreviewUrl"
        @click="handleAvatarClick"
      />

      <p class="mt-6 text-xs text-tx-muted">
        {{
          displayPreviewUrl
            ? 'Tap to fine-tune your crop'
            : 'Tap the avatar or choose a source below'
        }}
      </p>

      <!-- Source Buttons -->
      <div
        v-if="!displayPreviewUrl"
        class="mt-8 w-full max-w-xs space-y-3"
      >
        <button
          class="flex h-12 w-full items-center justify-center gap-2 rounded-xl bg-brand text-sm font-bold text-app-bg transition hover:bg-brand-hover"
          type="button"
          @click="triggerGallery"
        >
          Upload Photo from Gallery
        </button>

        <button
          class="flex h-12 w-full items-center justify-center gap-2 rounded-xl border border-border-subtle bg-app-surface text-sm font-semibold text-tx-primary transition hover:border-tx-muted disabled:cursor-not-allowed disabled:opacity-60"
          type="button"
          :disabled="cameraLaunchPending || isLoading"
          @click="selectCamera"
        >
          {{ cameraLaunchPending ? 'Opening camera…' : 'Take Picture' }}
        </button>
      </div>

      <!-- Validation / Upload Errors -->
      <div class="mt-4 w-full px-4 space-y-2">
        <div
          v-if="cameraLaunchPending"
          class="text-xs font-medium text-tx-secondary"
          role="status"
          aria-live="polite"
        >
          Checking camera permission…
        </div>
        <div
          v-if="cameraErrorMessage"
          class="rounded-lg border border-error-border bg-error-bg px-3 py-3 text-[13px] text-error-text"
          role="alert"
        >
          {{ cameraErrorMessage }}
        </div>
        <div
          v-if="visibleErrorKey"
          class="rounded-lg border border-error-border bg-error-bg px-3 py-3 text-[13px] text-error-text"
          role="alert"
        >
          {{ translate(visibleErrorKey) }}
        </div>
      </div>

      <!-- Upload Progress -->
      <div
        v-if="isLoading"
        class="mt-4 animate-pulse text-xs font-semibold text-brand"
        role="status"
        aria-live="polite"
      >
        Uploading &amp; saving profile... ({{ stage }})
      </div>
    </div>

    
    <!-- Continue -->
    <template v-if="displayPreviewUrl">
      <BottomAction
        :disabled="isLoading"
        @click="handleContinue"
      >
        {{ isLoading ? 'Processing...' : 'Continue to Style DNA' }}
      </BottomAction>
    </template>

    <!-- Source Selection -->
    <AvatarSourceSheet
      v-if="sourceSheetOpen"
      @close="sourceSheetOpen = false"
      @select-gallery="selectGallery"
      @select-camera="selectCamera"
    />

    <!-- Cropper Modal -->
    <AvatarCropper
      v-if="cropSource"
      :imageSrc="cropSource"
      :hasExistingAvatar="!!displayPreviewUrl"
      @cancel="cancelCropper"
      @remove="handleRemoveAvatar"
      @reselect="handleReselectAvatar"
      @save="onCropSavedLocally"
    />
  </template>

  <RouterView v-else v-slot="{ Component }">
    <component
      :is="Component"
      v-if="Component"
      @captured="handleCameraCaptured"
      @close="handleCameraClose"
    />
  </RouterView>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import { RouterView, useRoute, useRouter } from 'vue-router'
import { checkPermissions, requestPermissions } from 'tauri-plugin-camera-kessdev'
import OnboardingStepHeader from '@/components/onboarding/OnboardingStepHeader.vue'
import type { CropCoordinates } from '@/transport/profile.contracts'
import BottomAction from '@/components/onboarding/BottomAction.vue'
import AvatarPicker from '@/components/onboarding/avatar/AvatarPicker.vue'
import AvatarSourceSheet from '@/components/onboarding/avatar/AvatarSourceSheet.vue'
import AvatarCropper from '@/components/onboarding/avatar/AvatarCropper.vue'
import { useAvatarHandling } from '@/composables/useAvatarHandling'
import { useAvatarGallery } from '@/composables/useAvatarGallery'
import { computed, ref } from 'vue'

const emit = defineEmits<{
  (e: 'next'): void
  (e: 'skip'): void
  (e: 'avatar-saved', payload: {
    previewUrl: string | null
    cropCoordinates: CropCoordinates | null
  }): void
}>()

const { t: translate } = useI18n()
const route = useRoute()
const router = useRouter()

// The camera is a child route so this component's avatar state stays alive.
const isCameraRoute = computed(() => route.name === 'onboarding-avatar-camera')
const cameraLaunchPending = ref(false)
const cameraErrorMessage = ref<string | null>(null)

// Core handling & state composable
const {
  displayPreviewUrl,
  sourceSheetOpen,
  cropSource,
  validationErrorKey,
  avatarDebugMessage,
  pendingRawFile,
  pendingCropData,
  stage,
  isLoading,
  visibleErrorKey,
  openCropperForFile,
  handleAvatarClick,
  cancelCropper,
  handleReselectAvatar,
  onCropSavedLocally,
  handleRemoveAvatar,
  resetState,
  executeUpload,
  alertKey,
} = useAvatarHandling()

// Gallery flow composable
const { triggerGallery } = useAvatarGallery(
  openCropperForFile,
  (key) => { validationErrorKey.value = key },
  (msg) => { avatarDebugMessage.value = msg },
  () => isLoading.value,
)

// Source Sheet wrappers
function selectGallery() {
  sourceSheetOpen.value = false
  cameraErrorMessage.value = null
  triggerGallery()
}

async function selectCamera() {
  if (cameraLaunchPending.value || isLoading.value) return

  sourceSheetOpen.value = false
  cameraErrorMessage.value = null
  cameraLaunchPending.value = true

  try {
    console.log("checking for permission")
    let permission = await checkPermissions()

    if (permission !== 'granted') {
      await requestPermissions()
      permission = await checkPermissions()
    }

    if (permission !== 'granted') {
      cameraErrorMessage.value =
        'Camera access was not granted. Enable camera permission in your device settings and try again.'
      return
    }

    
    await router.push({ name: 'onboarding-avatar-camera' })
  } catch (error) {
    // Keep native/plugin details in logs; show a stable message to the user.
    console.error('[avatar] Failed to authorize or open camera route:', error)
    cameraErrorMessage.value =
      'Unable to open the camera. Check camera permission and try again.'
  } finally {
    cameraLaunchPending.value = false
  }
}

async function handleCameraCaptured(file: File) {
  console.info('[avatar] Received captured photo', {
    name: file.name,
    type: file.type,
    size: file.size,
  })

  try {
    openCropperForFile(file)
  } catch (error) {
    console.error(
      '[avatar] Failed to open captured photo in cropper:',
      error,
    )

    cameraErrorMessage.value =
      'The photo was captured, but could not be opened. Please try again.'
  } finally {
    // Never leave the user stranded on the camera route.
    await returnToAvatarRoute()
  }
}
function handleCameraClose() {
  void returnToAvatarRoute()
}

async function returnToAvatarRoute() {
  try {
    await router.replace({ name: 'onboarding-avatar' })
  } catch (error) {
    console.error('[avatar] Failed to return from camera route:', error)
  }
}

// Continue onboarding
async function handleContinue() {
  if (isLoading.value || cameraLaunchPending.value) return
  validationErrorKey.value = null

  if (pendingRawFile.value && pendingCropData.value) {
    // Preserve the coordinates in case executeUpload clears pending state.
    const cropCoordinates: CropCoordinates = {
      ...pendingCropData.value,
    }

    await executeUpload(pendingRawFile.value, cropCoordinates)

    // Do not advance or report success if the upload failed.
    if (alertKey.value) return

    emit('avatar-saved', {
      previewUrl: displayPreviewUrl.value,
      cropCoordinates,
    })

    emit('next')
    return
  }

  // No new upload is needed, but an existing avatar may already be saved.
  if (displayPreviewUrl.value) {
    emit('avatar-saved', {
      previewUrl: displayPreviewUrl.value,
      cropCoordinates: pendingCropData.value
        ? { ...pendingCropData.value }
        : null,
    })
  }

  emit('next')
}







// Skip onboarding
function handleSkip() {
  if (isLoading.value || cameraLaunchPending.value) return
  resetState()
  emit('skip')
}
</script>