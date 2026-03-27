#version 450

// 接收顶点着色器传来的纹理坐标
layout(location = 0) in vec2 fragTexCoord;

// 声明纹理采样器
layout(binding = 1) uniform sampler2D texSampler;

// 输出颜色到帧缓冲区的 Color Attachment 0
layout(location = 0) out vec4 outColor;




float cubic(float x) {
    x = abs(x);
    if (x < 1.0) return (1.5 * x - 2.5) * x * x + 1.0;
    else if (x < 2.0) return ((-0.5 * x + 2.5) * x - 4.0) * x + 2.0;
    return 0.0;
}

vec4 textureBicubic(sampler2D tex, vec2 uv, vec2 texSize) {
    uv = uv * texSize - 0.5;
    vec2 iuv = floor(uv);
    vec2 fuv = uv - iuv;

    vec4 result = vec4(0.0);
    for (int m = -1; m <= 2; m++) {
        for (int n = -1; n <= 2; n++) {
            vec2 offset = vec2(float(m), float(n));
            vec2 coord = (iuv + offset + 0.5) / texSize;
            float wx = cubic(float(m) - fuv.x);
            float wy = cubic(float(n) - fuv.y);
            result += texture(tex, coord) * wx * wy;
        }
    }
    return result;
}

void main() {
    vec2 screenSize = vec2(1493.0, 933.0);
    vec2 imageSize  = vec2(4096.0, 2304.0);

    // 保持比例缩放：取最小缩放因子
    float scale = min(screenSize.x / imageSize.x, screenSize.y / imageSize.y);

    // 把屏幕坐标映射到图像坐标
    vec2 scaledUV = fragTexCoord * screenSize / (imageSize * scale);

    // Bicubic 插值采样
    outColor = textureBicubic(texSampler, scaledUV, imageSize);
}

