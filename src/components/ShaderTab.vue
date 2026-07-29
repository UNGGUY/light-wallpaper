<script setup lang="ts">
import { ref } from 'vue'

const vertShader = ref('shader/vert.spv')
const fragShader = ref('shader/frag.spv')

const shaders = [
  { name: 'shader.frag', desc: '标准交叉淡入淡出过渡', icon: 'blend' },
  { name: 'shader_spark.frag', desc: '噪声溶解过渡 + 边缘发光效果', icon: 'spark' },
]
</script>

<template>
  <section class="tab">
    <h1 class="tab-title">着色器设置</h1>

    <p class="intro">
      着色器决定壁纸切换时的过渡动画。编辑 <code>.frag</code> / <code>.vert</code> 后用 <code>glslc</code> 编译为 SPIR-V 即可生效。
    </p>

    <div class="cols-2">
      <div class="card">
        <div class="card-label">顶点着色器</div>
        <div class="input-row">
          <input :value="vertShader" readonly class="input" />
          <button class="btn">浏览</button>
        </div>
      </div>

      <div class="card">
        <div class="card-label">片段着色器</div>
        <div class="input-row">
          <input :value="fragShader" readonly class="input" />
          <button class="btn">浏览</button>
        </div>
      </div>
    </div>

    <div class="card">
      <div class="card-label">可用的着色器</div>
      <div class="shader-list">
        <div v-for="s in shaders" :key="s.name" class="shader-row">
          <div class="shader-icon">
            <svg v-if="s.icon === 'blend'" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><circle cx="12" cy="12" r="10"/><circle cx="12" cy="12" r="10" stroke-dasharray="4 4"/></svg>
            <svg v-else width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><polygon points="12 2 22 8.5 22 15.5 12 22 2 15.5 2 8.5"/><polygon points="12 22 12 15.5"/><polygon points="22 8.5 12 15.5 2 8.5"/></svg>
          </div>
          <div class="shader-info">
            <span class="shader-name">{{ s.name }}</span>
            <span class="shader-desc">{{ s.desc }}</span>
          </div>
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
  width: 100%;
}

.cols-2 {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
  align-items: start;
}

.tab-title {
  font-size: 16px;
  font-weight: 700;
  color: var(--text);
  letter-spacing: -0.2px;
  margin-bottom: 2px;
}

.intro {
  font-size: 12.5px;
  color: var(--text-dim);
  line-height: 1.7;
}
.intro code {
  padding: 2px 6px;
  border-radius: 4px;
  background: var(--bg-card);
  border: 1px solid var(--border);
  font-size: 11px;
  color: var(--accent);
  font-family: "SF Mono", "Fira Code", "JetBrains Mono", monospace;
}

/* ── shader list ── */

.shader-list { display: flex; flex-direction: column; gap: 6px; }

.shader-row {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px;
  border-radius: var(--radius-sm);
  background: var(--bg-input);
  border: 1px solid var(--border);
  transition: border-color var(--transition);
}
.shader-row:hover { border-color: var(--border-focus); }

.shader-icon {
  width: 36px;
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 8px;
  background: var(--bg-hover);
  color: var(--accent);
  flex-shrink: 0;
}

.shader-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.shader-name {
  font-size: 12px;
  font-weight: 600;
  color: var(--text);
}

.shader-desc {
  font-size: 11px;
  color: var(--text-faint);
}
</style>
