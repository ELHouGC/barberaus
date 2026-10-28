
<script setup lang="ts">

import { useRoute } from 'vue-router'

const route = useRoute()
</script>

<template>
  <main class="min-h-screen text-tx-primary py-4"
  :class="{ 'bg-app-bg': route.name !== 'onboarding-avatar-camera'}">
    <RouterView />
  </main>
</template>