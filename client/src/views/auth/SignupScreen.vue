<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { useForm, useField } from 'vee-validate'
import * as yup from 'yup'
import { transportApi } from '@/transport/auth.api'
import {  SIGNUP_GENDERS, type SignupRequest } from '@/transport/auth.contracts'
import { useApiError } from '@/composables/useApiError'
import AuthLayout from '@/components/layout/AuthLayout.vue'
import AuthInput from '@/components/auth/AuthInput.vue'
import AuthButton from '@/components/auth/AuthButton.vue'

const router = useRouter()
const { t } = useI18n()
const { translate, fieldKeys } = useApiError()

const loading = ref(false)
const formErrorKey = ref<string | null>(null) // non-field error from the backend
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

// 1. Yup schema. Messages are i18n KEYS, translated at render time, so they
//    follow locale changes and share one channel with backend field errors.
const signupValidationSchema = yup.object({
  full_name: yup
    .string()
    .trim()
    .required('validation.fullName.required')
    .min(2, 'validation.fullName.min'),
  email: yup
    .string()
    .trim()
    .required('validation.email.required')
    .email('validation.email.invalid'),
  date_of_birth: yup
    .string()
    .required('validation.dateOfBirth.required')
    .test('is-past', 'validation.dateOfBirth.past', (value) => {
      if (!value) return false
      return new Date(value) < new Date()
    })
    .test('min-age', 'validation.dateOfBirth.minAge', (value) => {
      if (!value) return false
      return calculateAge(value) >= 13
    }),
  gender: yup
    .string()
    .oneOf([...SIGNUP_GENDERS], 'validation.gender.invalid')
    .required('validation.gender.required'),
  password: yup
    .string()
    .required('validation.password.required')
    .min(8, 'validation.password.min'),
})

// 2. VeeValidate form engine (typed with the request contract: no `as any`)
const { handleSubmit, errors, setErrors } = useForm<SignupRequest>({
  validationSchema: signupValidationSchema,
  initialValues: {
    full_name: '',
    email: '',
    date_of_birth: '',
    gender: 'PreferNotToSay',
    password: '',
  },
})

// 3. Field bindings
const { value: fullName, handleChange: handleFullNameChange } = useField<string>('full_name')
const { value: email, handleChange: handleEmailChange } = useField<string>('email')
const { value: dateOfBirth, handleChange: handleDobChange } = useField<string>('date_of_birth')
const { value: gender, handleChange: handleGenderChange } = useField<string>('gender')
const { value: password, handleChange: handlePasswordChange } = useField<string>('password')

const onSubmit = handleSubmit(async (values) => {
  formErrorKey.value = null
  loading.value = true

  try {
    const response = await transportApi.signup(values)

    if (!response.success) {
      // Field-level problems go under their input, anything else in the banner.
      const fields = fieldKeys(response.error)
      if (Object.keys(fields).length > 0) {
        setErrors(fields)
      } else {
        formErrorKey.value = response.error.i18n_key
      }
      return
    }

    router.push({
      path: '/verify',
      query: { userId: response.data.user_id, email: values.email },
    })
  } finally {
    loading.value = false
  }
})
</script>

<template>
  <AuthLayout
    :title="t('auth.signup.title')"
    :subtitle="t('auth.signup.subtitle')"
  >
    <form @submit.prevent="onSubmit" class="space-y-4" novalidate>
      <div>
        <AuthInput
          id="fullname"
          type="text"
          :label="t('auth.fields.fullName.label')"
          :modelValue="fullName"
          @update:modelValue="handleFullNameChange"
          placeholder="John Doe"
        />
        <p v-if="errors.full_name" class="mt-1 text-xs text-red-500">
          {{ translate(errors.full_name) }}
        </p>
      </div>

      <div>
        <AuthInput
          id="dob"
          type="date"
          :label="t('auth.fields.dateOfBirth.label')"
          :modelValue="dateOfBirth"
          @update:modelValue="handleDobChange"
        />
        <p v-if="errors.date_of_birth" class="mt-1 text-xs text-red-500">
          {{ translate(errors.date_of_birth) }}
        </p>
      </div>

      <div>
        <label for="gender" class="mb-1.5 block text-[0.875rem] font-medium text-[#FAFAFA]">
          {{ t('auth.fields.gender.label') }}
        </label>
        <select
          id="gender"
          :value="gender"
          @change="handleGenderChange(($event.target as HTMLSelectElement).value)"
          class="h-11 w-full rounded-lg border border-[#27272A] bg-[#09090B] px-3 text-[0.875rem] text-[#FAFAFA] outline-none transition focus:border-[#F59E0B]"
        >
          <option v-for="option in SIGNUP_GENDERS" :key="option" :value="option">
            {{ t(`auth.fields.gender.options.${option}`) }}
          </option>
        </select>
        <p v-if="errors.gender" class="mt-1 text-xs text-red-500">
          {{ translate(errors.gender) }}
        </p>
      </div>

      <div>
        <AuthInput
          id="email"
          type="email"
          :label="t('auth.fields.email.label')"
          :modelValue="email"
          @update:modelValue="handleEmailChange"
          placeholder="name@example.com"
        />
        <p v-if="errors.email" class="mt-1 text-xs text-red-500">
          {{ translate(errors.email) }}
        </p>
      </div>

      <div>
        <AuthInput
          id="password"
          :type="showPassword ? 'text' : 'password'"
          :label="t('auth.fields.password.label')"
          :modelValue="password"
          @update:modelValue="handlePasswordChange"
          placeholder="********"
        />
        <p v-if="errors.password" class="mt-1 text-xs text-red-500">
          {{ translate(errors.password) }}
        </p>
      </div>

      <p v-if="formErrorKey" class="text-xs text-red-500">
        {{ translate(formErrorKey) }}
      </p>

      <AuthButton type="submit" :isLoading="loading">
        {{ t('auth.signup.submit') }}
      </AuthButton>
    </form>

    <template #footer>
      <i18n-t keypath="auth.signup.haveAccount" tag="div" class="mb-4 text-tx-secondary">
        <template #link>
          <button
            type="button"
            @click="router.push('/login')"
            class="font-semibold text-brand hover:underline"
          >
            {{ t('auth.signup.signInLink') }}
          </button>
        </template>
      </i18n-t>
    </template>
  </AuthLayout>
</template>