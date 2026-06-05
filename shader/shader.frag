#version 450
layout(location = 0) in vec2 fragTexCoord;

// 如果你的 UBO 里还有其他数据（比如分辨率），可以保留；如果没有，UBO 也可以直接删掉
layout(binding = 0) uniform UniformBufferObject{
  float iTime;
  vec2 iResolution;
}ubo;

layout(binding = 1) uniform sampler2D texSamplers[2];

layout(push_constant) uniform PushConstants {
    float progress;
} pc;

layout(location = 0) out vec4 outColor;



void main() {
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

    vec4 oldColor = texture(texSamplers[0], uv);
    vec4 newColor = texture(texSamplers[1], uv);

    outColor = mix(oldColor, newColor, pc.progress);


}
