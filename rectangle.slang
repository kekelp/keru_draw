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
    var quad: QuadData = rectangleResources.buffer[instanceIndex];

    float2 uv = float2(
        vertexIndex & 1u,
        vertexIndex >> 1u & 1u
    );

    float2 resolution = float2(800.0, 600.0);

    float2 p = quad.top_left + uv * quad.size;
    float2 ndc = (p / resolution) * 2.0 - 1.0;

    position = float4(ndc.x, -ndc.y, 0.0, 1.0); // flip y

    RectangleVertexOutput output;
    output.uv = uv;
    output.color = quad.color;
    output._padding = float2(0.0, 0.0);
    return output;
}


float4 rectangle_fragment(RectangleVertexOutput data): SV_Target {
    return float4(data.color, 1.0);
}
