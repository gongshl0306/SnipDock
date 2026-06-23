import { createApp } from 'vue'
import App from './App.vue'
import QuickApp from './QuickApp.vue'
import './styles/tokens.css'
import './styles/global.css'

// 单前端 bundle 服务两个窗口，靠 URL hash 区分：
//   主窗口（无 hash 或 #/）→ App.vue（三栏管理界面）
//   快速窗口（#quick）     → QuickApp.vue（搜索 + 复制）
// 不引入 vue-router：只有两个视图，hash 判断一行够了。
const isQuick = window.location.hash.includes('quick')

createApp(isQuick ? QuickApp : App).mount('#app')
