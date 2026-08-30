<script setup lang="ts">
import { useRouter } from "vue-router";

const router = useRouter();

function openCreateContainer() {
  router.push("/createcontainer");
}
</script>

<template>
  <div>
    <h2>Containers</h2>

    <el-header>
      <el-button
        type="primary"
        @click="openCreateContainer"
      >
        Create Container
      </el-button>
    </el-header>
  </div>
</template>