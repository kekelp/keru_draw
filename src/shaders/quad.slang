// Shared quad vertex generation logic

// Generates a quad using vertex index
// vertexIndex layout:
//   0: (0, 0) - top-left
//   1: (1, 0) - top-right
//   2: (0, 1) - bottom-left
//   3: (1, 1) - bottom-right
float2 quad_vertex(
    float2 top_left,
    float2 size,
    uint vertexIndex,
    float2 resolution,
    out float4 position
) {
    float2 uv = float2(
        vertexIndex & 1u,
        vertexIndex >> 1u & 1u
    );

    float2 p = top_left + uv * size;
    float2 ndc = (p / resolution) * 2.0 - 1.0;

    position = float4(ndc.x, -ndc.y, 0.0, 1.0); // flip y

    return uv;
}
