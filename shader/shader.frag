#version 450

// 接收顶点着色器传来的纹理坐标
layout(location = 0) in vec2 fragTexCoord;

// 声明纹理采样器
layout(binding = 1) uniform sampler2D texSampler;

// 输出颜色到帧缓冲区的 Color Attachment 0
layout(location = 0) out vec4 outColor;

void main() {
    // 采样纹理
    vec4 texColor = texture(texSampler, fragTexCoord);
    
    // 输出最终颜色
    outColor = texColor;
}
