<template>
  <div class="mt-4 rounded-xl border border-warning-border bg-warning-bg p-3">
    <p class="text-xs leading-5 text-warning-text">
      {{ type === 'camera' 
        ? 'Camera access was denied. Upload a photo from your gallery or enable the camera in Settings.' 
        : 'Photo library access is needed to pick an avatar. Grant access in Device Settings or take a new picture.' }}
    </p>
    <button
      class="mt-2 text-xs font-bold text-tx-primary underline underline-offset-4"
      @click="openSettings"
      type="button"
    >
      Open Settings
    </button>
  </div>
</template>

<script setup lang="ts">
defineProps<{ type: 'gallery' | 'camera' }>()

function openSettings() {
  window.open('about:preferences#privacy', '_blank', 'noopener,noreferrer')
}
</script>