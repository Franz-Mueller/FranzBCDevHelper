<script setup lang="ts">
import AppSidebar from "./components/AppSidebar.vue";
import AppTopbar from "./components/AppTopbar.vue";
</script>

<template>
  <el-container class="app">
    <el-header class="topbar">
      <AppTopbar />
    </el-header>

    <el-container>
      <el-aside width="220px">
        <AppSidebar />
      </el-aside>

      <el-main>
        <RouterView />
      </el-main>
    </el-container>
  </el-container>
</template>

<style>
html,
body,
#app {
  margin: 0;
  width: 100%;
  height: 100%;
}

.app {
  height: 100vh;
}

.topbar {
  border-bottom: 1px solid var(--el-border-color);
}

.el-aside {
  border-right: 1px solid var(--el-border-color);
}
</style>