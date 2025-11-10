struct QuadData {
    float2 top_left;
    float2 size;
    float3 color;
}

struct RectangleVertexOutput {
    float2 uv;
    float3 color;
    float2 _padding;
}

struct RectangleResources {
    StructuredBuffer<QuadData> buffer;
}

ParameterBlock<RectangleResources> rectangleResources;

RectangleVertexOutput rectangle_vertex(
    uint instanceIndex,
    uint vertexIndex,
    out float4 position
) {
    QuadData quad = rectangleResources.buffer[instanceIndex];

    // Generate quad corner offsets from vertex index
    // vertexIndex: 0 -> (0,0), 1 -> (1,0), 2 -> (0,1), 3 -> (1,1)
    uint2 ucoords = uint2(
        vertexIndex & 1u,
        (vertexIndex >> 1u) & 1u
    );
    float2 uv = float2(ucoords);

    float2 resolution = float2(800.0, 600.0);
    float2 p = (quad.top_left + uv * quad.size) / resolution;
    p = p * 2.0 - 1.0;

    position = float4(p, 0.0, 1.0);

    RectangleVertexOutput output;
    output.uv = uv;
    output.color = quad.color;
    output._padding = float2(0.0, 0.0);
    return output;
}

float4 rectangle_fragment(RectangleVertexOutput data): SV_Target {
    return float4(data.color, 1.0);
}
