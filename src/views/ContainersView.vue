<script setup lang="ts">
import { ref } from "vue";
import { ElMessage } from "element-plus";
import { createContainer } from "../api/docker";

const loading = ref(false);

async function handleCreateContainer() {
  loading.value = true;

  try {
    await createContainer({
      deploymentType: "sandbox",
      version: "26.0.0.0",
      country: "de",
      containerName: "bc-test",
    });

    ElMessage.success("Container created");
  } catch (error) {
    ElMessage.error(String(error));
  } finally {
    loading.value = false;
  }
}
</script>

<template>
  <div>
    <h2>Containers</h2>

    <el-button
      type="primary"
      :loading="loading"
      @click="handleCreateContainer"
    >
      Create Container
    </el-button>
  </div>
</template>