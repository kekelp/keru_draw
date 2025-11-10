struct EllipseData {
    float2 center;
    float2 size;
    float3 color;
}

struct EllipseVertexOutput {
    float2 uv;
    float3 color;
    float2 _padding;
}

[[vk::binding(1, 0)]]
StructuredBuffer<EllipseData> ellipseBuffer;

EllipseVertexOutput ellipse_vertex(
    uint instanceIndex,
    uint vertexIndex,
    out float4 position
) {
    EllipseData ellipse = ellipseBuffer[instanceIndex];

    // Generate quad vertices from vertex index
    uint2 ucoords = uint2(
        vertexIndex & 1u,
        (vertexIndex >> 1u) & 1u
    );
    float2 uv = float2(ucoords);
    float2 corner = uv * 2.0 - 1.0;  // Map [0,1] to [-1,1]

    float2 localPos = corner * ellipse.size;
    float2 worldPos = ellipse.center + localPos;

    position = float4(worldPos, 0.0, 1.0);

    EllipseVertexOutput output;
    output.uv = uv;
    output.color = ellipse.color;
    output._padding = float2(0.0, 0.0);
    return output;
}

float4 ellipse_fragment(EllipseVertexOutput data): SV_Target {
    // UV is in range [0,1], center is at (0.5, 0.5)
    // Map to [-1, 1] range
    float2 p = (data.uv - 0.5) * 2.0;

    float dist = length(p);
    if (dist > 1.0) {
        discard;
    }

    return float4(data.color, 1.0);
}
