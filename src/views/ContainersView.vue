<script setup lang="ts">
import { onMounted, ref } from "vue";
import { ElMessage } from "element-plus";
import { useRouter } from "vue-router";
import { Delete, VideoPlay, VideoPause } from '@element-plus/icons-vue';

import {
    deleteContainer,
    getContainers,
    startContainer,
    stopContainer,
} from "../api/docker";

import type { Container } from "../api/docker";

const router = useRouter();

const tableData = ref<Container[]>([]);
const loading = ref(false);

async function loadContainers() {
    loading.value = true;

    try {
        tableData.value = await getContainers();
    } catch (error) {
        ElMessage.error(`Failed to load containers: ${String(error)}`);
    } finally {
        loading.value = false;
    }
}

onMounted(() => {
    loadContainers();
});

function openCreateContainer() {
    router.push("/createcontainer");
}

async function deleteContainerFromTable(row: Container) {
    try {
        await deleteContainer(row.name);

        ElMessage.success("Container deleted");

        await loadContainers();
    } catch (error) {
        ElMessage.error(String(error));
    }
}
async function startContainerFromTable(row: Container) {
    try {
        await startContainer(row.name);

        ElMessage.success("Container deleted");

        await loadContainers();
    } catch (error) {
        ElMessage.error(String(error));
    }
}
async function stopContainerFromTable(row: Container) {
    try {
        await stopContainer(row.name);

        ElMessage.success("Container deleted");

        await loadContainers();
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

        <el-table
            :data="tableData"
            v-loading="loading"
            style="width: 100%"
            max-height="250"
        >
            <el-table-column
                fixed
                prop="name"
                label="Name"
                width="200"
            />

            <el-table-column
                prop="id"
                label="ID"
                min-width="300"
            />

            <el-table-column
                fixed="right"
                label="Operations"
                width="150"
            >
                <template #default="scope">
                    <el-button
                        link
                        type="primary"
                        :icon="VideoPlay"
                        @click="startContainerFromTable(scope.row)"
                    />
                    <el-button
                        link
                        type="warning"
                        .icon="VideoPause"
                        @click="stopContainerFromTable(scope.row)"
                    />
                    <el-button
                        link
                        type="danger"
                        :icon="Delete"
                        @click="deleteContainerFromTable(scope.row)"
                    />
                </template>
            </el-table-column>
        </el-table>
    </div>
</template>