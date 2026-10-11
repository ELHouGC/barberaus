<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { transportApi } from '@/transport/auth.api'
import AuthLayout from '@/components/layout/AuthLayout.vue'
import AuthInput from '@/components/ui/AuthInput.vue'
import AuthButton from '@/components/ui/AuthButton.vue'
import type { SignupGender } from '@/transport/auth.contracts'
import { useForm, useField } from 'vee-validate'
import * as yup from 'yup'

const router = useRouter()

const loading = ref(false)
const errorMessage = ref<string | null>(null)
const showPassword = ref(false)

// Age calculation helper
const calculateAge = (dobString: string): number => {
  if (!dobString) return 0
  const dob = new Date(dobString)
  const today = new Date()
  let age = today.getFullYear() - dob.getFullYear()
  const monthDiff = today.getMonth() - dob.getMonth()
  if (monthDiff < 0 || (monthDiff === 0 && today.getDate() < dob.getDate())) {
    age--
  }
  return age
}

// 1. Declarative Yup Validation Schema
const signupValidationSchema = yup.object({
  full_name: yup
    .string()
    .trim()
    .required('Full name is required')
    .min(2, 'Full name must be at least 2 characters'),
  email: yup
    .string()
    .trim()
    .required('Email address is required')
    .email('Please enter a valid email address'),
  date_of_birth: yup
    .string()
    .required('Date of birth is required')
    .test('is-past', 'Date of birth must be in the past', (value) => {
      if (!value) return false
      return new Date(value) < new Date()
    })
    .test('min-age', 'You must be at least 13 years old to register', (value) => {
      if (!value) return false
      return calculateAge(value) >= 13
    }),
  gender: yup
    .string()
    .oneOf(['Male', 'Female', 'PreferNotToSay'], 'Invalid gender selection')
    .required('Gender is required'),
  password: yup
    .string()
    .required('Password is required')
    .min(8, 'Password must be at least 8 characters')
})

// 2. Initialize VeeValidate Form Engine
const { handleSubmit, errors } = useForm({
  validationSchema: signupValidationSchema,
  initialValues: {
    full_name: '',
    email: '',
    date_of_birth: '',
    gender: 'PreferNotToSay',
    password: ''
  }
})

// 3. Destructure field bindings safely using VeeValidate's helpers
const { value: fullName, handleChange: handleFullNameChange } = useField<string>('full_name')
const { value: email, handleChange: handleEmailChange } = useField<string>('email')
const { value: dateOfBirth, handleChange: handleDobChange } = useField<string>('date_of_birth')
const { value: gender, handleChange: handleGenderChange } = useField<SignupGender>('gender')
const { value: password, handleChange: handlePasswordChange } = useField<string>('password')

const onSubmit = handleSubmit(async (values) => {
  errorMessage.value = null
  loading.value = true

  try {
    const response = await transportApi.signup(values as any)
    if (!response.success) {
      throw new Error(response.error?.message || 'Failed to create account.')
    }
    
    const user_id = response.error ? null : response.data.user_id 
    router.push({ 
      path: '/verify', 
      query: { 
        email: email.value as string, 
        user_id: user_id ?? '' 
      } 
    })
  } catch (err: any) {
    errorMessage.value = err.message || 'Failed to create account. Please try again.'
  } finally {
    loading.value = false
  }
})
</script>
<template>
  <AuthLayout 
    title="Create an Account" 
    subtitle="Get started managing your shop with ClientFeed."
  >
    <form @submit.prevent="onSubmit" class="space-y-4" novalidate>
      <div>
        <AuthInput 
          id="fullname"
          type="text"
          label="Full Name" 
          :modelValue="fullName"
          @update:modelValue="handleFullNameChange"
          placeholder="John Doe" 
        />
        <p v-if="errors.full_name" class="mt-1 text-xs text-red-500">
          {{ errors.full_name }}
        </p>
      </div>



      <div>
        <AuthInput
          id="dob"
          type="date"
          label="Date of Birth"
          :modelValue="dateOfBirth"
          @update:modelValue="handleDobChange"
        
        />
        <p v-if="errors.date_of_birth" class="mt-1 text-xs text-red-500">
          {{ errors.date_of_birth }}
        </p>
      </div>

      <div>
        <label for="gender" class="mb-1.5 block text-[0.875rem] font-medium text-[#FAFAFA]">
          Gender
        </label>
        <select
          id="gender"
          :value="gender"
          @change="handleGenderChange(($event.target as HTMLSelectElement).value)"
          class="h-11 w-full rounded-lg border border-[#27272A] bg-[#09090B] px-3 text-[0.875rem] text-[#FAFAFA] outline-none transition focus:border-[#F59E0B]"
        >
          <option value="Male">Male</option>
          <option value="Female">Female</option>
          <option value="PreferNotToSay">Prefer not to say</option>
        </select>
        <p v-if="errors.gender" class="mt-1 text-xs text-red-500">
          {{ errors.gender }}
        </p>
      </div>

      <div>
        <AuthInput 
          id="email"
          type="email"
          label="Email Address" 
          :modelValue="email"
          @update:modelValue="handleEmailChange"
          placeholder="name@example.com" 
        />
        <p v-if="errors.email" class="mt-1 text-xs text-red-500">
          {{ errors.email }}
        </p>
      </div>
      
      <div>
        <AuthInput
          id="password"
          :type="showPassword ? 'text' : 'password'"
          label="Password" 
          :modelValue="password"
          @update:modelValue="handlePasswordChange"
          placeholder="********"
        />
        <p v-if="errors.password" class="mt-1 text-xs text-red-500">
          {{ errors.password }}
        </p>
      </div>

      <p v-if="errorMessage" class="text-xs text-red-500">
        {{ errorMessage }}
      </p>

      <AuthButton
        type="submit"
        :isLoading="loading"
      >
        Sign Up
      </AuthButton>
    </form>

    <template #footer>
      <div class="mb-4 text-tx-secondary">
        Already have an account? 
        <button 
          @click="router.push('/login')" 
          class="font-semibold text-brand hover:underline"
        >
          Sign in
        </button>
      </div>
    </template>
  </AuthLayout>
</template>