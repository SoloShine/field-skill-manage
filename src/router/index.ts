import { createRouter, createWebHistory } from 'vue-router'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      redirect: '/global',
    },
    {
      path: '/global',
      name: 'global',
      component: () => import('@/views/GlobalView.vue'),
    },
    {
      path: '/project',
      name: 'project',
      component: () => import('@/views/ProjectListView.vue'),
    },
    {
      path: '/project/detail',
      name: 'project-detail',
      component: () => import('@/views/ProjectDetailView.vue'),
    },
    {
      path: '/settings',
      name: 'settings',
      component: () => import('@/views/SettingsView.vue'),
    },
    {
      path: '/guide',
      name: 'guide',
      component: () => import('@/views/GuideView.vue'),
    },
  ],
})

export default router
