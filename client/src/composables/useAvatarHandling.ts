import { ref, computed, onBeforeUnmount } from 'vue'
import { useAvatarUpload } from '@/composables/useAvatarUpload'
import type { CropCoordinates } from '@/transport/profile.contracts'

export function useAvatarHandling() {
  const displayPreviewUrl = ref<string | null>(null)
  const sourceSheetOpen = ref(false)
  const cropSource = ref<string | null>(null)
  const validationErrorKey = ref<string | null>(null)
  const avatarDebugMessage = ref<string | null>(null)

  const pendingRawFile = ref<File | null>(null)
  const pendingCropData = ref<CropCoordinates | null>(null)
  const cropCandidateFile = ref<File | null>(null)

  const cropSourceObjectUrl = ref<string | null>(null)
  const displayPreviewObjectUrl = ref<string | null>(null)

  const { stage, alertKey, isLoading, executeUpload } = useAvatarUpload()

  const visibleErrorKey = computed(
    () => validationErrorKey.value ?? alertKey.value,
  )

  function releaseCropSourceObjectUrl(preserveUrl?: string) {
    const url = cropSourceObjectUrl.value
    cropSourceObjectUrl.value = null
    if (url && url !== preserveUrl) {
      URL.revokeObjectURL(url)
    }
  }

  function setDisplayPreviewUrl(url: string | null) {
    const previousOwnedUrl = displayPreviewObjectUrl.value
    if (previousOwnedUrl && previousOwnedUrl !== url) {
      URL.revokeObjectURL(previousOwnedUrl)
    }
    displayPreviewUrl.value = url
    displayPreviewObjectUrl.value = url?.startsWith('blob:') ? url : null
  }

  function fileToDataUrl(file: File): Promise<string> {
    return new Promise((resolve, reject) => {
      const reader = new FileReader()
      reader.onload = () => {
        if (typeof reader.result !== 'string') {
          reject(new Error('No image data was returned'))
          return
        }
        resolve(reader.result)
      }
      reader.onerror = () => {
        reject(reader.error ?? new Error('Failed to read image'))
      }
      reader.readAsDataURL(file)
    })
  }

  async function openCropperForFile(file: File) {
    try {
      validationErrorKey.value = null
      const previewUrl = await fileToDataUrl(file)
      cropCandidateFile.value = file
      cropSource.value = previewUrl
    } catch (error) {
      console.error('[Avatar] Failed to prepare selected image:', error)
      avatarDebugMessage.value = `Could not prepare image: ${
        error instanceof Error ? error.message : String(error)
      }`
    }
  }

  function closeCropper() {
    cropSource.value = null
    cropCandidateFile.value = null
    releaseCropSourceObjectUrl()
  }

  function handleAvatarClick() {
    if (isLoading.value) return
    validationErrorKey.value = null

    if (!displayPreviewUrl.value) {
      sourceSheetOpen.value = true
      return
    }

    if (pendingRawFile.value) {
      openCropperForFile(pendingRawFile.value)
      return
    }

    releaseCropSourceObjectUrl()
    cropCandidateFile.value = null
    cropSource.value = displayPreviewUrl.value
  }

  function cancelCropper() {
    closeCropper()
  }

  function handleReselectAvatar() {
    closeCropper()
    sourceSheetOpen.value = true
  }

  function onCropSavedLocally(payload: {
    cropData: CropCoordinates
    previewUrl: string
  }) {
    const candidateFile = cropCandidateFile.value
    releaseCropSourceObjectUrl(payload.previewUrl)

    cropSource.value = null
    cropCandidateFile.value = null

    if (candidateFile) {
      pendingRawFile.value = candidateFile
    }

    pendingCropData.value = payload.cropData
    setDisplayPreviewUrl(payload.previewUrl)
    validationErrorKey.value = null
  }

  function handleRemoveAvatar() {
    closeCropper()
    setDisplayPreviewUrl(null)
    pendingRawFile.value = null
    pendingCropData.value = null
    cropCandidateFile.value = null
    validationErrorKey.value = null
    sourceSheetOpen.value = false
  }

  function resetState() {
    closeCropper()
    setDisplayPreviewUrl(null)
    pendingRawFile.value = null
    pendingCropData.value = null
    cropCandidateFile.value = null
    validationErrorKey.value = null
    sourceSheetOpen.value = false
  }

  onBeforeUnmount(() => {
    const urls = new Set([
      cropSourceObjectUrl.value,
      displayPreviewObjectUrl.value,
    ])
    for (const url of urls) {
      if (url) URL.revokeObjectURL(url)
    }
    cropSourceObjectUrl.value = null
    displayPreviewObjectUrl.value = null
  })

  return {
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
    closeCropper,
    handleAvatarClick,
    cancelCropper,
    handleReselectAvatar,
    onCropSavedLocally,
    handleRemoveAvatar,
    resetState,
    executeUpload,
    alertKey,
  }
}