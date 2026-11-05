
<template>
  <div class="flex flex-1 flex-col px-4 pb-8 pt-5">
    <OnboardingStepHeader
      step-label="Step 2 of 3"
      layout="compact"
    />

    <div class="mt-7">
      <h1 class="text-2xl font-bold tracking-tight text-tx-primary">
        Build your Style DNA
      </h1>

      <p class="mt-2 text-sm leading-6 text-tx-secondary">
        Pick at least three looks and we'll tune your feed around your taste.
      </p>
    </div>

    <!-- Category tabs -->
    <div
      class="mt-8 grid grid-cols-3 rounded-xl border border-border-subtle bg-app-surface p-1"
      role="tablist"
      aria-label="Haircut style category"
    >
      <button
        v-for="tab in categories"
        :key="tab.id"
        type="button"
        role="tab"
        :aria-selected="activeCategory === tab.id"
        :class="[
          'h-10 rounded-lg text-sm transition',
          activeCategory === tab.id
            ? 'bg-brand font-semibold text-app-bg'
            : 'font-medium text-tx-secondary hover:text-tx-primary',
        ]"
        @click="onboardingStore.setStyleCategory(tab.id)"
      >
      
        {{ tab.label }}
      </button>
    </div>

    <p class="mb-4 mt-7 text-xs font-semibold uppercase tracking-wider text-tx-muted">
      Tap the looks that feel like you
    </p>

    <!-- Inspiration gallery -->
    <div class="columns-2 gap-3">
      <button
        v-for="photo in activePhotos"
        :key="photo.id"
        type="button"
        :aria-pressed="selectedIds.includes(photo.id)"
        :class="[
          'relative mb-3 block w-full break-inside-avoid overflow-hidden rounded-2xl border-2 bg-app-surface text-left transition',
          selectedIds.includes(photo.id)
            ? 'border-brand'
            : 'border-transparent hover:border-border-subtle',
        ]"
        @click="onboardingStore.toggleStylePhoto(photo.id)"
      >
        <img
          :src="photo.src"
          :alt="photo.alt"
          class="block h-auto w-full object-cover transition duration-300"
          loading="lazy"
        />

        <span
          v-if="selectedIds.includes(photo.id)"
          class="absolute inset-0 grid place-items-center bg-black/35"
        >
          <span
            class="grid size-8 place-items-center rounded-full bg-brand font-bold text-app-bg"
            aria-hidden="true"
          >
            ✓
          </span>
        </span>
      </button>
    </div>
  </div>

  <BottomAction
    :disabled="!canContinue"
    @click="emit('next')"
  >
    {{
      canContinue
        ? 'Continue to Location'
        : `Select at least 3 styles (${selectedIds.length}/3 selected)`
    }}
  </BottomAction>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import OnboardingStepHeader from '@/components/onboarding/OnboardingStepHeader.vue'
import BottomAction from '@/components/onboarding/BottomAction.vue'
import { useOnboardingStore } from '@/stores/onboarding.store'
import type { StyleCategory } from '@/stores/onboarding.store'

const emit = defineEmits<{
  (event: 'next'): void
}>()

const onboardingStore = useOnboardingStore()

const categories: { id: StyleCategory; label: string }[] = [
  { id: 'gents', label: 'Gents' },
  { id: 'ladies', label: 'Ladies' },
  { id: 'all', label: 'All Styles' },
]

