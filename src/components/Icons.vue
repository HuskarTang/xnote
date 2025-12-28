<template>
  <component 
    :is="iconComponent" 
    :style="style" 
    :class="className" 
  />
</template>

<script setup lang="ts">
import { computed } from 'vue'
import * as ElementPlusIconsVue from '@element-plus/icons-vue'

interface Props {
  name: string;
  size?: number;
  className?: string;
}

const props = withDefaults(defineProps<Props>(), {
  size: 16,
  className: ''
})

const style = computed(() => ({
  width: `${props.size}px`,
  height: `${props.size}px`,
  display: 'inline-block',
  verticalAlign: 'middle'
}))

// 图标名称映射到 Element Plus 图标组件
const iconComponent = computed(() => {
  const iconMap: Record<string, any> = {
    'search': ElementPlusIconsVue.Search,
    'close': ElementPlusIconsVue.Close,
    'add': ElementPlusIconsVue.Plus,
    'favorite-outline': ElementPlusIconsVue.Star,
    'favorite': ElementPlusIconsVue.StarFilled,
    'attachment': ElementPlusIconsVue.Paperclip,
    'tag': ElementPlusIconsVue.PriceTag,
    'tags': ElementPlusIconsVue.CollectionTag,
    'view': ElementPlusIconsVue.View,
    'edit': ElementPlusIconsVue.Edit,
    'split': ElementPlusIconsVue.Grid, // 使用网格图标代表分屏
    'delete': ElementPlusIconsVue.Delete,
    'permanent-delete': ElementPlusIconsVue.DeleteFilled,
    'restore': ElementPlusIconsVue.RefreshLeft,
    'export': ElementPlusIconsVue.Download, // 使用下载图标代表导出
    'note': ElementPlusIconsVue.Document,
    'folder': ElementPlusIconsVue.Folder,
    'file': ElementPlusIconsVue.Files,
    'open': ElementPlusIconsVue.TopRight,
    'sync': ElementPlusIconsVue.Refresh,
    'bookmark': ElementPlusIconsVue.Memo
  }
  
  return iconMap[props.name] || ElementPlusIconsVue.QuestionFilled
})
</script>

<style scoped>
svg {
  fill: currentColor;
}
</style>
