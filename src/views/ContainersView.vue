<script setup lang="ts">
import { reactive, ref } from "vue";
import { ElMessage } from "element-plus";
import { createContainer } from "../api/docker";

const loading = ref(false);

const form = reactive({
  deploymentType: "sandbox",
  version: "",
  country: "de",
  containerName: "",
});

async function handleCreateContainer() {
  loading.value = true;

  try {
    await createContainer({
      deploymentType: form.deploymentType,
      version: form.version,
      country: form.country,
      containerName: form.containerName,
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

    <el-form label-width="140px" style="max-width: 500px">
      <el-form-item label="Container name">
        <el-input v-model="form.containerName" />
      </el-form-item>

      <el-form-item label="Version">
        <el-input
          v-model="form.version"
          placeholder="26.0.0.0"
        />
      </el-form-item>

      <el-form-item label="Country">
        <el-input
          v-model="form.country"
          placeholder="de"
        />
      </el-form-item>

      <el-form-item label="Deployment type">
        <el-select v-model="form.deploymentType">
          <el-option
            label="Sandbox"
            value="sandbox"
          />
          <el-option
            label="OnPrem"
            value="onprem"
          />
        </el-select>
      </el-form-item>

      <el-form-item>
        <el-button
          type="primary"
          :loading="loading"
          @click="handleCreateContainer"
        >
          Create Container
        </el-button>
      </el-form-item>
    </el-form>
  </div>
</template>