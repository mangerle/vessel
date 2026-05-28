import { createApp } from "vue";
import { createPinia } from 'pinia'
import App from "./App.vue";
import router from './router'
import './assets/styles/theme.css'
import './assets/styles/dropdown.css'
import { attachConsole } from '@tauri-apps/plugin-log';

// 自动捕获控制台日志并发送至 Rust 端的日志记录器
attachConsole().catch((err) => {
  console.error("Failed to attach console logger", err);
});

const app = createApp(App)
app.use(createPinia())
app.use(router)
app.mount("#app");
