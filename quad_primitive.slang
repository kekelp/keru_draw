import primitives_common;

struct QuadData {
    float2 positions[4];
    float3 color;
}

[[vk::binding(1, 0)]]
StructuredBuffer<QuadData> quadBuffer;

void quadVertex(
    uint primitiveIndex,
    uint localVertexIndex,
    out float4 position,
    out float3 color
) {
    QuadData quad = quadBuffer[primitiveIndex];

    // Convert quad to triangles (0,1,2) and (0,2,3)
    uint indices[6] = {0, 1, 2, 0, 2, 3};
    uint quadVertexIndex = indices[localVertexIndex];

    position = float4(quad.positions[quadVertexIndex], 0.0, 1.0);
    color = quad.color;
}

float4 quadFragment(VertexOutput input): SV_Target {
    return float4(input.color, 1.0);
}
