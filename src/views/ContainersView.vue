<script setup lang="ts">
import { ElMessage } from "element-plus";
import { useRouter } from "vue-router";
import { deleteContainer } from "../api/docker";
import { Container } from "../api/docker";
import { getContainers } from "../api/docker";

const router = useRouter();

const tableData: Container[] = await getContainers();

function openCreateContainer() {
  router.push("/createcontainer");
}

async function deleteContainerFromTable(row: Container) {
  try {
    await deleteContainer(row.id, row.name);
    ElMessage.success("Container deleted");
  } catch (error) {
    ElMessage.error(String(error));
  }
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
    <el-table :data="tableData" style="width: 100%" max-height="250">
      <el-table-column fixed prop="name" label="Name" width="150" />
      <el-table-column prop="id" label="ID" width="120" />
      <el-table-column fixed="right" label="Operations" min-width="120">
        <template #default="scope">
          <el-button link type="primary" size="small" @click="deleteContainerFromTable(scope.row)">
            Delete
          </el-button>
        </template>
      </el-table-column>
    </el-table>
  </div>
</template>