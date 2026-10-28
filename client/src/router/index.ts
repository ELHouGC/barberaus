import { useAuthStore } from '@/stores/auth'
import { createRouter, createWebHistory } from 'vue-router'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/login',
      name: 'login',
      component: () => import('@/views/auth/LoginScreen.vue'),
      meta: { requiresGuest: true }
    },
    {
      path: '/signup',
      name: 'signup',
      component: () => import('@/views/auth/SignupScreen.vue'),
      meta: { requiresGuest: true }
    },
    {
      path: '/verify',
      name: 'verify',
      component: () => import('@/views/auth/VerifyScreen.vue'),
      meta: { requiresGuest: true }
    },
    {
      path: '/clientfeed',
      name: 'clientfeed',
      component: () => import('@/views/client/ClientFeed.vue'),
      meta: { requiresAuth: true }
    },
    {
      path: '/onboarding',
      name: 'onboarding',
      component: () => import('@/views/client/onboarding/OnboardingFlowView.vue'),
      redirect: { name: 'onboarding-avatar' },
      meta: { requiresAuth: true },
      children: [
        {
          path: 'avatar',
          name: 'onboarding-avatar',
          component: () => import('@/views/client/onboarding/OnboardingAvatarView.vue'),
          children: [
            {
              path: 'camera',
              name: 'onboarding-avatar-camera',
              component: () => import('@/views/client/onboarding/OnboardingAvatarCameraView.vue'),
            },
          ],
        },
        {
          path: 'style-dna',
          name: 'onboarding-style-dna',
          component: () => import('@/views/client/onboarding/OnboardingStyleDnaView.vue'),
        },
        {
          path: 'location',
          name: 'onboarding-location',
          component: () => import('@/views/client/onboarding/OnboardingDiscoverView.vue'),
        },
      ],
    },
    {
      path: '/',
      redirect: '/clientfeed'
    }
  ]
})

// Global Navigation Guard
router.beforeEach((to, _from, next) => {
  const authStore = useAuthStore()

  if (to.meta.requiresAuth && !authStore.isAuthenticated) {
    next({ name: 'login' })
  } else if (to.meta.requiresGuest && authStore.isAuthenticated) {
    next({ name: 'clientfeed' })
  } else {
    next()
  }
})

export default router