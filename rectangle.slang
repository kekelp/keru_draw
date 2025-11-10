struct QuadData {
    float2 center;
    float2 size;
    float3 color;
}

struct RectangleVertexOutput {
    float2 uv;
    float3 color;
    float2 _padding;
}

[[vk::binding(0, 0)]]
StructuredBuffer<QuadData> quadBuffer;

RectangleVertexOutput rectangle_vertex(
    uint instanceIndex,
    uint vertexIndex,
    out float4 position
) {
    QuadData quad = quadBuffer[instanceIndex];

    // Generate quad vertices from vertex index
    uint2 ucoords = uint2(
        vertexIndex & 1u,
        (vertexIndex >> 1u) & 1u
    );
    float2 uv = float2(ucoords);
    float2 corner = uv * 2.0 - 1.0;  // Map [0,1] to [-1,1]

    float2 localPos = corner * quad.size * 0.5;
    float2 worldPos = quad.center + localPos;

    position = float4(worldPos, 0.0, 1.0);

    RectangleVertexOutput output;
    output.uv = uv;
    output.color = quad.color;
    output._padding = float2(0.0, 0.0);
    return output;
}

float4 rectangle_fragment(RectangleVertexOutput data): SV_Target {
    return float4(data.color, 1.0);
}
