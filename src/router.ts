import { createRouter, createWebHashHistory } from "vue-router";

import HomeView from "./views/HomeView.vue";
import ContainersView from "./views/ContainersView.vue";
import RepositoriesView from "./views/RepositoriesView.vue";
import SettingsView from "./views/SettingsView.vue";

export default createRouter({
    history: createWebHashHistory(),
    routes: [
        {
            path: "/",
            component: HomeView,
        },
        {
            path: "/containers",
            component: ContainersView,
        },
        {
            path: "/repositories",
            component: RepositoriesView,
        },
        {
            path: "/settings",
            component: SettingsView,
        },
    ],
});