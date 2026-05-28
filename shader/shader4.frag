#version 450
layout(location = 0) in vec2 fragTexCoord;

layout(binding = 0) uniform UniformBufferObject {
    float iTime;
    vec2 iResolution;
} ubo;

layout(binding = 1) uniform sampler2D texSamplers[2];

layout(push_constant) uniform PushConstants {
    float progress; // 建议 CPU 端传入 0.0 ~ 1.0
} pc;

layout(location = 0) out vec4 outColor;

// ========== Value Noise 实现 (无需外部文件) ==========
float hash(vec2 p) {
    return fract(sin(dot(p, vec2(127.1, 311.7))) * 43758.5453123);
}

float valueNoise(vec2 p) {
    vec2 i = floor(p);
    vec2 f = fract(p);
    // Hermite 插值，消除网格感
    vec2 u = f * f * (3.0 - 2.0 * f);
    
    float a = hash(i + vec2(0.0, 0.0));
    float b = hash(i + vec2(1.0, 0.0));
    float c = hash(i + vec2(0.0, 1.0));
    float d = hash(i + vec2(1.0, 1.0));
    
    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

// FBM: 叠加多层噪声增加细节
float fbm(vec2 p) {
    float value = 0.0;
    float amplitude = 0.5;
    for (int i = 0; i < 4; i++) {
        value += amplitude * valueNoise(p);
        p *= 2.0;
        amplitude *= 0.5;
    }
    return value;
}
// ================================================

void main() {
    // 1. Aspect Ratio 修正 (保持你的原始逻辑)
    float screenAspect = ubo.iResolution.x / ubo.iResolution.y;
    vec2 texSize = vec2(textureSize(texSamplers[0], 0));
    float imageAspect = texSize.x / texSize.y;

    vec2 uv = fragTexCoord;
    if (imageAspect > screenAspect) {
        float scale = screenAspect / imageAspect;
        uv.x = uv.x * scale + (1.0 - scale) * 0.5;
    } else {
        float scale = imageAspect / screenAspect;
        uv.y = uv.y * scale + (1.0 - scale) * 0.5;
    }

    // 2. Progress 缓动 (让转场节奏更自然，避免线性生硬感)
    float t = pc.progress;
    t = t * t * (3.0 - 2.0 * t); // Smoothstep easing

   // 3. 噪声溶解核心逻辑
    float noiseFreq = 6.0;
    float edgeWidth = 0.06;
    
    float n = fbm(uv * noiseFreq);
    
    // t=0 → oldToNew=0 → 全旧图
    // t=1 → oldToNew=1 → 全新图
    // 高噪声区先过渡到新图（旧图被"烧蚀"）
    float oldToNew = smoothstep(t - edgeWidth, t + edgeWidth, n);
    
    // 4. 边缘发光（始终在交界面处发光，与方向无关）
    float glowMask = 1.0 - smoothstep(0.0, edgeWidth * 1.5, abs(n - t));
    vec3 glowColor = vec3(1.0, 0.7, 0.3) * glowMask * 1.5;

    // 5. 采样与混合
    vec4 oldColor = texture(texSamplers[0], uv);
    vec4 newColor = texture(texSamplers[1], uv);
    
    // oldToNew=0 → oldColor, oldToNew=1 → newColor
    outColor = mix(oldColor, newColor, oldToNew);
    outColor.rgb += glowColor;
    
}
