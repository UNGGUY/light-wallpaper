<script setup lang="ts">
import { ref } from 'vue'

const imagePath = ref('~/Pictures/assets/wallpapers/')
const interval = ref(15)
const mode = ref<'sequential' | 'random' | 'single'>('sequential')
const currentIndex = ref(1)

const wallpapers = ref([
  '01_sunset.jpg', '02_mountains.jpg', '03_forest.jpg', '04_ocean.jpg',
  '05_starry_night.jpg', '06_desert.jpg', '07_lake.jpg', '08_cityscape.jpg',
])

const intervals = [5, 10, 15, 30, 60, 120, 300]
const modes = [
  { value: 'sequential' as const, label: '顺序', hint: '按文件名循环' },
  { value: 'random' as const, label: '随机', hint: '随机选择' },
  { value: 'single' as const, label: '单张', hint: '不自动切换' },
]
</script>

<template>
  <div class="sec">

    <!-- row 0: title -->
    <h2 class="sec-title">壁纸</h2>

    <!-- row 2: dir -->
    <div class="card">
      <div class="card-label">目录</div>
      <div class="input-row">
        <input :value="imagePath" readonly class="input" />
        <button class="btn">浏览</button>
      </div>
    </div>

    <!-- row 3: interval -->
    <div class="card">
      <div class="card-label">间隔</div>
      <div class="chip-group">
        <button v-for="v in intervals" :key="v"
          :class="['chip', { active: interval === v }]"
          @click="interval = v">{{ v >= 60 ? v / 60 + 'min' : v + 's' }}</button>
      </div>
    </div>

    <!-- row 4: mode -->
    <div class="card">
      <div class="card-label">模式</div>
      <div class="mode-grid">
        <button v-for="m in modes" :key="m.value"
          :class="['mode-option', { active: mode === m.value }]"
          @click="mode = m.value">
          <span class="mode-title">{{ m.label }}</span>
          <span class="mode-hint">{{ m.hint }}</span>
        </button>
      </div>
    </div>

    <!-- row 5: file list -->
    <div class="card card-list">
      <div class="card-label">列表 <span class="badge">{{ wallpapers.length }}</span></div>
      <div class="files">
        <button v-for="(wp, i) in wallpapers" :key="wp"
          :class="['file-row', { active: i === currentIndex }]">
          <span :class="['dot', { on: i === currentIndex }]">
            <svg v-if="i === currentIndex" width="7" height="7" viewBox="0 0 8 8"><polygon points="1,0 7,4 1,8" fill="currentColor"/></svg>
          </span>
          <span class="file-name">{{ wp }}</span>
        </button>
      </div>
      <div class="player-bar">
        <button class="btn-icon" title="上一张">
          <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><polygon points="19 20 9 12 19 4 19 20"/><line x1="5" y1="19" x2="5" y2="5"/></svg>
        </button>
        <span class="pos">{{ currentIndex + 1 }} / {{ wallpapers.length }}</span>
        <button class="btn-icon" title="下一张">
          <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><polygon points="5 4 15 12 5 20 5 4"/><line x1="19" y1="5" x2="19" y2="19"/></svg>
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.sec { display: flex; flex-direction: column; gap: 12px; }

.sec-title {
  font-size: 13px;
  font-weight: 700;
  color: var(--text);
  letter-spacing: -0.1px;
}

.badge {
  display: inline-block;
  margin-left: 4px;
  padding: 0 6px;
  border-radius: 8px;
  background: var(--bg-hover);
  font-size: 9.5px;
  font-weight: 600;
  color: var(--text-dim);
  vertical-align: middle;
}

/* ── file list ── */
.card-list { display: flex; flex-direction: column; }
.files { max-height: 200px; overflow-y: auto; margin-bottom: 8px; }

.file-row {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 5px 8px;
  border: none;
  border-radius: var(--radius-xs);
  background: none;
  cursor: pointer;
  transition: background var(--transition);
  font-family: var(--font);
  text-align: left;
}
.file-row:hover { background: var(--bg-hover); }
.file-row.active { background: var(--accent-light); }

.dot {
  width: 18px; height: 18px;
  border-radius: 4px;
  background: var(--bg-hover);
  border: 1px solid var(--border);
  display: flex; align-items: center; justify-content: center;
  flex-shrink: 0;
  color: var(--accent);
}
.dot.on { border-color: var(--accent); }

.file-name { font-size: 11.5px; color: var(--text); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }

/* ── player bar ── */
.player-bar {
  display: flex; align-items: center; justify-content: center; gap: 14px;
  padding-top: 8px; border-top: 1px solid var(--border);
}
.pos { font-size: 10.5px; color: var(--text-faint); font-variant-numeric: tabular-nums; }
.btn-icon {
  display: flex; align-items: center; justify-content: center;
  width: 28px; height: 28px;
  border: 1px solid var(--border); border-radius: 50%;
  background: var(--bg-input); color: var(--text-dim);
  cursor: pointer; transition: all var(--transition); padding: 0;
}
.btn-icon:hover { border-color: var(--accent); color: var(--text); }
</style>
