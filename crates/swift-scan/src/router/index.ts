import { createRouter, createWebHistory } from 'vue-router'
import HomeView from '../views/HomeView.vue'

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/',
      name: 'home',
      component: HomeView
    },
    {
      path: '/transactions',
      name: 'transactions',
      component: () => import('../views/TransactionsView.vue')
    },
    {
      path: '/blocks',
      name: 'blocks',
      component: () => import('../views/BlocksView.vue')
    },
    {
      path: '/addresses',
      name: 'addresses',
      component: () => import('../views/AddressesView.vue')
    },
    {
      path: '/validators',
      name: 'validators',
      component: () => import('../views/ValidatorsView.vue')
    }
  ]
})

export default router