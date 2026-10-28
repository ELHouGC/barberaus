import { ref, computed } from 'vue'
import { profileApi } from '@/transport/profile.api'
import { cloudinaryApi, CloudinaryUploadError } from '@/transport/cloudinary.api'
import type { CropCoordinates } from '@/transport/profile.contracts'

export type UploadStage = 'idle' | 'signing' | 'uploading' | 'saving' | 'done' | 'error'

export function useAvatarUpload() {
  const stage = ref<UploadStage>('idle')
  const alertKey = ref<string | null>(null)
  const uploadedUrl = ref<string | null>(null)

  // Cache for retry scenarios (if saving fails after Cloudinary upload succeeds)
  let cachedFile: Blob | File | null = null
  let cachedCropData: CropCoordinates | null = null
  let lastCloudinaryUrl: string | null = null

  const isLoading = computed(() => ['signing', 'uploading', 'saving'].includes(stage.value))

  async function executeUpload(fileOrBlob: Blob | File, cropData: CropCoordinates) {
    cachedFile = fileOrBlob
    cachedCropData = cropData
    alertKey.value = null

 try {
      stage.value = 'signing'
      const sigResponse = await profileApi.getSignature()
      if (!sigResponse.success) {
        stage.value = 'error'
        alertKey.value = sigResponse.error.i18n_key
        return
      }

      stage.value = 'uploading'
      const cloudinaryResult = await cloudinaryApi.uploadOriginalImage(fileOrBlob, sigResponse.data)
      lastCloudinaryUrl = cloudinaryResult.secure_url
      uploadedUrl.value = lastCloudinaryUrl
      console.log('croping data from useAvatarUpload', cropData)
      await executeSave(cloudinaryResult.secure_url, cropData)

    } catch (err: any) {
      stage.value = 'error'
      // If it's our custom Cloudinary error, use its i18nKey. Otherwise, fallback to network/unexpected error.
      if (err instanceof CloudinaryUploadError) {
        alertKey.value = err.i18nKey
      } else {
        alertKey.value = 'client.network'
      }
    }
  }

  async function executeSave(url: string, cropData: CropCoordinates) {
    stage.value = 'saving'
    const saveResponse = await profileApi.saveAvatar({
      avatar_url: url,
      crop_coordinates: cropData,
    })

    if (!saveResponse.success) {
      stage.value = 'error'
      // Handle specific server error codes if needed, otherwise fallback to server i18n_key
      const { error } = saveResponse
      
      if (error.code === 'INVALID_AVATAR_URL') {
        alertKey.value = error.i18n_key
        return
      }

      alertKey.value = error.i18n_key
      return
    }

    stage.value = 'done'
  }

  async function retrySave() {
    if (!lastCloudinaryUrl || !cachedCropData) {
      if (cachedFile && cachedCropData) {
        await executeUpload(cachedFile, cachedCropData)
      }
      return
    }

    await executeSave(lastCloudinaryUrl, cachedCropData)
  }

  return {
    stage,
    alertKey,
    uploadedUrl,
    isLoading,
    executeUpload,
    retrySave,
  }
}