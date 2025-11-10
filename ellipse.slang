struct EllipseData {
    float2 top_left;
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
    var ellipse: EllipseData = ellipseResources.buffer[instanceIndex];

    float2 uv = float2(
        vertexIndex & 1u,
        vertexIndex >> 1u & 1u
    );

    float2 resolution = float2(800.0, 600.0);

    float2 p = ellipse.top_left + uv * ellipse.size;
    float2 ndc = (p / resolution) * 2.0 - 1.0;

    position = float4(ndc.x, -ndc.y, 0.0, 1.0); // flip y

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
