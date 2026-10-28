import { ref } from 'vue'
import { open } from '@tauri-apps/plugin-dialog'
import { readFile } from '@tauri-apps/plugin-fs'
import { validateAvatarFile } from '@/utils/avatarValidation'

export function useAvatarGallery(
  onFileReady: (file: File) => Promise<void> | void,
  onError: (key: string) => void,
  onDebugMessage: (msg: string | null) => void,
  isLoading: () => boolean,
) {
  const validationErrorKey = ref<string | null>(null)

  async function triggerGallery() {
    if (isLoading()) return

    validationErrorKey.value = null
    onDebugMessage(null)

    try {
      const selected = await open({
        multiple: false,
        directory: false,
        pickerMode: 'image',
        fileAccessMode: 'copy',
        filters: [
          {
            name: 'Supported images',
            extensions: ['image/jpeg', 'image/png', 'image/webp'],
          },
        ],
      })

      if (!selected || Array.isArray(selected)) return

      const bytes = await readFile(selected)

      if (bytes.byteLength === 0) {
        throw new Error('The selected image is empty')
      }

      const image = createSupportedImageFile(bytes)

      if (!image) {
        validationErrorKey.value = 'validation.avatar.invalid_format'
        onError('validation.avatar.invalid_format')
        return
      }

      const errorKey = await validateAvatarFile(image)

      if (errorKey) {
        validationErrorKey.value = errorKey
        onError(errorKey)
        return
      }

      await onFileReady(image)
    } catch (error) {
      console.error('[Avatar] Native gallery failed:', error)
      onDebugMessage(
        `Gallery error: ${error instanceof Error ? error.message : String(error)}`,
      )
    }
  }

  function createSupportedImageFile(bytes: Uint8Array): File | null {
    let type: string
    let extension: string

    if (
      bytes.length >= 8 &&
      bytes[0] === 0x89 &&
      bytes[1] === 0x50 &&
      bytes[2] === 0x4e &&
      bytes[3] === 0x47
    ) {
      type = 'image/png'
      extension = 'png'
    } else if (
      bytes.length >= 3 &&
      bytes[0] === 0xff &&
      bytes[1] === 0xd8 &&
      bytes[2] === 0xff
    ) {
      type = 'image/jpeg'
      extension = 'jpg'
    } else if (
      bytes.length >= 12 &&
      String.fromCharCode(...bytes.subarray(0, 4)) === 'RIFF' &&
      String.fromCharCode(...bytes.subarray(8, 12)) === 'WEBP'
    ) {
      type = 'image/webp'
      extension = 'webp'
    } else {
      return null
    }

    const buffer = new ArrayBuffer(bytes.byteLength)
    new Uint8Array(buffer).set(bytes)

    return new File([buffer], `avatar.${extension}`, { type })
  }

  return {
    triggerGallery,
  }
}