<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

const isPlaying = ref(true);

function togglePlay() {
  if (isPlaying.value) {
    invoke("pause_music");
  } else {
    invoke("resume_music");
  }
  isPlaying.value = !isPlaying.value;
}

function nextTrack() {
  invoke("next_track");
}

function prevTrack() {
  invoke("prev_track");
}
</script>

<template>
  <div class="controls">
    <button @click="prevTrack()" title="上一首">⏮</button>
    <button @click="togglePlay()" :title="isPlaying ? '暂停' : '播放'">
      {{ isPlaying ? '⏸' : '▶' }}
    </button>
    <button @click="nextTrack()" title="下一首">⏭</button>
  </div>
</template>

<style scoped>
.controls {
  display: flex;
  gap: 12px;
  align-items: center;
}

.controls button {
  width: 48px;
  height: 48px;
  border: none;
  border-radius: 50%;
  background: #2a2a2a;
  color: #e0e0e0;
  font-size: 20px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background 0.2s;
}

.controls button:hover {
  background: #444;
}
</style>
