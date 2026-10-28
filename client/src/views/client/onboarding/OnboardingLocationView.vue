<template>
  <div class="flex flex-1 flex-col px-4 pb-8 pt-5">
    <OnboardingStepHeader
      step-label="Step 2 of 3"
      layout="compact"
      show-skip
      @skip="emit('skip')"
    />

    <header class="mt-7">
      <h1 class="text-2xl font-bold tracking-tight text-tx-primary">
        Where do you get fresh?
      </h1>

      <p class="mt-2 text-sm leading-6 text-tx-secondary">
        Discover barbers and trending cuts near you.
      </p>
    </header>

    <div class="mt-8 space-y-4">
      <button
        type="button"
        class="flex h-14 w-full items-center justify-between rounded-xl border border-border-subtle bg-app-surface px-4 text-left transition hover:border-tx-muted disabled:cursor-not-allowed disabled:opacity-60"
        :disabled="locationStatus === 'requesting'"
        @click="detectLocation"
      >
        <span class="text-sm font-medium text-tx-primary">
          {{
            locationStatus === 'requesting'
              ? 'Detecting Location…'
              : 'Use Current Location'
          }}
        </span>

        <span class="text-tx-muted" aria-hidden="true">→</span>
      </button>

      <p
        v-if="locationMessage"
        class="px-1 text-xs leading-5"
        :class="
          locationStatus === 'success'
            ? 'text-success'
            : 'text-tx-secondary'
        "
        role="status"
        aria-live="polite"
      >
        {{ locationMessage }}
      </p>

      <label class="block">
        <span class="mb-2 block text-sm font-medium text-tx-primary">
          Selected City
        </span>

        <input
          :value="onboardingStore.city"
          type="text"
          placeholder="Enter your city or area"
          class="h-14 w-full rounded-xl border border-border-subtle bg-app-surface px-4 text-sm text-tx-primary outline-none transition placeholder:text-tx-muted focus:border-brand"
          @input="handleCityInput"
        />
      </label>
    </div>

    <fieldset class="mt-8">
      <legend class="text-sm font-medium text-tx-primary">
        Search Radius
      </legend>

      <div class="mt-3 grid grid-cols-4 gap-2">
        <button
          v-for="radius in radiusOptions"
          :key="radius"
          type="button"
          :aria-pressed="onboardingStore.searchRadiusKm === radius"
          :class="[
            'h-10 rounded-full border text-xs transition',
            onboardingStore.searchRadiusKm === radius
              ? 'border-brand bg-brand font-bold text-app-bg'
              : 'border-border-subtle bg-app-surface font-medium text-tx-secondary hover:border-tx-muted',
          ]"
          @click="onboardingStore.setSearchRadius(radius)"
        >
          {{ radius }} km
        </button>
      </div>
    </fieldset>

    <div class="mt-6 rounded-lg border border-border-subtle bg-app-surface p-3">
      <p class="text-xs leading-5 text-tx-secondary">
        {{
          onboardingStore.city.trim()
            ?  `Your selected search area is ${onboardingStore.city.trim()}, within ${onboardingStore.searchRadiusKm} km.` 
            : 'Enter a city to choose your search area.'
        }}
      </p>
    </div>
  </div>

  <BottomAction
    :disabled="!onboardingStore.canContinueWithLocation"
    @click="emit('next')"
  >
    Set Location &amp; Continue
  </BottomAction>
</template>
<script setup lang="ts">
import { computed, ref } from 'vue'
import BottomAction from '@/components/onboarding/BottomAction.vue'
import OnboardingStepHeader from '@/components/onboarding/OnboardingStepHeader.vue'
import { useOnboardingStore, type SearchRadiusKm } from '@/stores/onboarding.store'
import { getCurrentDeviceLocation } from '@/composables/useLocation'

type LocationStatus = 'idle' | 'requesting' | 'success' | 'denied' | 'error'

const emit = defineEmits<{
  (event: 'next'): void
  (event: 'skip'): void
}>()

const onboardingStore = useOnboardingStore()
const locationStatus = ref<LocationStatus>('idle')
const detectedAccuracyMeters = ref<number | null>(null)

const radiusOptions: SearchRadiusKm[] = [3, 5, 10, 25]

const locationMessage = computed(() => {
  switch (locationStatus.value) {
    case 'requesting':
      return 'Requesting your location…'

    case 'success':
      return detectedAccuracyMeters.value !== null
        ? `Location detected (±${Math.round(detectedAccuracyMeters.value)} m accuracy).`
        : 'Location detected.'

    case 'denied':
      return 'Location permission was denied. Enter your city manually.'

    case 'error':
      return 'Location is unavailable. Check that Location Services are enabled, or enter your city manually.'

    default:
      return null
  }
})

function handleCityInput(event: Event) {
  const input = event.target

  if (!(input instanceof HTMLInputElement)) return

  onboardingStore.setCity(input.value)

  if (locationStatus.value === 'success') {
    locationStatus.value = 'idle'
    detectedAccuracyMeters.value = null
  }
}

async function detectLocation() {
  if (locationStatus.value === 'requesting') return

  locationStatus.value = 'requesting'
  detectedAccuracyMeters.value = null

  try {
    const result = await getCurrentDeviceLocation()

    if (result.status === 'permission-denied') {
      locationStatus.value = 'denied'
      return
    }

    if (result.status === 'unavailable') {
      locationStatus.value = 'error'
      return
    }

    onboardingStore.setCoordinates(
      result.latitude,
      result.longitude,
    )

    detectedAccuracyMeters.value = result.accuracyMeters
    locationStatus.value = 'success'
  } catch {
    locationStatus.value = 'error'
  }
}
</script>