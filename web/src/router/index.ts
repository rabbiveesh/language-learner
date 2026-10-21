import { createRouter, createWebHistory } from 'vue-router'
import ReviewView from '../views/ReviewView.vue'

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    { path: '/', name: 'review', component: ReviewView },
    { path: '/words', name: 'words', component: () => import('../views/WordsView.vue') },
    { path: '/grammar', name: 'grammar', component: () => import('../views/GrammarView.vue') },
  ],
})

export default router
