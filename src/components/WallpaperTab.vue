<script setup lang="ts">
import { ref } from 'vue'

const imagePath = ref('~/Pictures/assets/wallpapers/')
const interval = ref(15)
const mode = ref<'sequential' | 'random' | 'single'>('sequential')
const currentIndex = ref(1)

const wallpapers = ref([
  '01_sunset.jpg',
  '02_mountains.jpg',
  '03_forest.jpg',
  '04_ocean.jpg',
  '05_starry_night.jpg',
  '06_desert.jpg',
  '07_lake.jpg',
  '08_cityscape.jpg',
])

const intervals = [5, 10, 15, 30, 60, 120, 300]
const modes = [
  { value: 'sequential' as const, label: '顺序播放', desc: '按文件名顺序循环' },
  { value: 'random' as const, label: '随机播放', desc: '随机选择下一张' },
  { value: 'single' as const, label: '单张', desc: '不自动切换' },
]
</script>

<template>
  <section class="tab">
    <h2 class="section-title">壁纸设置</h2>

    <!-- 图片目录 -->
    <div class="card">
      <div class="card-label">图片目录</div>
      <div class="input-group">
        <input type="text" :value="imagePath" readonly class="input" />
        <button class="btn">浏览</button>
      </div>
    </div>

    <!-- 切换间隔 -->
    <div class="card">
      <div class="card-label">切换间隔</div>
      <div class="chip-group">
        <button
          v-for="v in intervals"
          :key="v"
          :class="['chip', { active: interval === v }]"
          @click="interval = v"
        >
          {{ v >= 60 ? v / 60 + ' 分钟' : v + ' 秒' }}
        </button>
      </div>
    </div>

    <!-- 播放模式 -->
    <div class="card">
      <div class="card-label">播放模式</div>
      <div class="mode-cards">
        <button
          v-for="m in modes"
          :key="m.value"
          :class="['mode-card', { active: mode === m.value }]"
          @click="mode = m.value"
        >
          <span class="mode-label">{{ m.label }}</span>
          <span class="mode-desc">{{ m.desc }}</span>
        </button>
      </div>
    </div>

    <!-- 壁纸列表 -->
    <div class="card">
      <div class="card-label">壁纸列表 <span class="count">{{ wallpapers.length }} 张</span></div>
      <div class="file-list">
        <div
          v-for="(wp, i) in wallpapers"
          :key="wp"
          :class="['file-item', { active: i === currentIndex }]"
        >
          <div :class="['file-thumb', { playing: i === currentIndex }]">
            <span v-if="i === currentIndex" class="playing-icon">▶</span>
          </div>
          <span class="file-name">{{ wp }}</span>
        </div>
      </div>

      <div class="player-bar">
        <button class="btn-icon" title="上一张">
          <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor"><path d="M11 3L5 8l6 5V3z"/></svg>
        </button>
        <span class="position">{{ currentIndex + 1 }} / {{ wallpapers.length }}</span>
        <button class="btn-icon" title="下一张">
          <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor"><path d="M5 3l6 5-6 5V3z"/></svg>
        </button>
      </div>
    </div>

    <button class="btn-primary">保存配置</button>
  </section>
</template>

<style scoped>
.tab {
  display: flex;
  flex-direction: column;
  gap: 14px;
  max-width: 520px;
}

.section-title {
  font-size: 15px;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 2px;
}

/* ── Card ── */
.card {
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  padding: 14px;
}

.card-label {
  font-size: 11px;
  font-weight: 600;
  color: var(--text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  margin-bottom: 10px;
}

.count {
  font-weight: 400;
  color: var(--text-muted);
  text-transform: none;
  letter-spacing: 0;
}

/* ── Input ── */
.input-group { display: flex; gap: 8px; }

.input {
  flex: 1;
  padding: 8px 12px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--bg-input);
  color: var(--text-primary);
  font-size: 12px;
  outline: none;
  transition: border-color var(--transition);
}
.input:focus { border-color: var(--accent-dim); }

/* ── Chip ── */
.chip-group { display: flex; gap: 6px; flex-wrap: wrap; }

.chip {
  padding: 5px 12px;
  border: 1px solid var(--border);
  border-radius: 20px;
  background: var(--bg-input);
  color: var(--text-secondary);
  font-size: 11px;
  cursor: pointer;
  transition: all var(--transition);
}
.chip:hover { border-color: var(--border-active); color: var(--text-primary); }
.chip.active {
  border-color: var(--accent-dim);
  background: var(--accent-glow);
  color: var(--accent);
}

/* ── Mode Cards ── */
.mode-cards { display: flex; gap: 8px; }

.mode-card {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 3px;
  padding: 12px;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--bg-input);
  cursor: pointer;
  transition: all var(--transition);
  text-align: left;
}
.mode-card:hover { border-color: var(--border-active); }
.mode-card.active {
  border-color: var(--accent-dim);
  background: var(--accent-glow);
}
.mode-label { font-size: 12px; font-weight: 600; color: var(--text-primary); }
.mode-desc { font-size: 10px; color: var(--text-muted); }
.mode-card.active .mode-desc { color: var(--text-secondary); }

/* ── File List ── */
.file-list {
  max-height: 220px;
  overflow-y: auto;
  margin-bottom: 10px;
}

.file-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 7px 8px;
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: background var(--transition);
}
.file-item:hover { background: var(--bg-hover); }
.file-item.active { background: var(--accent-glow); }

.file-thumb {
  width: 32px;
  height: 20px;
  border-radius: 3px;
  background: var(--bg-hover);
  border: 1px solid var(--border);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}
.file-thumb.playing { border-color: var(--accent-dim); }

.playing-icon {
  font-size: 8px;
  color: var(--accent);
}

.file-name {
  font-size: 12px;
  color: var(--text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* ── Player Bar ── */
.player-bar {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 14px;
  padding-top: 8px;
  border-top: 1px solid var(--border);
}

.position {
  font-size: 11px;
  color: var(--text-muted);
  font-variant-numeric: tabular-nums;
}

.btn-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 30px;
  height: 30px;
  border: 1px solid var(--border);
  border-radius: 50%;
  background: var(--bg-input);
  color: var(--text-secondary);
  cursor: pointer;
  transition: all var(--transition);
  padding: 0;
}
.btn-icon:hover {
  border-color: var(--border-active);
  color: var(--text-primary);
  background: var(--bg-hover);
}

/* ── Buttons ── */
.btn {
  padding: 7px 16px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--bg-hover);
  color: var(--text-primary);
  font-size: 12px;
  cursor: pointer;
  transition: all var(--transition);
  white-space: nowrap;
}
.btn:hover { background: var(--border-active); }

.btn-primary {
  padding: 9px 0;
  border: none;
  border-radius: var(--radius);
  background: var(--green-dim);
  color: #fff;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: opacity var(--transition);
  width: 100%;
}
.btn-primary:hover { opacity: 0.9; }
</style>
