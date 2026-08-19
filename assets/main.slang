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

[[vk::binding(0,3)]]
cbuffer Lighting {
    float3   ambientColor;
    float3   lightDir;
    float3   lightColor;
}

[[vk::binding(0,2)]]
Sampler2DArray texture;

[[vk::binding(1,2)]]
Sampler2D shading;

[shader("vertex")]
VertexOutput vertex(VertexInput v) 
{
    VertexOutput output;
    float4 worldPos = mul(model, float4(v.position, 1.0));
    output.sv_position = mul(viewProj, worldPos);
    output.uvw = v.uvw;
    output.world_normal = v.normal;
    return output;
}

[shader("fragment")]
float4 fragment(VertexOutput v) : SV_Target
{
    float ndotl = saturate(dot(normalize(v.world_normal), lightDir));
    float4 step = shading.Sample(float2(ndotl, 0.0));
    float4 albedo = texture.Sample(v.uvw);
    float3 lit = ambientColor + step.rgb * lightColor;
    return float4(albedo.rgb * lit, albedo.a);
}
