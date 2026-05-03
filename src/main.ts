import { createApp } from "vue";
import { createPinia } from 'pinia'
import App from "./App.vue";
import router from './router'
import './assets/styles/theme.css'
import './assets/styles/dropdown.css'

const app = createApp(App)
app.use(createPinia())
app.use(router)
app.mount("#app");
