#version 450

// 必须与 Rust 的 location 和类型完全一致
layout(location = 0) in vec2 inPosition;
layout(location = 1) in vec2 inTexCoord; // <--- 注意这里是 vec2，不是 vec3!

layout(location = 0) out vec2 fragTexCoord; // 传递给片段着色器

void main() {
    gl_Position = vec4(inPosition, 0.0, 1.0);
    fragTexCoord = inTexCoord;
}
