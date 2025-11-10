import primitives_common;

struct EllipseData {
    float2 center;
    float2 radii;
    float3 color;
}

[[vk::binding(1, 0)]]
StructuredBuffer<EllipseData> ellipseBuffer;

void ellipse_vertex(
    uint instanceIndex,
    uint vertexIndex,
    out float4 position,
    out float2 uv,
    out float3 color
) {
    EllipseData ellipse = ellipseBuffer[instanceIndex];

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
    float2 localPos = corner * ellipse.radii;
    float2 worldPos = ellipse.center + localPos;

    position = float4(worldPos, 0.0, 1.0);
    uv = uvs[vertexIndex];
    color = ellipse.color;
}

float4 ellipse_fragment(VertexOutput input): SV_Target {
    // UV is in range [0,1], center is at (0.5, 0.5)
    // Map to [-1, 1] range
    float2 p = (input.uv - 0.5) * 2.0;

    // Ellipse distance test
    float dist = length(p);
    if (dist > 1.0) {
        discard;
    }

    return float4(input.color, 1.0);
}
