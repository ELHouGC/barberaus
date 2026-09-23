import { createRouter, createWebHistory } from 'vue-router'
import { useAuthStore } from '@/stores/auth'

import LoginScreen from '@/views/auth/LoginScreen.vue'
import SignupScreen from '@/views/auth/SignupScreen.vue'
import VerifyScreen from '@/views/auth/VerifyScreen.vue'
import ClientFeed from '@/views/client/ClientFeed.vue'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/login',
      name: 'login',
      component: LoginScreen,
      meta: { requiresGuest: true }
    },
    {
      path: '/signup',
      name: 'signup',
      component: SignupScreen,
      meta: { requiresGuest: true }
    },
    {
      path: '/verify',
      name: 'verify',
      component: VerifyScreen,
      meta: { requiresGuest: true }
    },
    {
      path: '/clientfeed',
      name: 'clientfeed',
      component: ClientFeed,
      meta: { requiresAuth: true }
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