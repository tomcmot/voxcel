struct VertexInput {
    [[vk::location(0)]] float3 position;
    [[vk::location(1)]] float3 uvw;
};

struct VertexOutput {
    float3 uvw;
    float4 sv_position : SV_Position;
};

[[vk::binding(0,1)]]
cbuffer Camera {
    float4x4 viewProj;
    float4x4 model;
}

[[vk::binding(0,3)]]
cbuffer TimeUniform {
    float time;
};

[[vk::binding(0,2)]]
Sampler2DArray texture;

[shader("vertex")]
VertexOutput vertMain(VertexInput vertIn) 
{
    VertexOutput output;
    output.sv_position = mul(viewProj, mul(model, float4(vertIn.position, 1.0)));
    output.uvw = vertIn.uvw;
    return output;
}

[shader("fragment")]
float4 fragMain(VertexOutput inVert) : SV_Target
{
    // float pulse = sin(time * 2.0) * 0.5 + 0.5;
    return texture.Sample(inVert.uvw);
}
