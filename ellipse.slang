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

struct EllipseResources {
    StructuredBuffer<EllipseData> buffer;
}

ParameterBlock<EllipseResources> ellipseResources;

EllipseVertexOutput ellipse_vertex(
    uint instanceIndex,
    uint vertexIndex,
    out float4 position
) {
    EllipseData ellipse = ellipseResources.buffer[instanceIndex];

    // Generate quad corner offsets from vertex index
    uint2 ucoords = uint2(
        vertexIndex & 1u,
        (vertexIndex >> 1u) & 1u
    );
    float2 uv = float2(ucoords);  // [0,1] range

    // Calculate offset from center: -0.5 to +0.5
    float2 offset = uv - 0.5;

    // Calculate pixel position
    float2 pixelPos = ellipse.center + offset * ellipse.size;

    // Convert from pixel coordinates to NDC (hardcoded 800x600 for now)
    float2 resolution = float2(800.0, 600.0);
    float2 ndc;
    ndc.x = (pixelPos.x / resolution.x) * 2.0 - 1.0;
    ndc.y = 1.0 - (pixelPos.y / resolution.y) * 2.0;

    position = float4(ndc, 0.0, 1.0);

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
