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

    // Generate quad corner offsets from vertex index, with flipped y
    // vertexIndex -> uv: 0->(0,1), 1->(1,1), 2->(0,0), 3->(1,0)
    float2 uv = float2(
        vertexIndex & 1u,
        1u - ((vertexIndex >> 1u) & 1u)
    );

    let resolution: float2 = float2(800.0, 600.0);
    let p: float2 = (quad.top_left + uv * quad.size) / resolution;
    let p2 = p * 2.0 - 1.0;
    let p3 = float2(p2.x, -p2.y);

    position = float4(p3, 0.0, 1.0);

    var output: RectangleVertexOutput;
    output.uv = uv;
    output.color = quad.color;
    output._padding = float2(0.0, 0.0);
    return output;
}

float4 rectangle_fragment(RectangleVertexOutput data): SV_Target {
    return float4(data.color, 1.0);
}
