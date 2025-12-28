import { createRouter, createWebHistory } from 'vue-router';
import MainLayout from '@/views/MainLayout.vue';
import FirstRunSetup from '@/views/FirstRunSetup.vue';

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      name: 'main',
      component: MainLayout,
    },
    {
      path: '/setup',
      name: 'setup',
      component: FirstRunSetup,
    },
  ],
});

export default router;