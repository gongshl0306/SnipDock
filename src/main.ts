import { createApp } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'
import App from './App.vue'
import QuickApp from './QuickApp.vue'
import './styles/tokens.css'
import './styles/global.css'

// 单前端 bundle 服务两个窗口，靠 Tauri 窗口 label 区分：
//   "main"  → App.vue（三栏管理界面）
//   "quick" → QuickApp.vue（搜索 + 复制）
// 用 label 而不是 URL hash，因为 Tauri 的 IPC 注入要求用 WebviewUrl::App
// 加载（外部 URL 会让 invoke 失效，导致白屏）。label 是窗口创建时定的，
// 在 webview 启动前已可读。
const label = getCurrentWindow().label

createApp(label === 'quick' ? QuickApp : App).mount('#app')
