<script setup lang="ts">
</script>

<template>
  <section class="tab">
    <div class="hero">
      <svg class="hero-icon" viewBox="0 0 48 48" fill="none">
        <rect x="4" y="4" width="16" height="16" rx="3.5" fill="currentColor" opacity="1"/>
        <rect x="24" y="4" width="16" height="16" rx="3.5" fill="currentColor" opacity="0.65"/>
        <rect x="4" y="24" width="16" height="16" rx="3.5" fill="currentColor" opacity="0.45"/>
        <rect x="24" y="24" width="16" height="16" rx="3.5" fill="currentColor" opacity="0.22"/>
      </svg>
      <h2 class="hero-title">Light Wallpaper</h2>
      <span class="hero-ver">v0.1.0</span>
    </div>

    <p class="tagline">
      超轻量 Wayland 动态壁纸引擎<br />
      原生 Vulkan 渲染 · 着色器过渡 · 背景音乐
    </p>

    <div class="tech">
      <div v-for="item in [
        { name: 'Vulkan', role: '渲染后端' },
        { name: 'Wayland', role: '显示协议' },
        { name: 'Rust', role: '后端语言' },
        { name: 'Tauri v2', role: '桌面框架' },
        { name: 'Vue 3', role: '前端 UI' },
        { name: 'rodio', role: '音频引擎' },
      ]" :key="item.name" class="tech-cell">
        <span class="tech-name">{{ item.name }}</span>
        <span class="tech-role">{{ item.role }}</span>
      </div>
    </div>

    <p class="footnote">Unlicense — 自由使用、修改、分发</p>
  </section>
</template>

<style scoped>
.tab {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 20px;
  padding: 28px 0;
  width: 100%;
  text-align: center;
}

.hero {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
}

.hero-icon {
  width: 56px;
  height: 56px;
  color: var(--accent);
  margin-bottom: 4px;
}

.hero-title {
  font-size: 22px;
  font-weight: 800;
  color: var(--text);
  letter-spacing: -0.4px;
}

.hero-ver {
  font-size: 11px;
  padding: 2px 10px;
  border-radius: 10px;
  background: var(--bg-card);
  border: 1px solid var(--border);
  color: var(--text-faint);
}

.tagline {
  font-size: 12.5px;
  color: var(--text-dim);
  line-height: 1.8;
}

.tech {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 6px;
  width: 100%;
}

.tech-cell {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 3px;
  padding: 15px 8px;
  border-radius: var(--radius);
  background: var(--bg-card);
  border: 1px solid var(--border);
}

.tech-name {
  font-size: 12.5px;
  font-weight: 700;
  color: var(--text);
}

.tech-role {
  font-size: 10.5px;
  color: var(--text-faint);
}

.footnote {
  font-size: 11px;
  color: var(--text-faint);
}
</style>
