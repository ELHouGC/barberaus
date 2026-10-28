<template>
  <div
    class="absolute inset-0 z-40 flex items-end justify-center bg-app-bg/75 backdrop-blur-sm"
    @click="$emit('close')"
  >
    <div
      class="w-full max-w-[480px] rounded-t-3xl border border-border-subtle bg-app-surface p-4 pb-[max(24px,env(safe-area-inset-bottom))]"
      @click.stop
    >
      <div class="mx-auto mb-5 h-1 w-10 rounded-full bg-tx-muted" />
      <h2 class="text-lg font-bold text-tx-primary">Choose a photo source</h2>
      <div class="mt-4 space-y-2">
        <button
          class="flex h-12 w-full items-center gap-3 rounded-xl bg-border-subtle px-4 text-sm font-semibold text-tx-primary transition hover:bg-border-subtle/80"
          @click="$emit('select-gallery')"
          type="button"
        >
          Upload from Gallery
        </button>
        <button
          class="flex h-12 w-full items-center gap-3 rounded-xl bg-border-subtle px-4 text-sm font-semibold text-tx-primary transition hover:bg-border-subtle/80"
          @click="$emit('select-camera')"
          type="button"
        >
          Take Picture
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
defineEmits<{ (e: 'close'): void; (e: 'select-gallery'): void; (e: 'select-camera'): void }>()
</script>