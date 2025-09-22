import {createRouter, createWebHashHistory} from 'vue-router'

const routes = [
    {path: '/', name: 'home', component: () => import('@/views/Home.vue'), meta: {title: '首页', icon: 'ion:home-outline'}},
    {path: '/server', name: 'server', component: () => import('@/views/Server.vue'), meta: {title: '服务器', icon: 'fa-solid:server'}},
    {path: '/proxy', name: 'proxy', component: () => import('@/views/Proxy.vue'), meta: {title: '代理', icon: 'mdi:proxy'}},
    {path: '/versions', name: 'versions', component: () => import('@/views/Versions.vue'), meta: {title: '版本', icon: 'mdi:update'}},
    {path: '/frpc', name: 'frpc', component: () => import('@/views/Frpc.vue'), meta: {title: 'frpc', icon: 'mdi:console'}},
]

const router = createRouter({history: createWebHashHistory(), routes})
export default router