const stylePhotos = {
  gents: [
    {
      id: 'gents-textured-fade',
      src: 'https://images.unsplash.com/photo-1613772884837-87ba2082e2e3?auto=format&fit=crop&w=800&q=85',
      alt: 'Textured fade haircut',
      height: 'h-56',
    },
    {
      id: 'gents-short-crop',
      src: 'https://images.unsplash.com/photo-1578390432942-d323db577792?auto=format&fit=crop&w=800&q=85',
      alt: 'Short crop haircut',
      height: 'h-72',
    },
    {
      id: 'gents-classic-taper',
      src: 'https://images.unsplash.com/photo-1611420745411-ed8fee30b8fb?auto=format&fit=crop&w=800&q=85',
      alt: 'Classic tapered haircut',
      height: 'h-72',
    },
    {
      id: 'gents-clean-fade',
      src: 'https://images.unsplash.com/photo-1486642100029-66db6e0a97e3?auto=format&fit=crop&w=800&q=85',
      alt: 'Clean fade and beard',
      height: 'h-52',
    },
    {
      id: 'gents-modern-cut',
      src: 'https://images.unsplash.com/photo-1599351431202-1e0f0137899a?auto=format&fit=crop&w=800&q=85',
      alt: 'Modern haircut',
      height: 'h-64',
    },
    {
      id: 'gents-curly-top',
      src: 'https://images.unsplash.com/photo-1621605815971-fbc98d665033?auto=format&fit=crop&w=800&q=85',
      alt: 'Curly top haircut',
      height: 'h-72',
    },
  ],

  ladies: [
    {
      id: 'ladies-braids',
      src: 'https://images.unsplash.com/photo-1673470907547-1c0c6a996095?auto=format&fit=crop&w=800&q=85',
      alt: 'Braided hairstyle',
      height: 'h-72',
    },
    {
      id: 'ladies-blonde-bob',
      src: 'https://images.unsplash.com/photo-1648157963892-fa90a04e278b?auto=format&fit=crop&w=800&q=85',
      alt: 'Blonde bob hairstyle',
      height: 'h-52',
    },
    {
      id: 'ladies-pixie',
      src: 'https://images.unsplash.com/photo-1653656120465-ae4eaa59ed33?auto=format&fit=crop&w=800&q=85',
      alt: 'Pixie hairstyle',
      height: 'h-56',
    },
    {
      id: 'ladies-buzz',
      src: 'https://images.unsplash.com/photo-1647338064753-865bc227ebb7?auto=format&fit=crop&w=800&q=85',
      alt: 'Buzz cut hairstyle',
      height: 'h-72',
    },
    {
      id: 'ladies-red-layers',
      src: 'https://images.unsplash.com/photo-1706607599714-472e63e7f4e0?auto=format&fit=crop&w=800&q=85',
      alt: 'Red layered hairstyle',
      height: 'h-64',
    },
    {
      id: 'ladies-natural-curls',
      src: 'https://images.unsplash.com/photo-1604898419914-4d580cf22fea?auto=format&fit=crop&w=800&q=85',
      alt: 'Natural curly hairstyle',
      height: 'h-72',
    },
  ],

  all: [
    {
      id: 'all-braided',
      src: 'https://images.unsplash.com/photo-1673470907547-1c0c6a996095?auto=format&fit=crop&w=800&q=85',
      alt: 'Braided hairstyle inspiration',
      height: 'h-72',
    },
    {
      id: 'all-crop',
      src: 'https://images.unsplash.com/photo-1578390432942-d323db577792?auto=format&fit=crop&w=800&q=85',
      alt: 'Short crop hairstyle inspiration',
      height: 'h-52',
    },
    {
      id: 'all-pixie',
      src: 'https://images.unsplash.com/photo-1653656120465-ae4eaa59ed33?auto=format&fit=crop&w=800&q=85',
      alt: 'Pixie hairstyle inspiration',
      height: 'h-60',
    },
    {
      id: 'all-taper',
      src: 'https://images.unsplash.com/photo-1611420745411-ed8fee30b8fb?auto=format&fit=crop&w=800&q=85',
      alt: 'Tapered haircut inspiration',
      height: 'h-72',
    },
    {
      id: 'all-shaved',
      src: 'https://images.unsplash.com/photo-1647338064753-865bc227ebb7?auto=format&fit=crop&w=800&q=85',
      alt: 'Shaved hairstyle inspiration',
      height: 'h-64',
    },
    {
      id: 'all-long-layers',
      src: 'https://images.unsplash.com/photo-1644669236339-2aa66ff27920?auto=format&fit=crop&w=800&q=85',
      alt: 'Long layered hairstyle inspiration',
      height: 'h-72',
    },
  ],
} satisfies Record<
  StyleCategory,
  {
    id: string
    src: string
    alt: string
    height: string
  }[]
>

const activeCategory = computed(
  () => onboardingStore.selectedStyleCategory,
)

const selectedIds = computed(
  () => onboardingStore.selectedStylePhotoIds,
)

const activePhotos = computed(
  () => stylePhotos[activeCategory.value],
)

const canContinue = computed(
  () => onboardingStore.canContinueWithStyles,
)
</script>
