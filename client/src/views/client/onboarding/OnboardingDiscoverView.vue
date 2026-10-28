<template>
  <div class="flex flex-1 flex-col bg-app-bg text-[#FAFAFA]">
    <div class="flex-1 px-4 pb-28 pt-5">
      <OnboardingStepHeader step-label="Step 3 of 3" layout="compact" />

      <header class="mt-6">
        <h1 class="text-2xl font-bold tracking-[-0.025em]">Find Barbers in Your Area</h1>
        <p class="mt-2 text-sm leading-6 text-tx-secondary">
          Set your search area, explore nearby talent, and follow anyone you like.
        </p>
      </header>

      <SearchAreaCard
        :city="onboardingStore.city"
        :location-status="locationStatus"
        @change-area="openFilter('location')"
        @detect-location="detectLocation"
      />

      <div class="mt-7 flex items-end justify-between gap-4">
        <div>
          <h2 class="text-lg font-bold tracking-[-0.02em]">Suggested for you</h2>
          <p class="mt-1 text-xs text-tx-muted">Demo profiles for UI development</p>
        </div>
      </div>

      <DiscoveryFilterChips
        :search-radius-km="onboardingStore.searchRadiusKm"
        :min-rating="activeFilters.minRating.value"
        :selected-services-count="activeFilters.selectedServices.value.length"
        :available-only="activeFilters.availableOnly.value"
        :active-filter-count="activeFilterCount"
        @open-filter="openFilter"
      />

      <div v-if="filteredBarbers.length" class="mt-4 grid grid-cols-2 gap-3">
        <BarberCard
          v-for="barber in filteredBarbers"
          :key="barber.id"
          :barber="barber"
          :followed="isFollowed(barber.id)"
          @open-profile="openBarberProfile"
          @toggle-follow="toggleFollow"
        />
      </div>

      <div
        v-else
        class="mt-4 rounded-2xl border border-border-subtle bg-app-surface px-5 py-8 text-center"
      >
        <h3 class="text-sm font-bold text-tx-primary">No barbers match these filters</h3>
        <p class="mt-2 text-xs leading-5 text-tx-secondary">
          Try a wider search distance or remove some filters.
        </p>
        <button
          type="button"
          class="mt-4 rounded-xl bg-brand px-4 py-2.5 text-xs font-bold text-app-bg"
          @click="resetFilters"
        >
          Reset filters
        </button>
      </div>

      <p class="mt-4 text-center text-[10px] text-tx-muted">
        Illustrative profiles only — live barber results aren't connected yet.
      </p>
    </div>

    <BottomAction @click="emit('next')">Continue</BottomAction>

    <Teleport to="body">
      <BarberProfileSheet
        v-if="selectedBarber"
        :barber="selectedBarber"
        :followed="isFollowed(selectedBarber.id)"
        @close="closeBarberProfile"
        @toggle-follow="toggleFollow"
      />

      <div
        v-if="toastMessage"
        role="status"
        aria-live="polite"
        class="fixed bottom-24 left-1/2 z-[70] w-max max-w-[calc(100%-2rem)] -translate-x-1/2 rounded-xl border border-border-subtle bg-[#27272A] px-4 py-3 text-sm font-medium text-tx-primary shadow-xl"
      >
        {{ toastMessage }}
      </div>

      <DiscoveryFiltersSheet
        v-if="activeSheet"
        :active-sheet="activeSheet"
        :title="activeSheetTitle"
        :manual-city="manualCity"
        :draft-radius="draftFilters.draftRadius.value"
        :draft-min-rating="draftFilters.draftMinRating.value"
        :draft-services="draftFilters.draftServices.value"
        :draft-available-only="draftFilters.draftAvailableOnly.value"
        :services="availableServices"
        @close="closeFilter"
        @apply="applyFilters"
        @reset="resetFilters"
        @apply-location="handleManualLocationApply"
        @toggle-service="toggleDraftService"
        @update:manual-city="manualCity = $event"
        @update:draft-radius="draftFilters.draftRadius.value = $event"
        @update:draft-min-rating="draftFilters.draftMinRating.value = $event"
        @update:draft-available-only="draftFilters.draftAvailableOnly.value = $event"
      />

      <LocationServicesDialog
        v-if="locationServicesDialogOpen"
        :is-opening="isOpeningLocationSettings"
        :error-message="locationSettingsError"
        @open-settings="handleOpenLocationSettings"
        @close="locationServicesDialogOpen = false"
      />
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { onMounted, onBeforeUnmount, ref } from 'vue'
import BottomAction from '@/components/onboarding/BottomAction.vue'
import OnboardingStepHeader from '@/components/onboarding/OnboardingStepHeader.vue'
import BarberCard from '@/components/onboarding/discovery/BarberCard.vue'
import BarberProfileSheet from '@/components/onboarding/discovery/BarberProfileSheet.vue'
import DiscoveryFilterChips from '@/components/onboarding/discovery/DiscoveryFilterChips.vue'
import DiscoveryFiltersSheet from '@/components/onboarding/discovery/DiscoveryFiltersSheet.vue'
import LocationServicesDialog from '@/components/onboarding/discovery/LocationServicesDialog.vue'
import SearchAreaCard from '@/components/onboarding/discovery/SearchAreaCard.vue'
import { useBarberFilters, availableServices } from '@/composables/discovery/useBarberFilters'
import { demoBarbers } from '@/data/demoBarbers'
import { getCurrentDeviceLocation } from '@/composables/useLocation'
import { openLocationSettings } from 'tauri-plugin-location-settings-api'
import { useOnboardingStore } from '@/stores/onboarding.store'
import type { DemoBarber, LocationStatus } from '@/types/discovery'

