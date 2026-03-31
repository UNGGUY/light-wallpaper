// #version 450
//
// // 接收顶点着色器传来的纹理坐标
// layout(location = 0) in vec2 fragTexCoord;
//
// // 声明纹理采样器
// layout(binding = 1) uniform sampler2D texSampler;
//
// // 输出颜色到帧缓冲区的 Color Attachment 0
// layout(location = 0) out vec4 outColor;
//
//
// void main() {
//
//   outColor = texture(texSampler,fragTexCoord);
// }

// #version 450
//
// layout(location = 0) in vec2 fragTexCoord;
// layout(binding = 1) uniform sampler2D texSampler;
// layout(location = 0) out vec4 outColor;
//
//
// void main() {
//     outColor = textureBicubic(texSampler, fragTexCoord);
// }
//
//

#version 450

layout(location = 0) in vec2 fragTexCoord;
layout(binding = 0) uniform UniformBufferObject {
    float iTime;
    vec2 iResolution; // 屏幕分辨率
} ubo;

layout(binding = 1) uniform sampler2D texSampler;
layout(location = 0) out vec4 outColor;


// Bicubic 权重函数
float cubic(float v) {
    v = abs(v);
    if (v <= 1.0) {
        return (1.5 * v - 2.5) * v * v + 1.0;
    } else if (v < 2.0) {
        return ((-0.5 * v + 2.5) * v - 4.0) * v + 2.0;
    } else {
        return 0.0;
    }
}

// Bicubic 采样函数
vec4 textureBicubic(sampler2D tex, vec2 uv) {
    vec2 texSize = vec2(textureSize(tex, 0));
    vec2 coord = uv * texSize;
    vec2 base = floor(coord - 0.5);
    vec2 f = coord - base - 0.5;

    vec4 sum = vec4(0.0);
    float totalWeight = 0.0;

    for (int j = -1; j <= 2; j++) {
        for (int i = -1; i <= 2; i++) {
            float w = cubic(float(i) - f.x) * cubic(float(j) - f.y);
            vec2 sampleUV = (base + vec2(i, j) + 0.5) / texSize;
            sum += textureLod(tex, sampleUV, 0.0) * w;
            totalWeight += w;
        }
    }
    return sum / totalWeight;
}


void main() {
    // 屏幕宽高比
    float screenAspect = ubo.iResolution.x / ubo.iResolution.y;
    // 图片宽高比
    vec2 texSize = vec2(textureSize(texSampler, 0));
    float imageAspect = texSize.x / texSize.y;

    vec2 uv = fragTexCoord;

    if (imageAspect > screenAspect) {
        // 图片太宽，裁掉左右
        float scale = screenAspect / imageAspect;
        uv.x = uv.x * scale + (1.0 - scale) * 0.5;
    } else {
        // 图片太高，裁掉上下
        float scale = imageAspect / screenAspect;
        uv.y = uv.y * scale + (1.0 - scale) * 0.5;
    }

    outColor = textureBicubic(texSampler, uv);
}
