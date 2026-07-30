struct VertexInput {
    [[vk::location(0)]] float3 position;
    [[vk::location(1)]] float3 normal;
    [[vk::location(2)]] float3 uvw;
};

[[vk::binding(0,1)]]
cbuffer Camera {
    float4x4 viewProj;
    float4x4 model;
    float3x3 normal;        // e.g. 3.0 for a 3-step toon ramp
}

[[vk::binding(1,1)]]
cbuffer OutlineVert {
    float thickness;
}

[[vk::binding(0,3)]]
cbuffer OutlineFrag {
    float3 color;
}

[shader("vertex")]
float4 vertex(VertexInput v) : SV_Position
{
    float3 expanded = v.position + normalize(v.position) * thickness;
    float4 worldPos = mul(model, float4(expanded, 1.0));
    return mul(viewProj, worldPos);
}

[shader("fragment")]
float4 fragment() : SV_Target
{
    return float4(color, 1.);
}