const emit = defineEmits<{
  (event: 'next'): void
}>()

const onboardingStore = useOnboardingStore()
const {
  activeSheet,
  activeSheetTitle,
  activeFilterCount,
  activeFilters,
  draftFilters,
  manualCity,
  filteredBarbers,
  openFilter,
  closeFilter,
  toggleDraftService,
  applyFilters,
  resetFilters,
  applyManualLocation,
  isFollowed,
} = useBarberFilters()

const locationServicesDialogOpen = ref(false)
const locationStatus = ref<LocationStatus>('idle')
const isOpeningLocationSettings = ref(false)
const locationSettingsError = ref<string | null>(null)
const selectedBarber = ref<DemoBarber | null>(null)
const toastMessage = ref<string | null>(null)

let waitingForSettingsReturn = false
let toastTimeout: ReturnType<typeof setTimeout> | undefined

onMounted(() => {
  document.addEventListener('visibilitychange', handleVisibilityChange)
})

onBeforeUnmount(() => {
  document.removeEventListener('visibilitychange', handleVisibilityChange)
  if (toastTimeout) clearTimeout(toastTimeout)
})

async function detectLocation() {
  if (locationStatus.value === 'requesting') return

  locationStatus.value = 'requesting'

  try {
    const result = await getCurrentDeviceLocation()

    if (result.status === 'permission-denied') {
      locationStatus.value = 'denied'
      return
    }

    if (result.status === 'services-disabled') {
      locationStatus.value = 'disabled'
      locationServicesDialogOpen.value = true
      return
    }

    if (result.status === 'unavailable') {
      locationStatus.value = 'unavailable'
      return
    }

    onboardingStore.setCoordinates(result.latitude, result.longitude)
    locationStatus.value = 'success'
  } catch (error) {
    console.error('[onboarding] Location detection failed:', error)
    locationStatus.value = 'unavailable'
  }
}

async function handleOpenLocationSettings() {
  if (isOpeningLocationSettings.value) return

  isOpeningLocationSettings.value = true
  locationSettingsError.value = null

  try {
    await openLocationSettings()
    waitingForSettingsReturn = true
    locationServicesDialogOpen.value = false
  } catch (error) {
    console.error('[onboarding] Unable to open device settings:', error)
    locationSettingsError.value =
      'Could not open Settings. Please open your device settings manually.'
  } finally {
    isOpeningLocationSettings.value = false
  }
}

function handleVisibilityChange() {
  if (document.visibilityState !== 'visible' || !waitingForSettingsReturn) return

  waitingForSettingsReturn = false
  if (locationStatus.value === 'disabled') void detectLocation()
}

function handleManualLocationApply() {
  if (applyManualLocation()) locationStatus.value = 'idle'
}

function openBarberProfile(barber: DemoBarber) {
  selectedBarber.value = barber
}

function closeBarberProfile() {
  selectedBarber.value = null
}

function toggleFollow(barberId: string) {
  const barber = demoBarbers.find((item) => item.id === barberId)
  if (!barber) return

  const wasFollowed = isFollowed(barberId)
  onboardingStore.toggleFollowedBarber(barberId)
  showToast(wasFollowed ? `Unfollowed ${barber.name}` : `You're now following ${barber.name}`)
}

function showToast(message: string) {
  toastMessage.value = message
  if (toastTimeout) clearTimeout(toastTimeout)

  toastTimeout = setTimeout(() => {
    toastMessage.value = null
    toastTimeout = undefined
  }, 2500)
}
</script>
