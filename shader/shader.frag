#version 450

// 接收顶点着色器传来的纹理坐标
layout(location = 0) in vec2 fragTexCoord;

// 声明纹理采样器
layout(binding = 1) uniform sampler2D texSampler;

// 输出颜色到帧缓冲区的 Color Attachment 0
layout(location = 0) out vec4 outColor;

void main() {

    vec2 screenSize = vec2(1493.0, 933.0);
    vec2 imageSize  = vec2(4096.0, 2304.0);

    float screenAspect = screenSize.x / screenSize.y;
    float imageAspect  = imageSize.x / imageSize.y;

    vec2 uv = fragTexCoord;

    if (imageAspect > screenAspect) {
        // 图像更宽，按宽度缩放
        uv.y = uv.y * (screenAspect / imageAspect);
    } else {
        // 图像更高，按高度缩放
        uv.x = uv.x * (imageAspect / screenAspect);
    }

    // 采样纹理
    vec4 texColor = texture(texSampler,uv);
    
    // 输出最终颜色
    outColor = texColor;
}

