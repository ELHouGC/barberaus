<template>
  <section class="mt-6 rounded-2xl border border-border-subtle bg-app-surface p-3.5">
    <div class="flex items-center gap-3">
      <span class="grid size-10 shrink-0 place-items-center rounded-xl bg-brand/10 text-brand" aria-hidden="true">
        <svg class="size-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8">
          <path d="M20 10c0 5-8 11-8 11S4 15 4 10a8 8 0 1 1 16 0Z" />
          <circle cx="12" cy="10" r="2.5" />
        </svg>
      </span>

      <div class="min-w-0 flex-1">
        <span class="block text-[10px] font-bold uppercase tracking-[0.14em] text-tx-muted">Search area</span>
        <span class="mt-0.5 block truncate text-sm font-bold">{{ city.trim() || 'Location not selected' }}</span>
        <span class="mt-0.5 block text-[11px] leading-4 text-tx-secondary">Choose an area or use your device location</span>
      </div>

      <button
        type="button"
        class="shrink-0 rounded-lg border border-[#3F3F46] px-3 py-2 text-xs font-bold text-tx-secondary transition hover:bg-[#27272A]"
        @click="emit('change-area')"
      >
        Change
      </button>
    </div>

    <button
      type="button"
      class="mt-3 flex h-10 w-full items-center justify-center gap-2 rounded-xl bg-[#27272A] text-xs font-bold text-tx-primary transition hover:bg-[#3F3F46] disabled:cursor-not-allowed disabled:opacity-60"
      :disabled="locationStatus === 'requesting'"
      @click="emit('detect-location')"
    >
      <span v-if="locationStatus === 'requesting'" class="animate-pulse">Detecting location…</span>
      <span v-else-if="locationStatus === 'success'">Location detected</span>
      <span v-else>Use my current location</span>
    </button>

    <p
      v-if="locationStatus === 'denied'"
      role="alert"
      class="mt-3 rounded-lg border border-warning-border bg-warning-bg p-3 text-xs text-warning-text"
    >
      Location permission wasn't granted. You can try again or choose your area manually.
    </p>
    <p
      v-else-if="locationStatus === 'unavailable'"
      role="alert"
      class="mt-3 rounded-lg border border-warning-border bg-warning-bg p-3 text-xs text-warning-text"
    >
      We couldn't get your location. Check your device settings and try again.
    </p>
    <p v-else-if="locationStatus === 'success'" role="status" class="mt-3 text-xs text-tx-secondary">
      Coordinates detected. Your search area still needs to be identified.
    </p>
  </section>
</template>

<script setup lang="ts">
import type { LocationStatus } from '@/types/discovery'

defineProps<{
  city: string
  locationStatus: LocationStatus
}>()

const emit = defineEmits<{
  (event: 'change-area'): void
  (event: 'detect-location'): void
}>()
</script>
