import { createRouter, createWebHistory } from 'vue-router'
import ContainerList from '../views/ContainerList.vue'

const routes = [
  {
    path: '/',
    name: 'ContainerList',
    component: ContainerList
  }
]

const router = createRouter({
  history: createWebHistory(),
  routes
})

export default router
