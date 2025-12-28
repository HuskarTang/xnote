<template>
  <div id="app" class="app">
    <router-view />
  </div>
</template>

<script setup lang="ts">
import { onMounted } from 'vue';
import { useAppStore } from '@/stores/app';

const appStore = useAppStore();

onMounted(async () => {
  await appStore.initialize();
});
</script>

<style lang="scss">
.app {
  height: 100vh;
  overflow: hidden;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
}
</style>