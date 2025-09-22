import {createApp, h} from 'vue'
import {createPinia} from 'pinia'
import App from './App.vue'
import router from './router'
import './styles/global.css'

const app = createApp({render: () => h(App)})
app.use(createPinia())
app.use(router)
app.mount('#app')