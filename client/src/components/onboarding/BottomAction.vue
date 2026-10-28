<template>
  <div class="sticky bottom-0 mt-auto border-t border-border-subtle bg-app-bg/95 px-4 pb-[max(20px,env(safe-area-inset-bottom))] pt-4 backdrop-blur-xl">
    <button
      class="h-12 w-full rounded-xl bg-brand text-sm font-bold text-app-bg transition hover:bg-brand-hover focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand disabled:cursor-not-allowed disabled:opacity-40"
      :disabled="disabled"
      @click="$emit('click')"
      type="button"
    >
      <slot />
    </button>
  </div>
</template>

<script setup lang="ts">
defineProps<{ disabled?: boolean }>()
defineEmits<{ (e: 'click'): void }>()
</script>