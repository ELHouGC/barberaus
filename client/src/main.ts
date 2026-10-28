import { createApp } from 'vue'
import { createPinia } from 'pinia'
import router from './router'
import './style.css'
import App from './App.vue'
import i18n from '@/i18n'
import { useAuthStore } from './stores/auth.ts'
const app = createApp(App)
const pinia = createPinia()

app.use(pinia)
await useAuthStore().refreshSession();
app.use(router)
app.use(i18n)


app.mount('#app')