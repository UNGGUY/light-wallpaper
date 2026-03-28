#version 450

layout(binding = 0) uniform UniformBufferObject {
    float iTime;
    vec2 iResolution;
} ubo;

layout(location = 0) out vec4 outColor;

void main() {
    // 当前片段坐标
    vec2 F = gl_FragCoord.xy;

    // Iterator and attenuation
    float i = 0.2;
    float a;

    // Resolution for scaling and centering
    vec2 r = ubo.iResolution.xy;
    vec2 p = (F + F - r) / r.y / 0.7;

    // Diagonal vector for skewing
    vec2 d = vec2(-1.0, 1.0);

    // Blackhole center
    vec2 b = p - i * d;

    // Rotate and apply perspective
    mat2 m1 = mat2(
        1.0, 1.0,
        d.x / (0.1 + i / dot(b, b)),
        d.y / (0.1 + i / dot(b, b))
    );
    vec2 c = p * m1;

    // Rotate into spiraling coordinates
    a = dot(c, c);
    float angle = 0.5 * log(a) + ubo.iTime * i;
    mat2 m2 = mat2(cos(angle), -sin(angle),
                   sin(angle),  cos(angle));
    vec2 v = (c * m2) / i;

    // Waves cumulative total for coloring
    vec2 w = vec2(0.0);

    // Loop through waves
    for (; i++ < 9.0; w += 1.0 + sin(v)) {
        v += 0.7 * sin(v.yx * i + ubo.iTime) / i + 0.5;
    }

    // Accretion disk radius
    i = length(sin(v / 0.3) * 0.4 + c * (3.0 + d));

    // Red/blue gradient + wave coloring + accretion disk brightness
    outColor = 1.0 - exp(
        -exp(c.x * vec4(0.6, -0.4, -1.0, 0.0))
        / w.xyyx
        / (2.0 + i * i / 4.0 - i)
        / (0.5 + 1.0 / a)
        / (0.03 + abs(length(p) - 0.7))
    );
}
