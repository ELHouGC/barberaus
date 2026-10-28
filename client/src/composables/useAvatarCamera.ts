import { type Ref } from 'vue'
import { validateAvatarFile } from '@/utils/avatarValidation'

export function useAvatarCamera(
    cameraInputRef: Ref<HTMLInputElement | null>,
    onFileReady: (file: File) => Promise<void> | void,
    onError: (key: string) => void,
    isLoading: () => boolean,
) {
 
  function triggerCamera() {
    if (isLoading()) return
    cameraInputRef.value?.click()
  }

  async function onFileSelected(event: Event) {
    const target = event.target as HTMLInputElement
    const file = target.files?.item(0) ?? null

    target.value = ''

    if (!file) return

    const errorKey = await validateAvatarFile(file)

    if (errorKey) {
      onError(errorKey)
      return
    }

    await onFileReady(file)
  }

  return {
    cameraInputRef,
    triggerCamera,
    onFileSelected,
  }
}