<script setup lang="ts">
import {
  House,
  Box,
  Folder,
  Setting,
} from "@element-plus/icons-vue";
</script>

<template>
  <el-menu
    router
    default-active="/"
    class="sidebar"
  >
    <el-menu-item index="/">
      <el-icon>
        <House />
      </el-icon>
      <span>Home</span>
    </el-menu-item>

    <el-menu-item index="/containers">
      <el-icon>
        <Box />
      </el-icon>
      <span>Containers</span>
    </el-menu-item>

    <el-menu-item index="/repositories">
      <el-icon>
        <Folder />
      </el-icon>
      <span>Repositories</span>
    </el-menu-item>

    <el-menu-item index="/settings">
      <el-icon>
        <Setting />
      </el-icon>
      <span>Settings</span>
    </el-menu-item>
  </el-menu>
</template>

<style scoped>
.sidebar {
  height: 100%;
  border-right: none;
}
</style>