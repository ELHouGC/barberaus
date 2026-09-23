<script setup lang="ts">
defineProps<{
  title: string
  subtitle: string
  hideHeaderIcon?: boolean
}>()
</script>

<template>
  <div class="flex min-h-screen w-full items-center justify-center bg-[#09090B] px-4 py-10">
    <div class="w-full max-w-md rounded-xl border border-[#27272A] bg-[#18181B] p-8 shadow-[0_20px_25px_-5px_rgba(0,0,0,0.5)]">
      
      <div v-if="!hideHeaderIcon" class="mb-6 flex flex-col items-center gap-2">
        <div class="flex items-center gap-2">
          <!-- Scissors Icon -->
          <svg class="size-6 text-[#F59E0B]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
            <circle cx="6" cy="6" r="3" />
            <circle cx="6" cy="18" r="3" />
            <path d="M20 4 8.12 15.88M14.47 14.48 20 20M8.12 8.12 12 12" />
          </svg>
          <span class="text-lg font-bold tracking-tight text-[#FAFAFA]">BarberOS</span>
        </div>
      </div>
      
      <slot name="custom-header"></slot>

      <h1 class="text-center text-[1.5rem] font-semibold text-[#FAFAFA]">{{ title }}</h1>
      <p class="mt-1 text-center text-[0.875rem] text-[#A1A1AA]">
        <slot name="subtitle">{{ subtitle }}</slot>
      </p>

      <slot />

      <div class="mt-5 text-center text-[0.875rem] text-[#A1A1AA]">
        <slot name="footer" />
      </div>
      
    </div>
  </div>
</template>