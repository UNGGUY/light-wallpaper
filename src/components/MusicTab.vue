<script setup lang="ts">
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'


const audioPath = ref('~/Music/assets/bgm')
const mode = ref<'sequential' | 'random' | 'single' | 'off'>('sequential')
const volume = ref(75)
const isPlaying = ref(true)
const currentIndex = ref(1)

const tracks = ref([
  '01_peaceful_piano.mp3',
  '02_ambient_dreams.mp3',
  '03_lofi_chill.mp3',
  '04_nature_sounds.mp3',
  '05_jazz_cafe.mp3',
])

const modes = [
  { value: 'sequential' as const, label: '顺序', desc: '按名循环' },
  { value: 'random' as const, label: '随机', desc: '随机播放' },
  { value: 'single' as const, label: '单曲', desc: '单曲循环' },
  { value: 'off' as const, label: '关闭', desc: '停止播放' },
]

function nextTrack() {
  invoke("next_track")
}
function prevTrack() {
  invoke("prev_track")
}

function togglePlay() {
  if (isPlaying.value) {
    invoke("pause_music")
  } else {
    invoke("resume_music")
  }
  isPlaying.value = !isPlaying.value
}
</script>

<template>
<section class="tab">
  <h2 class="section-title">音乐设置</h2>

  <!-- 音频目录 -->
  <div class="card">
    <div class="card-label">音频目录</div>
    <div class="input-group">
      <input type="text" :value="audioPath" readonly class="input" />
      <button class="btn">浏览</button>
    </div>
  </div>

  <!-- 播放模式 -->
  <div class="card">
    <div class="card-label">播放模式</div>
    <div class="mode-cards">
      <button v-for="m in modes" :key="m.value" :class="['mode-card', { active: mode === m.value }]"
        @click="mode = m.value">
        <span class="mode-label">{{ m.label }}</span>
        <span class="mode-desc">{{ m.desc }}</span>
      </button>
    </div>
  </div>

  <!-- 播放控制 & 音量 -->
  <div class="card player-card">
    <div class="now-playing">
      <div class="cover">
        <span :class="['cover-icon', { spin: isPlaying }]">🎵</span>
      </div>
      <div class="track-info">
        <span class="track-name">{{ tracks[currentIndex] }}</span>
        <span class="track-index">第 {{ currentIndex + 1 }} / {{ tracks.length }} 首</span>
      </div>
    </div>

    <div class="controls">
      <button class="ctrl-btn" title="上一首" @click="nextTrack()">
        <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
          <path d="M6 6h2v12H6zm3.5 6l8.5 6V6z" />
        </svg>
      </button>
      <button class="ctrl-btn play" @click="togglePlay()">
        <svg v-if="isPlaying" width="22" height="22" viewBox="0 0 24 24" fill="currentColor">
          <path d="M6 19h4V5H6v14zm8-14v14h4V5h-4z" />
        </svg>
        <svg v-else width="22" height="22" viewBox="0 0 24 24" fill="currentColor">
          <path d="M8 5v14l11-7z" />
        </svg>
      </button>
      <button class="ctrl-btn" title="下一首" @click="prevTrack()">
        <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
          <path d="M6 18l8.5-6L6 6v12zM16 6v12h2V6h-2z" />
        </svg>
      </button>
    </div>

    <div class="volume-row">
      <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor" opacity="0.4">
        <path d="M3 9v6h4l5 5V4L7 9H3zm13.5 3c0-1.77-1.02-3.29-2.5-4.03v8.05c1.48-.73 2.5-2.25 2.5-4.02z" />
      </svg>
      <input type="range" v-model.number="volume" min="0" max="100" class="slider" />
      <span class="vol-num">{{ volume }}</span>
    </div>
  </div>

  <!-- 播放列表 -->
  <div class="card">
    <div class="card-label">播放列表</div>
    <div class="track-list">
      <div v-for="(track, i) in tracks" :key="track" :class="['track-item', { active: i === currentIndex }]">
        <div class="track-num">{{ String(i + 1).padStart(2, '0') }}</div>
        <span class="track-title">{{ track }}</span>
        <span v-if="i === currentIndex" class="now-badge">播放中</span>
      </div>
    </div>
  </div>
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

