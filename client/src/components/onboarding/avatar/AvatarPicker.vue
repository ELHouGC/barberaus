<template>
  <button
    aria-label="Choose profile photo source"
    class="relative rounded-full focus-visible:outline-2 focus-visible:outline-offset-4 focus-visible:outline-brand"
    @click="$emit('click')"
    type="button"
  >
    <span
      class="grid size-[110px] place-items-center overflow-hidden rounded-full border-2 bg-gradient-to-br from-app-surface to-border-subtle shadow-2xl transition"
      :class="imageUrl ? 'border-brand' : 'border-border-subtle'"
    >
      <img v-if="imageUrl" alt="Your selected profile" class="size-full object-cover" :src="imageUrl" />
      <svg v-else aria-hidden="true" class="size-12 text-tx-muted" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.8">
        <circle cx="12" cy="8" r="4" />
        <path d="M4 21a8 8 0 0 1 16 0" />
      </svg>
    </span>
    <span class="absolute -bottom-1 -right-1 grid size-9 place-items-center rounded-full border-4 border-app-bg bg-brand text-app-bg">
      <svg aria-hidden="true" class="size-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.8">
        <path v-if="imageUrl" d="m14.5 5.5 4 4M4 20l4.5-1 10-10a2.8 2.8 0 0 0-4-4l-10 10L4 20Z" />
        <template v-else>
          <path d="M14.5 5 13 3h-2L9.5 5H6a2 2 0 0 0-2 2v10a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7a2 2 0 0 0-2-2h-3.5Z" />
          <circle cx="12" cy="12" r="3" />
        </template>
      </svg>
    </span>
  </button>
</template>

<script setup lang="ts">
defineProps<{ imageUrl?: string | null }>()
defineEmits<{ (e: 'click'): void }>()
</script>