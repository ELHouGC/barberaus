import type { AvatarSignatureResponse } from './profile.contracts'

export interface CloudinaryUploadResult {
  secure_url: string
  public_id: string
  [key: string]: any
}

export class CloudinaryUploadError extends Error {
  public i18nKey: string

  constructor(i18nKey: string, message: string) {
    super(message)
    this.name = 'CloudinaryUploadError'
    this.i18nKey = i18nKey
  }
}

export const cloudinaryApi = {
  async uploadOriginalImage(fileOrBlob: Blob | File, sig: AvatarSignatureResponse) {
    const formData = new FormData()
    formData.append('file', fileOrBlob, 'avatar-master.jpg')
    formData.append('api_key', sig.api_key)
    formData.append('timestamp', String(sig.timestamp))
    formData.append('folder', sig.folder)
    formData.append('signature', sig.signature)

    const response = await window.fetch(
      `https://api.cloudinary.com/v1_1/${sig.cloud_name}/image/upload`,
      {
        method: 'POST',
        body: formData,
      }
    )

    const data = await response.json()

    if (!response.ok) {
      const cloudinaryMsg = data.error?.message || ''
      
      // Map known Cloudinary messages to your i18n keys
      let i18nKey = 'cloudinary.upload_failed'
      if (cloudinaryMsg.toLowerCase().includes('file size') || cloudinaryMsg.toLowerCase().includes('too large')) {
        i18nKey = 'cloudinary.file_too_large'
      } else if (cloudinaryMsg.toLowerCase().includes('not a valid image')) {
        i18nKey = 'cloudinary.invalid_image'
      }

      throw new CloudinaryUploadError(i18nKey, cloudinaryMsg || `Upload failed with status ${response.status}`)
    }

    return data
  },
}