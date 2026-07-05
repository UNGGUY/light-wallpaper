<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";


</script>

<template><button @click="invoke('begin')">hello</button></template>

<style scoped></style>
