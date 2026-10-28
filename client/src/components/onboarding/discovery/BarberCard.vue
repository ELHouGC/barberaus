<template>
  <article class="overflow-hidden rounded-2xl border border-border-subtle bg-app-surface">
    <div class="relative">
      <button
        type="button"
        class="block w-full text-left"
        :aria-label="`View ${barber.name}'s profile`"
        @click="emit('open-profile', barber)"
      >
        <img
          :src="barber.image"
          :alt="barber.imageAlt"
          class="aspect-[4/4.3] w-full object-cover"
          loading="lazy"
        />
      </button>

      <span
        v-if="barber.available"
        class="absolute left-2 top-2 rounded-full bg-app-bg/85 px-2 py-1 text-[9px] font-bold text-emerald-400"
      >
        Available today
      </span>
    </div>

    <div class="p-3">
      <button
        type="button"
        class="block max-w-full truncate text-left text-sm font-bold hover:text-brand"
        @click="emit('open-profile', barber)"
      >
        {{ barber.name }}
      </button>

      <p class="mt-1 flex items-center gap-1.5 text-[11px] text-tx-secondary">
        <span class="font-bold text-brand">★ {{ barber.rating }}</span>
        <span>·</span>
        <span>{{ barber.distance }} km</span>
      </p>

      <p class="mt-1.5 truncate text-[11px] text-tx-muted">
        {{ barber.specialty }} · {{ barber.secondarySpecialty }}
      </p>

      <button
        type="button"
        class="mt-3 flex h-9 w-full items-center justify-center rounded-lg border text-xs font-bold transition"
        :class="followed ? 'border-brand bg-brand/10 text-brand' : 'border-[#3F3F46] text-tx-secondary hover:bg-[#27272A]'"
        :aria-pressed="followed"
        @click="emit('toggle-follow', barber.id)"
      >
        {{ followed ? 'Following ✓' : 'Follow' }}
      </button>
    </div>
  </article>
</template>

<script setup lang="ts">
import type { DemoBarber } from '@/types/discovery'

defineProps<{
  barber: DemoBarber
  followed: boolean
}>()

const emit = defineEmits<{
  (event: 'open-profile', barber: DemoBarber): void
  (event: 'toggle-follow', barberId: string): void
}>()
</script>
