import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/tauri';
import { useRouter } from 'vue-router';
import type { AppConfig } from '@/types';

export const useAppStore = defineStore('app', () => {
  const router = useRouter();
  
  const config = ref<AppConfig | null>(null);
  const isInitialized = ref(false);
  const isLoading = ref(false);
  const error = ref<string | null>(null);

  const isFirstRun = computed(() => {
    return config.value?.data_directory === undefined || config.value?.data_directory === null;
  });

  async function initialize() {
    if (isInitialized.value) return;
    
    isLoading.value = true;
    error.value = null;
    
    try {
      // Check if this is first run
      const firstRun = await invoke<boolean>('is_first_run');
      
      if (firstRun) {
        router.push('/setup');
        return;
      }
      
      // Load configuration
      config.value = await invoke<AppConfig>('get_config');
      isInitialized.value = true;
      
      // Navigate to main view if not already there
      if (router.currentRoute.value.path === '/setup') {
        router.push('/');
      }
    } catch (err) {
      error.value = err as string;
      console.error('Failed to initialize app:', err);
    } finally {
      isLoading.value = false;
    }
  }

  async function setDataDirectory(path: string) {
    isLoading.value = true;
    error.value = null;
    
    try {
      await invoke('set_data_directory', { path });
      
      // Reload configuration
      config.value = await invoke<AppConfig>('get_config');
      isInitialized.value = true;
      
      // Navigate to main view
      router.push('/');
    } catch (err) {
      error.value = err as string;
      throw err;
    } finally {
      isLoading.value = false;
    }
  }

  async function updateConfig(newConfig: Partial<AppConfig>) {
    if (!config.value) return;
    
    const updatedConfig = { ...config.value, ...newConfig };
    
    try {
      await invoke('update_config', { newConfig: updatedConfig });
      config.value = updatedConfig;
    } catch (err) {
      error.value = err as string;
      throw err;
    }
  }

  function clearError() {
    error.value = null;
  }

  return {
    config,
    isInitialized,
    isLoading,
    error,
    isFirstRun,
    initialize,
    setDataDirectory,
    updateConfig,
    clearError,
  };
});