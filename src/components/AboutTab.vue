<script setup lang="ts">
</script>

<template>
  <section class="tab">
    <div class="hero">
      <svg class="hero-logo" viewBox="0 0 48 48" fill="none">
        <rect x="4" y="4" width="16" height="16" rx="3" fill="currentColor" opacity="1"/>
        <rect x="24" y="4" width="16" height="16" rx="3" fill="currentColor" opacity="0.7"/>
        <rect x="4" y="24" width="16" height="16" rx="3" fill="currentColor" opacity="0.5"/>
        <rect x="24" y="24" width="16" height="16" rx="3" fill="currentColor" opacity="0.25"/>
      </svg>
      <h2>Light Wallpaper</h2>
      <span class="ver">v0.1.0</span>
    </div>

    <p class="summary">
      超轻量 Wayland 动态壁纸引擎<br />
      原生 Vulkan 渲染 · 着色器过渡 · 背景音乐
    </p>

    <div class="tech-grid">
      <div class="tech-item">
        <span class="tech-name">Vulkan</span>
        <span class="tech-role">渲染后端</span>
      </div>
      <div class="tech-item">
        <span class="tech-name">Wayland</span>
        <span class="tech-role">显示协议</span>
      </div>
      <div class="tech-item">
        <span class="tech-name">Rust</span>
        <span class="tech-role">后端语言</span>
      </div>
      <div class="tech-item">
        <span class="tech-name">Tauri v2</span>
        <span class="tech-role">桌面框架</span>
      </div>
      <div class="tech-item">
        <span class="tech-name">Vue 3</span>
        <span class="tech-role">前端 UI</span>
      </div>
      <div class="tech-item">
        <span class="tech-name">rodio</span>
        <span class="tech-role">音频引擎</span>
      </div>
    </div>

    <p class="license">Unlicense — 自由使用、修改、分发</p>
  </section>
</template>

<style scoped>
.tab {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 20px;
  padding: 36px 0;
  max-width: 520px;
  margin: 0 auto;
}

.hero {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
}

.hero-logo {
  width: 56px;
  height: 56px;
  color: var(--accent);
  margin-bottom: 4px;
}

.hero h2 {
  font-size: 20px;
  font-weight: 700;
  color: var(--text-primary);
  letter-spacing: -0.3px;
}

.ver {
  font-size: 11px;
  padding: 2px 10px;
  border-radius: 10px;
  background: var(--bg-card);
  border: 1px solid var(--border);
  color: var(--text-muted);
}

.summary {
  font-size: 12px;
  color: var(--text-secondary);
  text-align: center;
  line-height: 1.8;
}

/* ── Tech Grid ── */
.tech-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 6px;
  width: 100%;
}

.tech-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2px;
  padding: 14px 8px;
  border-radius: var(--radius);
  background: var(--bg-card);
  border: 1px solid var(--border);
}

.tech-name {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-primary);
}

.tech-role {
  font-size: 10px;
  color: var(--text-muted);
}

.license {
  font-size: 11px;
  color: var(--text-muted);
}
</style>
