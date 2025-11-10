import primitives_common;

struct QuadData {
    float2 center;
    float2 size;
    float3 color;
}

[[vk::binding(0, 0)]]
StructuredBuffer<QuadData> quadBuffer;

void quadVertex(
    uint instanceIndex,
    uint vertexIndex,
    out float4 position,
    out float2 uv,
    out float3 color
) {
    QuadData quad = quadBuffer[instanceIndex];

    // Generate quad vertices from vertex index (0-3)
    float2 corners[4] = {
        float2(-1.0, -1.0), // bottom-left
        float2( 1.0, -1.0), // bottom-right
        float2(-1.0,  1.0), // top-left
        float2( 1.0,  1.0), // top-right
    };

    float2 uvs[4] = {
        float2(0.0, 1.0),
        float2(1.0, 1.0),
        float2(0.0, 0.0),
        float2(1.0, 0.0),
    };

    float2 corner = corners[vertexIndex];
    float2 localPos = corner * quad.size * 0.5;
    float2 worldPos = quad.center + localPos;

    position = float4(worldPos, 0.0, 1.0);
    uv = uvs[vertexIndex];
    color = quad.color;
}

float4 quadFragment(VertexOutput input): SV_Target {
    return float4(input.color, 1.0);
}
