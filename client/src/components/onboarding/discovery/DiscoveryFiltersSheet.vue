<template>
  <div
    class="fixed inset-0 z-40 flex items-end justify-center bg-black/75 px-4 pb-0 backdrop-blur-sm sm:items-center sm:pb-4"
    @click.self="emit('close')"
  >
    <section
      role="dialog"
      aria-modal="true"
      aria-labelledby="filter-sheet-title"
      class="max-h-[85dvh] w-full max-w-md overflow-y-auto rounded-t-3xl border border-border-subtle bg-app-surface p-5 pb-8 sm:rounded-3xl"
    >
      <div class="mb-5 flex items-center justify-between gap-4">
        <h2 id="filter-sheet-title" class="text-lg font-bold text-tx-primary">{{ title }}</h2>
        <button
          type="button"
          class="grid size-9 place-items-center rounded-full bg-[#27272A] text-tx-secondary"
          aria-label="Close sheet"
          @click="emit('close')"
        >
          ✕
        </button>
      </div>

      <div v-if="activeSheet === 'location'" class="space-y-4">
        <p class="text-sm leading-6 text-tx-secondary">Enter the city or area where you want to find barbers.</p>
        <label for="manual-city" class="block text-xs font-semibold text-tx-secondary">City or area</label>
        <input
          id="manual-city"
          :value="manualCity"
          type="text"
          maxlength="100"
          autocomplete="address-level2"
          placeholder="e.g. Algiers"
          class="h-12 w-full rounded-xl border border-border-subtle bg-app-bg px-3 text-sm text-tx-primary outline-none placeholder:text-tx-muted focus:border-brand"
          @input="emit('update:manual-city', ($event.target as HTMLInputElement).value)"
          @keydown.enter="emit('apply-location')"
        />
        <p class="text-xs leading-5 text-tx-muted">This sets your search area manually. It does not determine GPS coordinates.</p>
        <button
          type="button"
          class="h-12 w-full rounded-xl bg-brand text-sm font-bold text-app-bg disabled:opacity-50"
          :disabled="!manualCity.trim()"
          @click="emit('apply-location')"
        >
          Use this area
        </button>
      </div>

      <div v-else class="space-y-6">
        <div v-if="includes('distance')">
          <h3 class="mb-3 text-sm font-semibold text-tx-primary">Search distance</h3>
          <div class="grid grid-cols-4 gap-2">
            <button
              v-for="radius in [3, 5, 10, 25] as const"
              :key="radius"
              type="button"
              class="rounded-xl border py-3 text-sm font-bold transition"
              :class="draftRadius === radius ? 'border-brand bg-brand/10 text-brand' : 'border-border-subtle text-tx-secondary'"
              :aria-pressed="draftRadius === radius"
              @click="emit('update:draft-radius', radius)"
            >
              {{ radius }} km
            </button>
          </div>
        </div>

        <div v-if="includes('rating')">
          <h3 class="mb-3 text-sm font-semibold text-tx-primary">Minimum rating</h3>
          <div class="flex flex-wrap gap-2">
            <button
              v-for="rating in [0, 4.5, 4.7, 4.8, 4.9]"
              :key="rating"
              type="button"
              class="rounded-xl border px-3 py-2.5 text-xs font-bold transition"
              :class="draftMinRating === rating ? 'border-brand bg-brand/10 text-brand' : 'border-border-subtle text-tx-secondary'"
              :aria-pressed="draftMinRating === rating"
              @click="emit('update:draft-min-rating', rating)"
            >
              {{ rating === 0 ? 'Any rating' : `★ ${rating}+` }}
            </button>
          </div>
        </div>

        <div v-if="includes('services')">
          <h3 class="mb-3 text-sm font-semibold text-tx-primary">Services</h3>
          <div class="flex flex-wrap gap-2">
            <button
              v-for="service in services"
              :key="service"
              type="button"
              class="rounded-xl border px-3 py-2.5 text-xs font-semibold transition"
              :class="draftServices.includes(service) ? 'border-brand bg-brand/10 text-brand' : 'border-border-subtle text-tx-secondary'"
              :aria-pressed="draftServices.includes(service)"
              @click="emit('toggle-service', service)"
            >
              {{ service }}
            </button>
          </div>
          <p class="mt-2 text-xs text-tx-muted">Barbers matching at least one selected service will be shown.</p>
        </div>

        <div
          v-if="includes('availability')"
          class="flex items-center justify-between gap-4 rounded-xl border border-border-subtle p-3"
        >
          <div>
            <h3 class="text-sm font-semibold text-tx-primary">Available today</h3>
            <p class="mt-1 text-xs text-tx-secondary">Show only demo profiles marked available.</p>
          </div>
          <input
            :checked="draftAvailableOnly"
            type="checkbox"
            class="size-5 accent-amber-500"
            aria-label="Only show available barbers"
            @change="emit('update:draft-available-only', ($event.target as HTMLInputElement).checked)"
          />
        </div>

        <div class="flex gap-3 pt-2">
          <button
            v-if="activeSheet === 'all'"
            type="button"
            class="h-12 flex-1 rounded-xl border border-border-subtle text-sm font-semibold text-tx-secondary"
            @click="emit('reset')"
          >
            Reset
          </button>
          <button type="button" class="h-12 flex-1 rounded-xl bg-brand text-sm font-bold text-app-bg" @click="emit('apply')">
            Apply filters
          </button>
        </div>
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
import type { FilterSheet, SearchRadiusKm } from '@/types/discovery'

const props = defineProps<{
  activeSheet: Exclude<FilterSheet, null>
  title: string
  manualCity: string
  draftRadius: SearchRadiusKm
  draftMinRating: number
  draftServices: string[]
  draftAvailableOnly: boolean
  services: readonly string[]
}>()

const emit = defineEmits<{
  (event: 'close'): void
  (event: 'apply'): void
  (event: 'reset'): void
  (event: 'apply-location'): void
  (event: 'toggle-service', service: string): void
  (event: 'update:manual-city', value: string): void
  (event: 'update:draft-radius', value: SearchRadiusKm): void
  (event: 'update:draft-min-rating', value: number): void
  (event: 'update:draft-available-only', value: boolean): void
}>()

function includes(section: 'distance' | 'rating' | 'services' | 'availability') {
  return props.activeSheet === 'all' || props.activeSheet === section
}
</script>
