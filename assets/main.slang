struct VertexInput {
    [[vk::location(0)]] float3 position;
    [[vk::location(1)]] float3 normal;
    [[vk::location(2)]] float3 uvw;
};

struct VertexOutput {
    float4 sv_position : SV_Position;
    float3 world_normal : TEXCOORD0;
    float3 uvw : TEXCOORD1;
};

[[vk::binding(0,1)]]
cbuffer Camera {
    float4x4 viewProj;
    float4x4 model;
}

[[vk::binding(1,1)]]
cbuffer Chunk {
    // position of the chunk in the world * chunk size
    float3 world_position;
}

[[vk::binding(0,3)]]
cbuffer Lighting {
    float3   ambientColor;
    float3   lightDir;
    float3   lightColor;
}

[[vk::binding(1,3)]]
cbuffer Biome {
    float biome;
}

[[vk::binding(0,2)]]
Sampler2DArray texture;

[[vk::binding(1,2)]]
Sampler2D shading;

[[vk::binding(2,2)]]
Sampler2DArray mask;

float3 hue_shift(float3 color, float hue, float sat);
float biome_hue(float biome);
float biome_sat(float biome);
float3 rgb2hsv(float3 c);
float3 hsv2rgb(float3 c);

[shader("vertex")]
VertexOutput vertex(VertexInput v) 
{
    VertexOutput output;
    float4 worldPos = mul(model, float4(v.position + world_position, 1.0));
    output.sv_position = mul(viewProj, worldPos);
    output.uvw = v.uvw;
    output.world_normal = v.normal;
    return output;
}

[shader("fragment")]
float4 fragment(VertexOutput v) : SV_Target
{
    float4 mask = mask.Sample(v.uvw);
    float ndotl = saturate(dot(normalize(v.world_normal), lightDir));
    float4 step = shading.Sample(float2(ndotl, 0.0));
    float4 albedo = texture.Sample(v.uvw);
    float3 lit = ambientColor + step.rgb * lightColor;
    if (mask.a > 0) {
        float3 hue = hue_shift(albedo.rgb, biome_hue(biome), biome_sat(biome));
        albedo = float4(hue, albedo.a);
    }
    return float4(albedo.rgb * lit, albedo.a);
}

float3 hue_shift(float3 color, float hue, float sat)
{
    float3 hsv = rgb2hsv(color);
    hsv.x = hue;
    hsv.y = hsv.y * sat;
    return hsv2rgb(hsv);
}
// Converts RGB to HSV space
float3 rgb2hsv(float3 c) {
    float4 K = float4(0.0, -1.0 / 3.0, 2.0 / 3.0, -1.0);
    float4 p = lerp(float4(c.bg, K.wz), float4(c.gb, K.xy), step(c.b, c.g));
    float4 q = lerp(float4(p.xyw, c.r), float4(c.r, p.yzx), step(p.x, c.r));

    float d = q.x - min(q.w, q.y);
    float e = 1.0e-10;
    return float3(abs(q.z + (q.w - q.y) / (6.0 * d + e)), d / (q.x + e), q.x);
}

// Converts HSV back to RGB space
float3 hsv2rgb(float3 c) {
    float4 K = float4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    float3 p = abs(frac(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * lerp(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
}

float biome_hue(float biome) {
    if (biome < 2) {
        return 63./360;
    }
    if (biome < 3) {
        return 40./360;
    }
    if (biome < 4) {
        return 152./360;
    }
    if (biome < 5) {
        return 243./360;
    }
    return 101./360;
}
float biome_sat(float biome) {
    if (biome < 2) {
        return 0.1;
    }
    return 1;
}
/*
    hue 27, 40, 63, 101, 152, 243

*/