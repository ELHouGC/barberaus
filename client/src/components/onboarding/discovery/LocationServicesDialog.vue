<template>
  <div
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/75 px-6 backdrop-blur-sm"
    @click.self="emit('close')"
  >
    <section
      role="dialog"
      aria-modal="true"
      aria-labelledby="location-services-title"
      aria-describedby="location-services-description"
      class="w-full max-w-sm rounded-3xl border border-border-subtle bg-app-surface p-5 shadow-2xl"
    >
      <div class="grid size-12 place-items-center rounded-2xl bg-brand/10 text-brand" aria-hidden="true">
        <svg class="size-6" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
          <path d="M20 10c0 5-8 11-8 11S4 15 4 10a8 8 0 1 1 16 0Z" />
          <circle cx="12" cy="10" r="2.5" />
        </svg>
      </div>

      <h2 id="location-services-title" class="mt-4 text-lg font-bold text-tx-primary">Turn on Location Services</h2>
      <p id="location-services-description" class="mt-2 text-sm leading-6 text-tx-secondary">
        Barberaus needs your device's location to find barbers near you. Enable Location Services in your device settings, then return here and try again.
      </p>

      <div class="mt-5 space-y-2">
        <button
          type="button"
          class="h-12 w-full rounded-xl bg-brand text-sm font-bold text-app-bg transition hover:bg-brand-hover disabled:cursor-wait disabled:opacity-60"
          :disabled="isOpening"
          @click="emit('open-settings')"
        >
          {{ isOpening ? 'Opening Settings…' : 'Open Settings' }}
        </button>
        <p v-if="errorMessage" role="alert" class="mt-2 text-xs leading-5 text-error-text">{{ errorMessage }}</p>
        <button
          type="button"
          class="h-12 w-full rounded-xl border border-border-subtle text-sm font-semibold text-tx-primary transition hover:bg-[#27272A]"
          @click="emit('close')"
        >
          Not now
        </button>
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  isOpening: boolean
  errorMessage: string | null
}>()

const emit = defineEmits<{
  (event: 'open-settings'): void
  (event: 'close'): void
}>()
</script>