/* ── Input ── */
.input-group {
  display: flex;
  gap: 8px;
}

.input {
  flex: 1;
  padding: 8px 12px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--bg-input);
  color: var(--text-primary);
  font-size: 12px;
  outline: none;
}

.input:focus {
  border-color: var(--accent-dim);
}

/* ── Mode Cards ── */
.mode-cards {
  display: flex;
  gap: 8px;
}

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

.mode-card:hover {
  border-color: var(--border-active);
}

.mode-card.active {
  border-color: var(--accent-dim);
  background: var(--accent-glow);
}

.mode-label {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-primary);
}

.mode-desc {
  font-size: 10px;
  color: var(--text-muted);
}

.mode-card.active .mode-desc {
  color: var(--text-secondary);
}

/* ── Player Card ── */
.player-card {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.now-playing {
  display: flex;
  align-items: center;
  gap: 14px;
}

.cover {
  width: 52px;
  height: 52px;
  border-radius: 10px;
  background: var(--bg-input);
  border: 1px solid var(--border);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.cover-icon {
  font-size: 22px;
}

.cover-icon.spin {
  animation: spin 3s linear infinite;
}

@keyframes spin {
  from {
    transform: rotate(0deg);
  }

  to {
    transform: rotate(360deg);
  }
}

.track-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
  overflow: hidden;
}

.track-name {
  font-size: 13px;
  font-weight: 500;
  color: var(--text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.track-index {
  font-size: 11px;
  color: var(--text-muted);
}

/* ── Controls ── */
.controls {
  display: flex;
  justify-content: center;
  align-items: center;
  gap: 20px;
}

.ctrl-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  border: none;
  border-radius: 50%;
  background: none;
  color: var(--text-secondary);
  cursor: pointer;
  transition: all var(--transition);
  padding: 0;
}

.ctrl-btn:hover {
  color: var(--text-primary);
}

.ctrl-btn.play {
  width: 48px;
  height: 48px;
  background: var(--accent-dim);
  color: #fff;
}

.ctrl-btn.play:hover {
  opacity: 0.85;
}

/* ── Volume ── */
.volume-row {
  display: flex;
  align-items: center;
  gap: 10px;
  color: var(--text-muted);
}

.slider {
  flex: 1;
  height: 3px;
  -webkit-appearance: none;
  appearance: none;
  background: var(--border);
  border-radius: 2px;
  outline: none;
  cursor: pointer;
}

.slider::-webkit-slider-thumb {
  -webkit-appearance: none;
  width: 12px;
  height: 12px;
  border-radius: 50%;
  background: var(--accent);
  cursor: pointer;
  box-shadow: 0 0 4px var(--accent-glow);
}

.vol-num {
  font-size: 11px;
  font-variant-numeric: tabular-nums;
  width: 22px;
  text-align: right;
}

/* ── Track List ── */
.track-list {
  max-height: 200px;
  overflow-y: auto;
}

.track-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 7px 8px;
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: background var(--transition);
}

.track-item:hover {
  background: var(--bg-hover);
}

.track-item.active {
  background: var(--accent-glow);
}

.track-num {
  font-size: 10px;
  color: var(--text-muted);
  font-variant-numeric: tabular-nums;
  width: 18px;
}

.track-title {
  flex: 1;
  font-size: 12px;
  color: var(--text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.now-badge {
  font-size: 10px;
  padding: 2px 8px;
  border-radius: 10px;
  background: var(--accent-glow);
  color: var(--accent);
  flex-shrink: 0;
}

/* ── Button ── */
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

.btn:hover {
  background: var(--border-active);
}
</style>
