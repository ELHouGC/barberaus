<template>
  <div
    class="-mx-4 mt-4 flex gap-2 overflow-x-auto px-4 pb-2 [scrollbar-width:none] [&::-webkit-scrollbar]:hidden"
    aria-label="Barber filters"
  >
    <button
      type="button"
      :class="chipClass(searchRadiusKm !== 10)"
      :aria-pressed="searchRadiusKm !== 10"
      @click="emit('open-filter', 'distance')"
    >
      {{ searchRadiusKm }} km <span aria-hidden="true">⌄</span>
    </button>

    <button
      type="button"
      :class="chipClass(minRating > 0)"
      :aria-pressed="minRating > 0"
      @click="emit('open-filter', 'rating')"
    >
      {{ minRating ? `${minRating.toFixed(1)}+ rating` : 'Rating' }}
      <span aria-hidden="true">⌄</span>
    </button>

    <button
      type="button"
      :class="chipClass(selectedServicesCount > 0)"
      :aria-pressed="selectedServicesCount > 0"
      @click="emit('open-filter', 'services')"
    >
      Services<span v-if="selectedServicesCount"> · {{ selectedServicesCount }}</span>
      <span aria-hidden="true">⌄</span>
    </button>

    <button
      type="button"
      :class="chipClass(availableOnly)"
      :aria-pressed="availableOnly"
      @click="emit('open-filter', 'availability')"
    >
      Available <span aria-hidden="true">⌄</span>
    </button>

    <button
      type="button"
      :class="chipClass(activeFilterCount > 0)"
      :aria-pressed="activeFilterCount > 0"
      @click="emit('open-filter', 'all')"
    >
      <svg class="size-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" aria-hidden="true">
        <path d="M4 6h16M7 12h10M10 18h4" />
      </svg>
      All filters
      <span v-if="activeFilterCount">· {{ activeFilterCount }}</span>
      <span aria-hidden="true">⌄</span>
    </button>
  </div>
</template>

<script setup lang="ts">
import type { FilterSheetName, SearchRadiusKm } from '@/types/discovery'

withDefaults(defineProps<{
  searchRadiusKm: SearchRadiusKm
  minRating: number
  selectedServicesCount: number
  availableOnly: boolean
  activeFilterCount: number
}>(), {})

const emit = defineEmits<{
  (event: 'open-filter', sheet: FilterSheetName): void
}>()

function chipClass(active: boolean) {
  return [
    'flex h-9 shrink-0 items-center gap-1.5 rounded-full border px-3 text-xs font-semibold transition',
    active
      ? 'border-brand bg-brand/10 text-brand'
      : 'border-[#3F3F46] bg-app-surface text-[#D4D4D8]',
  ]
}
</script>
