import { computed, ref } from 'vue'
import { useOnboardingStore } from '@/stores/onboarding.store'
import { demoBarbers } from '@/data/demoBarbers'
import type { DemoBarber, FilterSheet, FilterSheetName, SearchRadiusKm } from '@/types/discovery'

export const availableServices = [
  'Skin fades',
  'Beards',
  'Textured cuts',
  'Tapers',
  'Classic cuts',
  'Hot towel',
  'Creative cuts',
  'Color',
] as const

export function useBarberFilters() {
  const onboardingStore = useOnboardingStore()

  const activeSheet = ref<FilterSheet>(null)
  const manualCity = ref('')
  const minRating = ref(0)
  const selectedServices = ref<string[]>([])
  const availableOnly = ref(false)

  // Draft values prevent un-applied changes from affecting displayed results.
  const draftRadius = ref<SearchRadiusKm>(onboardingStore.searchRadiusKm)
  const draftMinRating = ref(0)
  const draftServices = ref<string[]>([])
  const draftAvailableOnly = ref(false)

  const activeFilterCount = computed(() => {
    let count = 0
    if (onboardingStore.searchRadiusKm !== 10) count++
    if (minRating.value > 0) count++
    if (selectedServices.value.length > 0) count++
    if (availableOnly.value) count++
    return count
  })

  const activeSheetTitle = computed(() => {
    switch (activeSheet.value) {
      case 'location': return 'Choose your area'
      case 'distance': return 'Search distance'
      case 'rating': return 'Minimum rating'
      case 'services': return 'Services'
      case 'availability': return 'Availability'
      case 'all': return 'All filters'
      default: return ''
    }
  })

  const filteredBarbers = computed(() => demoBarbers.filter((barber: DemoBarber) => {
    if (barber.distance > onboardingStore.searchRadiusKm) return false
    if (barber.rating < minRating.value) return false
    if (availableOnly.value && !barber.available) return false

    if (selectedServices.value.length > 0) {
      const barberServices = [barber.specialty, barber.secondarySpecialty]
      const hasSelectedService = selectedServices.value.some((service) =>
        barberServices.includes(service),
      )
      if (!hasSelectedService) return false
    }

    return true
  }))

  function openFilter(sheet: FilterSheetName) {
    manualCity.value = onboardingStore.city
    draftRadius.value = onboardingStore.searchRadiusKm
    draftMinRating.value = minRating.value
    draftServices.value = [...selectedServices.value]
    draftAvailableOnly.value = availableOnly.value
    activeSheet.value = sheet
  }

  function closeFilter() {
    activeSheet.value = null
  }

  function toggleDraftService(service: string) {
    const index = draftServices.value.indexOf(service)
    if (index === -1) draftServices.value.push(service)
    else draftServices.value.splice(index, 1)
  }

  function applyFilters() {
    onboardingStore.setSearchRadius(draftRadius.value)
    minRating.value = draftMinRating.value
    selectedServices.value = [...draftServices.value]
    availableOnly.value = draftAvailableOnly.value
    closeFilter()
  }

  function resetFilters() {
    onboardingStore.setSearchRadius(10)
    minRating.value = 0
    selectedServices.value = []
    availableOnly.value = false
    closeFilter()
  }

  function applyManualLocation(): boolean {
    const city = manualCity.value.trim()
    if (!city) return false

    // Manual city entry invalidates the previous coordinates; it does not geocode.
    onboardingStore.setCity(city)
    closeFilter()
    return true
  }

  function isFollowed(barberId: string) {
    return onboardingStore.followedBarberIds.includes(barberId)
  }

  return {
    activeSheet,
    activeSheetTitle,
    activeFilterCount,
    activeFilters: {
      minRating,
      selectedServices,
      availableOnly,
    },
    draftFilters: {
      draftRadius,
      draftMinRating,
      draftServices,
      draftAvailableOnly,
    },
    manualCity,
    filteredBarbers,
    openFilter,
    closeFilter,
    toggleDraftService,
    applyFilters,
    resetFilters,
    applyManualLocation,
    isFollowed,
  }
}
