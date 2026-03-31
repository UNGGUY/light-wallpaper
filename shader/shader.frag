#version 450

layout(location = 0) in vec2 fragTexCoord;
layout(binding = 0) uniform UniformBufferObject {
    float iTime;
    vec2 iResolution; // 屏幕分辨率
} ubo;

layout(binding = 1) uniform sampler2D texSampler;
layout(location = 0) out vec4 outColor;

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

    // 使用硬件 bilinear + 强制第 0 层 mipmap，避免自定义 bicubic 的柔化
    outColor = textureLod(texSampler, uv, 0.0);
}
