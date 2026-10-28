<template>
  <div
    class="fixed inset-0 z-50 flex items-end justify-center bg-black/75 px-4 backdrop-blur-sm sm:items-center sm:py-6"
    @click.self="emit('close')"
  >
    <section
      role="dialog"
      aria-modal="true"
      aria-labelledby="barber-profile-title"
      class="max-h-[88dvh] w-full max-w-md overflow-y-auto rounded-t-3xl border border-border-subtle bg-app-surface pb-6 sm:rounded-3xl"
    >
      <div class="relative">
        <img :src="barber.image" :alt="barber.imageAlt" class="aspect-[16/9] w-full object-cover" />

        <button
          type="button"
          class="absolute right-3 top-3 grid size-10 place-items-center rounded-full bg-app-bg/85 text-lg text-tx-primary"
          aria-label="Close barber profile"
          @click="emit('close')"
        >
          ×
        </button>

        <span
          v-if="barber.available"
          class="absolute bottom-3 left-3 rounded-full bg-app-bg/90 px-3 py-1.5 text-xs font-semibold text-emerald-400"
        >
          Available today
        </span>
      </div>

      <div class="p-5">
        <h2 id="barber-profile-title" class="text-xl font-bold text-tx-primary">{{ barber.name }}</h2>
        <div class="mt-2 flex flex-wrap items-center gap-x-3 gap-y-1 text-sm">
          <span class="font-bold text-brand">★ {{ barber.rating.toFixed(1) }}</span>
          <span class="text-tx-secondary">{{ barber.distance }} km away</span>
        </div>

        <div class="mt-5">
          <h3 class="text-sm font-bold text-tx-primary">Specialties</h3>
          <div class="mt-3 flex flex-wrap gap-2">
            <span
              v-for="service in [barber.specialty, barber.secondarySpecialty]"
              :key="service"
              class="rounded-full border border-border-subtle bg-app-bg px-3 py-2 text-xs font-medium text-tx-secondary"
            >
              {{ service }}
            </span>
          </div>
        </div>

        <p class="mt-5 text-xs leading-5 text-tx-muted">
          Demo profile for UI development. Detailed business information and real availability will come from the backend.
        </p>

        <button
          type="button"
          class="mt-5 h-12 w-full rounded-xl text-sm font-bold transition"
          :class="followed ? 'border border-border-subtle bg-app-bg text-tx-primary' : 'bg-brand text-app-bg hover:bg-brand-hover'"
          :aria-pressed="followed"
          @click="emit('toggle-follow', barber.id)"
        >
          {{ followed ? 'Following ✓' : 'Follow barber' }}
        </button>
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
import type { DemoBarber } from '@/types/discovery'

defineProps<{
  barber: DemoBarber
  followed: boolean
}>()

const emit = defineEmits<{
  (event: 'close'): void
  (event: 'toggle-follow', barberId: string): void
}>()
</script>
