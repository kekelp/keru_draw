import primitives_common;

struct TriangleData {
    float2 positions[3];
    float3 color;
}

[[vk::binding(0, 0)]]
StructuredBuffer<TriangleData> triangleBuffer;

void triangleVertex(
    uint primitiveIndex,
    uint localVertexIndex,
    out float4 position,
    out float3 color
) {
    TriangleData tri = triangleBuffer[primitiveIndex];
    position = float4(tri.positions[localVertexIndex], 0.0, 1.0);
    color = tri.color;
}

float4 triangleFragment(VertexOutput input): SV_Target {
    return float4(input.color, 1.0);
}
