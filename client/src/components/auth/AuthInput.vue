<script setup lang="ts">
defineProps<{
  id: string
  label: string
  modelValue: string
  type?: string
  placeholder?: string
  helperText?: string
  error?: string
}>()

defineEmits<{
  (e: 'update:modelValue', value: string): void
}>()
</script>

<template>
  <div class="w-full">
    <label :for="id" class="mb-1.5 block text-[0.875rem] font-medium text-[#FAFAFA]">
      {{ label }}
    </label>
    <div class="relative">
      <input
        :id="id"
        :type="type || 'text'"
        :value="modelValue"
        @input="$emit('update:modelValue', ($event.target as HTMLInputElement).value)"
        :placeholder="placeholder"
        class="h-11 w-full rounded-lg border bg-[#09090B] px-3 text-[0.875rem] text-[#FAFAFA] outline-none transition placeholder:text-[#52525B] focus:border-[#F59E0B] focus:ring-0"
        :class="[
          error ? 'border-red-500' : 'border-[#27272A]',
          { 'pr-10': $slots.icon }
        ]"
      />
      <div v-if="$slots.icon" class="absolute right-3 top-1/2 -translate-y-1/2">
        <slot name="icon"></slot>
      </div>
    </div>
    <p v-if="helperText && !error" class="mt-1.5 text-[0.75rem] text-[#A1A1AA]">
      {{ helperText }}
    </p>
    <p v-if="error" class="mt-1.5 text-[0.75rem] text-red-500">
      {{ error }}
    </p>
  </div>
</template>
