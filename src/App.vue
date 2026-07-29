<script setup lang="ts">
import { ref, computed } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'
import DashboardTab from './components/DashboardTab.vue'
import ShaderTab from './components/ShaderTab.vue'
import AboutTab from './components/AboutTab.vue'

const activeTab = ref('dashboard')

const tabs = [
  { key: 'dashboard', label: '仪表盘' },
  { key: 'shader', label: '着色器' },
  { key: 'about', label: '关于' },
] as const

const tabMap: Record<string, any> = {
  dashboard: DashboardTab,
  shader: ShaderTab,
  about: AboutTab,
}

const tabComponent = computed(() => tabMap[activeTab.value])

const appWindow = getCurrentWindow()
</script>

<template>
  <div class="shell">
    <!-- 自定义标题栏 -->
    <header class="titlebar" data-tauri-drag-region>
      <div class="titlebar-brand">
        <svg class="brand-icon" viewBox="0 0 24 24" fill="none">
          <rect x="3" y="3" width="7" height="7" rx="1.5" fill="currentColor" opacity="0.95" />
          <rect x="13" y="3" width="7" height="7" rx="1.5" fill="currentColor" opacity="0.7" />
          <rect x="3" y="13" width="7" height="7" rx="1.5" fill="currentColor" opacity="0.5" />
          <rect x="13" y="13" width="7" height="7" rx="1.5" fill="currentColor" opacity="0.3" />
        </svg>
        <span class="brand-text">Light Wallpaper</span>
      </div>

      <nav class="titlebar-nav">
        <button
          v-for="t in tabs"
          :key="t.key"
          :class="['nav-link', { active: activeTab === t.key }]"
          @click="activeTab = t.key"
        >{{ t.label }}</button>
      </nav>

      <div class="titlebar-status">
        <span class="status-live"></span>
        运行中
      </div>

      <!-- 窗口控制按钮 -->
      <div class="win-ctrls">
        <button class="win-btn" @click="appWindow.minimize()" title="最小化">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><line x1="5" y1="12" x2="19" y2="12"/></svg>
        </button>
        <button class="win-btn" @click="appWindow.toggleMaximize()" title="最大化">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="4" y="4" width="16" height="16" rx="2"/></svg>
        </button>
        <button class="win-btn win-close" @click="appWindow.close()" title="关闭">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
        </button>
      </div>
    </header>

    <main class="body">
      <KeepAlive>
        <component :is="tabComponent" />
      </KeepAlive>
    </main>
  </div>
</template>

<style scoped>
.shell {
  display: flex;
  flex-direction: column;
  height: 100vh;
  width: 100%;
  border-radius: 10px;
  overflow: hidden;
}

/* ── titlebar ── */
.titlebar {
  display: flex;
  align-items: center;
  gap: 20px;
  height: 42px;
  padding: 0 8px 0 20px;
  flex-shrink: 0;
  background: linear-gradient(180deg, rgba(16,16,30,0.96) 0%, rgba(12,12,24,0.94) 100%);
  backdrop-filter: blur(24px);
  -webkit-backdrop-filter: blur(24px);
  border-bottom: 1px solid rgba(255,255,255,0.06);
}

.titlebar-brand {
  display: flex;
  align-items: center;
  gap: 8px;
}

.brand-icon {
  width: 18px;
  height: 18px;
  color: var(--accent);
  opacity: 0.85;
}

.brand-text {
  font-size: 12.5px;
  font-weight: 700;
  color: var(--text-dim);
  letter-spacing: -0.1px;
}

.titlebar-nav {
  display: flex;
  gap: 1px;
  flex: 1;
}

.nav-link {
  padding: 5px 14px;
  border: none;
  border-radius: var(--radius-sm);
  background: none;
  color: rgba(255,255,255,0.45);
  font-size: 12px;
  font-family: var(--font);
  font-weight: 500;
  cursor: pointer;
  transition: all var(--transition);
}
.nav-link:hover { color: #fff; background: rgba(255,255,255,0.06); }
.nav-link.active {
  color: #fff;
  background: rgba(255,255,255,0.10);
}

.titlebar-status {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 10.5px;
  color: rgba(255,255,255,0.35);
}

.status-live {
  width: 5px;
  height: 5px;
  border-radius: 50%;
  background: var(--green);
  box-shadow: 0 0 5px var(--green);
}

/* ── window controls ── */
.win-ctrls {
  display: flex;
  gap: 2px;
  flex-shrink: 0;
}

.win-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 28px;
  border: none;
  border-radius: 4px;
  background: none;
  color: rgba(255,255,255,0.35);
  cursor: pointer;
  transition: all var(--transition);
}
.win-btn:hover { background: rgba(255,255,255,0.08); color: #fff; }
.win-close:hover { background: #e04040; color: #fff; }

/* ── body ── */
.body {
  flex: 1;
  overflow-y: auto;
  padding: 24px 32px;
}
</style>
