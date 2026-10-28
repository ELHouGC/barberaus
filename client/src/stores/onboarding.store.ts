
import { computed, ref } from 'vue'
import { defineStore } from 'pinia'

import type { CropCoordinates } from '@/transport/profile.contracts'

export type StyleCategory = 'gents' | 'ladies' | 'all'

export type SearchRadiusKm = 3 | 5 | 10 | 25

export type AvatarSaveStatus = 'empty' | 'draft' | 'saved'

export interface OnboardingAvatarDraft {
  previewUrl: string | null
  cropCoordinates: CropCoordinates | null
  status: AvatarSaveStatus
}

export const useOnboardingStore = defineStore('onboarding', () => {
  // Avatar metadata
  const avatar = ref<OnboardingAvatarDraft>({
    previewUrl: null,
    cropCoordinates: null,
    status: 'empty',
  })

  // Style DNA preferences
  const selectedStyleCategory = ref<StyleCategory>('gents')
  const selectedStylePhotoIds = ref<string[]>([])

  // Location preferences
  const city = ref('')
  const latitude = ref<number | null>(null)
  const longitude = ref<number | null>(null)
  const searchRadiusKm = ref<SearchRadiusKm>(5)

  // Followed barbers
  const followedBarberIds = ref<string[]>([])

  // Completion state
  const isComplete = ref(false)

  const canContinueWithStyles = computed(
    () => selectedStylePhotoIds.value.length >= 3,
  )

  const canContinueWithLocation = computed(
    () => city.value.trim().length > 0,
  )

  function setAvatarDraft(
    previewUrl: string | null,
    cropCoordinates: CropCoordinates | null,
  ) {
    avatar.value = {
      previewUrl,
      cropCoordinates,
      status: previewUrl ? 'draft' : 'empty',
    }
  }

  function markAvatarSaved() {
    if (avatar.value.status !== 'empty') {
      avatar.value.status = 'saved'
    }
  }

  function clearAvatar() {
    avatar.value = {
      previewUrl: null,
      cropCoordinates: null,
      status: 'empty',
    }
  }

  function setStyleCategory(category: StyleCategory) {
    selectedStyleCategory.value = category
  }

  function toggleStylePhoto(photoId: string) {
    const index = selectedStylePhotoIds.value.indexOf(photoId)

    if (index === -1) {
      selectedStylePhotoIds.value.push(photoId)
    } else {
      selectedStylePhotoIds.value.splice(index, 1)
    }
  }

  function setCity(value: string) {
    city.value = value

    // Manual city changes invalidate coordinates from an earlier location.
    latitude.value = null
    longitude.value = null
  }

  function setCoordinates(lat: number, lng: number) {
    latitude.value = lat
    longitude.value = lng
  }

  function setSearchRadius(radius: SearchRadiusKm) {
    searchRadiusKm.value = radius
  }

  function toggleFollowedBarber(barberId: string) {
    const index = followedBarberIds.value.indexOf(barberId)

    if (index === -1) {
      followedBarberIds.value.push(barberId)
    } else {
      followedBarberIds.value.splice(index, 1)
    }
  }

  function resetDraft() {
    clearAvatar()

    selectedStyleCategory.value = 'gents'
    selectedStylePhotoIds.value = []

    city.value = ''
    latitude.value = null
    longitude.value = null
    searchRadiusKm.value = 5

    followedBarberIds.value = []
    isComplete.value = false
  }

  return {
    avatar,
    selectedStyleCategory,
    selectedStylePhotoIds,
    city,
    latitude,
    longitude,
    searchRadiusKm,
    followedBarberIds,
    isComplete,
    canContinueWithStyles,
    canContinueWithLocation,
    setAvatarDraft,
    markAvatarSaved,
    clearAvatar,
    setStyleCategory,
    toggleStylePhoto,
    setCity,
    setCoordinates,
    setSearchRadius,
    toggleFollowedBarber,
    resetDraft,
  }
})
