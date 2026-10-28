<script setup lang="ts">
defineProps<{
  isLoading?: boolean
}>()
</script>

<template>
  <button
    :disabled="isLoading"
    class="flex h-11 w-full items-center justify-center gap-2 rounded-lg bg-[#F59E0B] text-[0.875rem] font-semibold text-[#09090B] transition hover:bg-[#D97706] disabled:opacity-60"
  >
    <svg v-if="isLoading" class="size-4 animate-spin" viewBox="0 0 24 24" fill="none">
      <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
      <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8v8H4z" />
    </svg>
    <slot />
  </button>
</template>
