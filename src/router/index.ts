import { createRouter, createWebHistory } from 'vue-router'
import MainLayout from '../layout/MainLayout.vue'

const routes = [
  {
    path: '/',
    component: MainLayout,
    children: [
      {
        path: '',
        redirect: 'containers'
      },
      {
        path: 'connections',
        name: 'connections',
        component: () => import('../views/Connections.vue')
      },
      {
        path: 'compose',
        name: 'compose',
        component: () => import('../views/Compose.vue')
      },
      {
        path: 'containers',
        name: 'containers',
        component: () => import('../views/Containers.vue')
      },
      {
        path: 'images',
        name: 'images',
        component: () => import('../views/Images.vue')
      },
      {
        path: 'networks',
        name: 'networks',
        component: () => import('../views/Networks.vue')
      },
      {
        path: 'volumes',
        name: 'volumes',
        component: () => import('../views/Volumes.vue')
      }
    ]
  }
]

const router = createRouter({
  history: createWebHistory(),
  routes
})

export default router
