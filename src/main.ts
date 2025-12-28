import { createApp } from 'vue'
import { createPinia } from 'pinia'
import App from './App.vue'

// 引入 highlight.js 样式
import 'highlight.js/styles/github.css'

const app = createApp(App)
const pinia = createPinia()

app.use(pinia)
app.mount('#app')