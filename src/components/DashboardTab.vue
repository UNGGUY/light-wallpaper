<script setup lang="ts">
import WallpaperSection from './WallpaperSection.vue'
import MusicSection from './MusicSection.vue'
</script>

<template>
  <section class="dash">
    <h1 class="dash-title">仪表盘</h1>
    <div class="dash-grid">
      <WallpaperSection />
      <MusicSection />
    </div>
  </section>
</template>

<style scoped>
.dash {
  width: 100%;
}

.dash-title {
  font-size: 17px;
  font-weight: 700;
  color: var(--text);
  letter-spacing: -0.3px;
  margin-bottom: 18px;
}

.dash-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;
  align-items: start;
}
</style>
