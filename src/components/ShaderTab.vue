<script setup lang="ts">
import { ref } from 'vue'

const vertShader = ref('shader/vert.spv')
const fragShader = ref('shader/frag.spv')
</script>

<template>
  <section class="tab">
    <h2 class="section-title">着色器设置</h2>

    <p class="desc">
      着色器决定了壁纸切换时的过渡动画效果。<br />
      修改 .frag /.vert 文件并用 <code>glslc</code> 编译为 SPIR-V 后重新加载即可生效。
    </p>

    <div class="card">
      <div class="card-label">顶点着色器</div>
      <div class="input-group">
        <input type="text" :value="vertShader" readonly class="input" />
        <button class="btn">浏览</button>
      </div>
    </div>

    <div class="card">
      <div class="card-label">片段着色器</div>
      <div class="input-group">
        <input type="text" :value="fragShader" readonly class="input" />
        <button class="btn">浏览</button>
      </div>
    </div>

    <div class="card">
      <div class="card-label">可用着色器</div>
      <div class="shader-list">
        <div class="shader-item">
          <div class="shader-icon">⟷</div>
          <div>
            <div class="shader-name">shader.frag</div>
            <div class="shader-desc">标准交叉淡入淡出过渡</div>
          </div>
        </div>
        <div class="shader-item">
          <div class="shader-icon">✦</div>
          <div>
            <div class="shader-name">shader_spark.frag</div>
            <div class="shader-desc">噪声溶解过渡 + 边缘发光效果</div>
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
  max-width: 520px;
}

.section-title {
  font-size: 15px;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 2px;
}

.desc {
  font-size: 12px;
  color: var(--text-secondary);
  line-height: 1.7;
}
.desc code {
  padding: 2px 6px;
  border-radius: 3px;
  background: var(--bg-card);
  border: 1px solid var(--border);
  font-size: 11px;
  color: var(--accent);
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
}
.input:focus { border-color: var(--accent-dim); }

/* ── Shader List ── */
.shader-list { display: flex; flex-direction: column; gap: 6px; }

.shader-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 12px;
  border-radius: var(--radius-sm);
  background: var(--bg-input);
  border: 1px solid var(--border);
  transition: border-color var(--transition);
}
.shader-item:hover { border-color: var(--border-active); }

.shader-icon {
  font-size: 18px;
  width: 36px;
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 8px;
  background: var(--bg-hover);
  flex-shrink: 0;
}

.shader-name {
  font-size: 12px;
  font-weight: 500;
  color: var(--text-primary);
}

.shader-desc {
  font-size: 11px;
  color: var(--text-muted);
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
.btn:hover { background: var(--border-active); }
</style>
