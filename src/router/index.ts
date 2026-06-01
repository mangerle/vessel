import { createRouter, createWebHistory } from 'vue-router'
import MainLayout from '../layout/MainLayout.vue'
import Compose from '../views/Compose.vue'
import Containers from '../views/Containers.vue'
import ContainerDetail from '../views/ContainerDetail.vue'
import Images from '../views/Images.vue'
import Networks from '../views/Networks.vue'
import Volumes from '../views/Volumes.vue'
import Settings from '../views/Settings.vue'

const routes = [
  {
    path: '/',
    component: MainLayout,
    children: [
      {
        path: '',
        redirect: 'compose'
      },
      {
        path: 'compose',
        name: 'compose',
        component: Compose
      },
      {
        path: 'containers',
        name: 'containers',
        component: Containers
      },
      {
        path: 'containers/:id',
        name: 'container-detail',
        component: ContainerDetail
      },
      {
        path: 'images',
        name: 'images',
        component: Images
      },
      {
        path: 'networks',
        name: 'networks',
        component: Networks
      },
      {
        path: 'volumes',
        name: 'volumes',
        component: Volumes
      },
      {
        path: 'settings',
        name: 'settings',
        component: Settings
      }
    ]
  }
]

const router = createRouter({
  history: createWebHistory(),
  routes
})

export default router
