<script setup lang="ts">
import { ref } from 'vue'
import WallpaperTab from './components/WallpaperTab.vue'
import MusicTab from './components/MusicTab.vue'
import ShaderTab from './components/ShaderTab.vue'
import AboutTab from './components/AboutTab.vue'

const activeTab = ref('wallpaper')

const tabs = [
  { key: 'wallpaper', label: '壁纸', icon: '🖼' },
  { key: 'music', label: '音乐', icon: '🎵' },
  { key: 'shader', label: '着色器', icon: '✨' },
  { key: 'about', label: '关于', icon: 'ℹ' },
] as const
</script>

<template>
<div class="app">
  <aside class="sidebar">
    <div class="brand">
      <svg class="logo-icon" viewBox="0 0 32 32" fill="none">
        <rect x="4" y="4" width="10" height="10" rx="2" fill="currentColor" opacity="0.9" />
        <rect x="18" y="4" width="10" height="10" rx="2" fill="currentColor" opacity="0.7" />
        <rect x="4" y="18" width="10" height="10" rx="2" fill="currentColor" opacity="0.5" />
        <rect x="18" y="18" width="10" height="10" rx="2" fill="currentColor" opacity="0.3" />
      </svg>
      <div>
        <span class="brand-name">Light Wallpaper</span>
        <span class="brand-version">v0.1.0</span>
      </div>
    </div>

    <nav class="nav">
      <button v-for="tab in tabs" :key="tab.key" :class="['nav-item', { active: activeTab === tab.key }]"
        @click="activeTab = tab.key">
        <span class="nav-icon">{{ tab.icon }}</span>
        <span class="nav-label">{{ tab.label }}</span>
      </button>
    </nav>

    <div class="sidebar-footer">
      <div class="status-dot" />
      <span>引擎运行中</span>
    </div>
  </aside>

  <main class="main">
    <WallpaperTab v-show="activeTab === 'wallpaper'" />
    <MusicTab v-show="activeTab === 'music'" />
    <ShaderTab v-show="activeTab === 'shader'" />
    <AboutTab v-show="activeTab === 'about'" />
  </main>
</div>
</template>

<style>
:root {
  --bg-deep: #0d1117;
  --bg-sidebar: #0f1419;
  --bg-card: #161b22;
  --bg-hover: #1c2333;
  --bg-input: #0d1117;
  --border: #21262d;
  --border-active: #30363d;
  --text-primary: #e6edf3;
  --text-secondary: #8b949e;
  --text-muted: #484f58;
  --accent: #58a6ff;
  --accent-dim: #1f6feb;
  --accent-glow: rgba(88, 166, 255, 0.15);
  --green: #3fb950;
  --green-dim: #238636;
  --radius: 8px;
  --radius-sm: 4px;
  --transition: 0.2s ease;
}

* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

body {
  background: var(--bg-deep);
  color: var(--text-primary);
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", sans-serif;
  font-size: 13px;
  line-height: 1.6;
  overflow: hidden;
  user-select: none;
  -webkit-font-smoothing: antialiased;
}

::-webkit-scrollbar {
  width: 4px;
}

::-webkit-scrollbar-track {
  background: transparent;
}

::-webkit-scrollbar-thumb {
  background: var(--border-active);
  border-radius: 2px;
}
</style>

<style scoped>
.app {
  display: flex;
  height: 100vh;
  width: 760px;
  margin: 0 auto;
}

/* ── Sidebar ── */
.sidebar {
  width: 170px;
  flex-shrink: 0;
  background: var(--bg-sidebar);
  border-right: 1px solid var(--border);
  display: flex;
  flex-direction: column;
  padding: 20px 12px;
}

.brand {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 0 8px 20px;
  border-bottom: 1px solid var(--border);
  margin-bottom: 16px;
}

.logo-icon {
  width: 28px;
  height: 28px;
  color: var(--accent);
  flex-shrink: 0;
}

.brand-name {
  display: block;
  font-size: 13px;
  font-weight: 600;
  color: var(--text-primary);
  line-height: 1.3;
}

.brand-version {
  font-size: 10px;
  color: var(--text-muted);
}

.nav {
  display: flex;
  flex-direction: column;
  gap: 2px;
  flex: 1;
}

.nav-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 10px;
  border: none;
  border-radius: 6px;
  background: none;
  color: var(--text-secondary);
  font-size: 13px;
  cursor: pointer;
  transition: all var(--transition);
  text-align: left;
}

.nav-item:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.nav-item.active {
  background: var(--accent-glow);
  color: var(--accent);
}

.nav-icon {
  font-size: 16px;
  width: 20px;
  text-align: center;
}

.nav-label {
  font-weight: 500;
}

.sidebar-footer {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px 10px 0;
  border-top: 1px solid var(--border);
  font-size: 11px;
  color: var(--text-muted);
}

.status-dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  background: var(--green);
  box-shadow: 0 0 6px var(--green);
}

/* ── Main ── */
.main {
  flex: 1;
  padding: 24px;
  overflow-y: auto;
}
</style>
