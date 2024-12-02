import { createRouter, createWebHashHistory } from 'vue-router'

const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    {
      path: '/',
      name: 'home',
      component: () => import('../views/HomeView.vue')
    },
    {
      path: '/create',
      name: 'create',
      component: () => import('../views/CreateWalletView.vue')
    },
    {
      path: '/import',
      name: 'import',
      component: () => import('../views/ImportWalletView.vue')
    },
    {
      path: '/wallet/:address',
      name: 'wallet-details',
      component: () => import('../views/WalletDetailsView.vue')
    },
    {
      path: '/send',
      name: 'send-transaction',
      component: () => import('../views/SendTransactionView.vue')
    }
  ]
})