// utils/avatarValidation.ts
export async function validateAvatarFile(file: File): Promise<string | null> {
  const MAX_SIZE = 5 * 1024 * 1024 // 5MB
  const allowedTypes = ['image/jpeg', 'image/png', 'image/webp']

  if (!allowedTypes.includes(file.type)) {
    return 'validation.invalid_format' // "Please select a valid JPEG, PNG, or WebP image."
  }

  if (file.size > MAX_SIZE) {
    return 'validation.file_too_large' // "The image file is too large. Maximum allowed size is 5MB."
  }

  return null
}
