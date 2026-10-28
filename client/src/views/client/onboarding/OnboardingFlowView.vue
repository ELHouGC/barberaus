<template>
<OnboardingLayout>
  <RouterView v-slot="{ Component }">
    <component
      :is="Component"
      v-if="Component"
      v-on="stepListeners"
    />
  </RouterView>
</OnboardingLayout>
</template>

<script setup lang="ts">
import { RouterView, useRoute, useRouter } from 'vue-router'
import { useOnboardingStore } from '@/stores/onboarding.store'
import type { CropCoordinates } from '@/transport/profile.contracts'
import OnboardingLayout from '@/components/layout/onboarding/OnboardingLayout.vue'
import { computed } from 'vue';
const onboardingStore = useOnboardingStore()
const route = useRoute()
const router = useRouter()

const emit = defineEmits<{
  (event : 'complete' ): void
}>()

const stepListeners = computed(() => {
  const currentRouteName = route.name?.toString()

  if (currentRouteName === 'onboarding-avatar') {
    return {
      next: handleNext,
      skip: handleSkip,
      'avatar-saved': handleAvatarSaved,
    }
  }

  if (currentRouteName === 'onboarding-style-dna') {
    return {
      next: handleNext,
    }
  }

  return {}
})
interface AvatarSavedPayload {
  previewUrl: string | null
  cropCoordinates: CropCoordinates | null
}

function handleAvatarSaved(payload: AvatarSavedPayload): void {
  onboardingStore.setAvatarDraft(
    payload.previewUrl,
    payload.cropCoordinates,
  )

  onboardingStore.markAvatarSaved()
}
const nextRouteByStep: Record<string, string> = {
  'onboarding-avatar': 'onboarding-style-dna',
  'onboarding-style-dna': 'onboarding-location',
  'onboarding-location': 'onboarding-barbers',
}

const skipRouteByStep: Record<string, string> = {
  'onboarding-avatar': 'onboarding-style-dna',
  'onboarding-location': 'onboarding-barbers',
}

async function handleNext(): Promise<void> {
  const currentRouteName = route.name?.toString()

  if (currentRouteName === 'onboarding-barbers') {
    emit('complete')
    return
  }

  const destination = currentRouteName
    ? nextRouteByStep[currentRouteName]
    : undefined

  if (!destination || !router.hasRoute(destination)) {
    console.warn(
      `[onboarding] Next route is not registered for "${currentRouteName ?? 'unnamed route'}".`,
    )
    return
  }

  await router.push({ name: destination })
}

async function handleSkip(): Promise<void> {
  const currentRouteName = route.name?.toString()

  if (currentRouteName === 'onboarding-avatar') {
    onboardingStore.clearAvatar()
  }

  if (currentRouteName === 'onboarding-barbers') {
    emit('complete')
    return
  }

  const destination = currentRouteName
    ? skipRouteByStep[currentRouteName]
    : undefined

  if (!destination || !router.hasRoute(destination)) {
    console.warn(
      `[onboarding] Skip route is not registered for "${currentRouteName ?? 'unnamed route'}".`,
    )
    return
  }

  await router.push({ name: destination })
}






</script>