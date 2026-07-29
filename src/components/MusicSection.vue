<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

interface MusicStatePayload {
  current_index: number
  is_playing: boolean
  mode: string
}

const audioPath = ref('~/Music/assets/bgm')
const mode = ref<'Sequential' | 'Random' | 'Single' | 'Off'>('Sequential')
const volume = ref(30)
const isPlaying = ref(true)
const currentIndex = ref(0)
const name_list = ref<string[]>([])

invoke<string[]>("name_list").then(res => { name_list.value = res })

onMounted(() => {
  listen<MusicStatePayload>('music-state-changed', (event) => {
    currentIndex.value = event.payload.current_index
    isPlaying.value = event.payload.is_playing
    mode.value = event.payload.mode as typeof mode.value
  })
})

const modes = [
  { value: 'Sequential' as const, label: '顺序', hint: '按名循环' },
  { value: 'Random' as const, label: '随机', hint: '随机播放' },
  { value: 'Single' as const, label: '单曲', hint: '单曲循环' },
  { value: 'Off' as const, label: '关闭', hint: '停止播放' },
]

function nextTrack() { invoke("next_track") }
function prevTrack() { invoke("prev_track") }
function togglePlay() { invoke(isPlaying.value ? "pause_music" : "resume_music") }
function setMode(m: typeof mode.value) { invoke("set_music_playmode", { mode: m }) }
function setVolume(v: number) { invoke("set_music_volume", { volume: v / 100 }) }
</script>

<template>
  <div class="sec">

    <!-- row 0: title -->
    <h2 class="sec-title">音乐</h2>

    <!-- row 2: dir -->
    <div class="card">
      <div class="card-label">目录</div>
      <div class="input-row">
        <input :value="audioPath" readonly class="input" />
        <button class="btn">浏览</button>
      </div>
    </div>

    <!-- row 3: mode -->
    <div class="card">
      <div class="card-label">模式</div>
      <div class="mode-grid">
        <button v-for="m in modes" :key="m.value"
          :class="['mode-option', { active: mode === m.value }]"
          @click="setMode(m.value)">
          <span class="mode-title">{{ m.label }}</span>
          <span class="mode-hint">{{ m.hint }}</span>
        </button>
      </div>
    </div>

    <!-- row 4: player -->
    <div class="card card-dark player-card">
      <div class="player-top">
        <div class="cover">
          <svg :class="{ spin: isPlaying }" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"><circle cx="12" cy="12" r="10"/><circle cx="12" cy="12" r="3"/></svg>
        </div>
        <div class="track-info">
          <span class="track-name">{{ name_list[currentIndex] || '—' }}</span>
          <span class="track-meta">第 {{ currentIndex + 1 }} / {{ name_list.length }} 首</span>
        </div>
      </div>

      <div class="player-ctrls">
        <button class="ctrl" @click="prevTrack()">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor"><path d="M6 6h2v12H6zm3.5 6 8.5 6V6z"/></svg>
        </button>
        <button class="ctrl play" @click="togglePlay()">
          <svg v-if="isPlaying" width="18" height="18" viewBox="0 0 24 24" fill="currentColor"><path d="M6 19h4V5H6v14zm8-14v14h4V5h-4z"/></svg>
          <svg v-else width="18" height="18" viewBox="0 0 24 24" fill="currentColor"><path d="M8 5v14l11-7z"/></svg>
        </button>
        <button class="ctrl" @click="nextTrack()">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor"><path d="M6 18l8.5-6L6 6v12zM16 6v12h2V6h-2z"/></svg>
        </button>
      </div>

      <div class="volume">
        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" opacity="0.3"><polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5"/></svg>
        <input type="range" v-model.number="volume" min="0" max="100" class="slider" @input="setVolume(volume)" />
        <span class="vol-val">{{ volume }}</span>
      </div>
    </div>

    <!-- row 5: playlist -->
    <div class="card card-list">
      <div class="card-label">列表 <span class="badge">{{ name_list.length }}</span></div>
      <div class="tracks" v-if="name_list.length">
        <button v-for="(t, i) in name_list" :key="t"
          :class="['track-row', { active: i === currentIndex }]">
          <span class="trix">{{ String(i + 1).padStart(2, '0') }}</span>
          <span class="tname">{{ t }}</span>
          <span v-if="i === currentIndex" class="tag">{{ isPlaying ? '播放中' : '暂停' }}</span>
        </button>
      </div>
      <div v-else class="empty">暂无</div>
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

/* ── player (dark card) ── */
.player-card { display: flex; flex-direction: column; gap: 14px; }
.player-top { display: flex; align-items: center; gap: 12px; }
.cover {
  width: 48px; height: 48px;
  border-radius: 12px;
  background: rgba(255,255,255,0.06);
  border: 1px solid rgba(255,255,255,0.1);
  display: flex; align-items: center; justify-content: center;
  flex-shrink: 0;
  color: var(--accent);
}
.cover .spin { animation: spin 4s linear infinite; }
@keyframes spin { to { transform: rotate(360deg); } }

.track-info { display: flex; flex-direction: column; gap: 2px; overflow: hidden; }
.track-name { font-size: 12.5px; font-weight: 600; color: #EAE6E0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.track-meta { font-size: 10.5px; color: rgba(255,255,255,0.3); }

.player-ctrls { display: flex; justify-content: center; align-items: center; gap: 16px; }
.ctrl {
  display: flex; align-items: center; justify-content: center;
  width: 32px; height: 32px; border: none; border-radius: 50%;
  background: none; color: rgba(255,255,255,0.4);
  cursor: pointer; transition: all var(--transition); padding: 0;
}
.ctrl:hover { color: rgba(255,255,255,0.8); }
.ctrl.play {
  width: 44px; height: 44px;
  background: var(--accent); color: #fff;
}
.ctrl.play:hover { opacity: 0.85; }

.volume { display: flex; align-items: center; gap: 10px; color: rgba(255,255,255,0.25); }
.slider {
  flex: 1; height: 3px; -webkit-appearance: none; appearance: none;
  background: rgba(255,255,255,0.12); border-radius: 2px; outline: none; cursor: pointer;
}
.slider::-webkit-slider-thumb {
  -webkit-appearance: none; width: 13px; height: 13px; border-radius: 50%;
  background: var(--accent); cursor: pointer; box-shadow: 0 0 8px var(--accent-glow);
}
.vol-val { font-size: 10.5px; font-variant-numeric: tabular-nums; width: 20px; text-align: right; }

/* ── playlist ── */
.card-list { flex: 1; display: flex; flex-direction: column; }
.tracks { max-height: 200px; overflow-y: auto; }
.track-row {
  display: flex; align-items: center; gap: 8px; width: 100%;
  padding: 5px 8px; border: none; border-radius: var(--radius-xs);
  background: none; cursor: pointer; transition: background var(--transition);
  text-align: left; font-family: var(--font);
}
.track-row:hover { background: var(--bg-hover); }
.track-row.active { background: var(--accent-light); }
.trix { font-size: 10px; color: var(--text-faint); font-variant-numeric: tabular-nums; width: 16px; }
.tname { flex: 1; font-size: 11.5px; color: var(--text); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.tag { font-size: 9.5px; padding: 1px 7px; border-radius: 8px; background: var(--green-bg); color: var(--green); flex-shrink: 0; }

.empty { text-align: center; color: var(--text-faint); font-size: 11px; padding: 16px 0; }
</style>
