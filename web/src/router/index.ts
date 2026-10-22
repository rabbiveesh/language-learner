import { createRouter, createWebHistory } from 'vue-router'

import { useSessionStore } from '@/stores/session'
import ReviewView from '../views/ReviewView.vue'

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    { path: '/', name: 'review', component: ReviewView },
    { path: '/words', name: 'words', component: () => import('../views/WordsView.vue') },
    { path: '/grammar', name: 'grammar', component: () => import('../views/GrammarView.vue') },
    { path: '/account', name: 'account', component: () => import('../views/AccountView.vue') },
    // `/setup?token=…` is the one-time link for the first passkey.
    { path: '/sign-in', alias: '/setup', name: 'sign-in', component: () => import('../views/SignInView.vue'), meta: { public: true } },
  ],
})

router.beforeEach(async (to) => {
  const session = useSessionStore()
  const status = session.status ?? (await session.load())
  if (to.meta.public || status.signed_in) return true
  return { name: 'sign-in', query: { next: to.fullPath } }
})

export default router
