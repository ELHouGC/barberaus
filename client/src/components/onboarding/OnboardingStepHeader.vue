
<template>
  <header
    :class="
        layout === 'compact'
        ? 'flex h-9 items-center justify-between'
        : 'flex items-center justify-between p-4'
    "
    >       
    <span
      class="rounded-full border border-border-subtle bg-app-surface px-3 py-1.5 text-xs font-medium text-tx-muted"
    >
      {{ stepLabel }}
    </span>

    <button
      v-if="showSkip"
      class="rounded-lg px-2 py-2 text-sm font-semibold text-tx-secondary transition hover:text-tx-primary"
      type="button"
      @click="$emit('skip')"
    >
      Skip for now
    </button>
  </header>
</template>

<script setup lang="ts">
defineProps<{
  stepLabel: string
  showSkip?: boolean
  layout?: 'padded' | 'compact'
}>()

defineEmits<{
  (event: 'skip'): void
}>()
</script>
