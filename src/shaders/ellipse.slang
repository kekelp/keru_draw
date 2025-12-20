struct EllipseData {
    float2 top_left;
    float2 size;
    float3 color;
}

struct EllipseResources {
    StructuredBuffer<EllipseData> buffer;
}

ParameterBlock<EllipseResources> ellipseResources;

import quad;

float2 ellipse_vertex(
    uint instanceIndex,
    uint vertexIndex,
    float2 resolution,
    out float4 position
) {
    var ellipse: EllipseData = ellipseResources.buffer[instanceIndex];
    return quad_vertex(ellipse.top_left, ellipse.size, vertexIndex, resolution, position);
}

float4 ellipse_fragment(uint primitiveIndex, float2 uv): SV_Target {
    var ellipse: EllipseData = ellipseResources.buffer[primitiveIndex];

    // UV is in range [0,1], center is at (0.5, 0.5)
    // Map to [-1, 1] range
    float2 p = (uv - 0.5) * 2.0;

    float dist = length(p);
    if (dist > 1.0) {
        discard;
    }

    return float4(ellipse.color, 1.0);
}
