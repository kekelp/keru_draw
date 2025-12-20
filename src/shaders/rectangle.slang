import quad;

struct QuadData {
    float2 top_left;
    float2 size;
    Aabb clip;
    float3 color;
    float _padding1;
    float corner_radius;
    float border_width;
    float3 border_color;
    float _padding2;
}

struct RectangleResources {
    StructuredBuffer<QuadData> buffer;
}

ParameterBlock<RectangleResources> rectangleResources;

float2 rectangle_vertex(
    uint instanceIndex,
    uint vertexIndex,
    float2 resolution,
    out float4 position
) {
    var quad: QuadData = rectangleResources.buffer[instanceIndex];
    return quad_vertex(quad.top_left, quad.size, quad.clip, vertexIndex, resolution, position);
}

float sd_rounded_box(float2 p, float2 size, float r) {
    float2 q = abs(p) - size + r;
    return length(max(q, 0.0)) + min(max(q.x, q.y), 0.0) - r;
}

float4 rectangle_fragment(uint primitiveIndex, float2 uv): SV_Target {
    var quad: QuadData = rectangleResources.buffer[primitiveIndex];

    float2 p = (uv - 0.5) * quad.size;

    float2 half_size = quad.size * 0.5;
    float dist = sd_rounded_box(p, half_size, quad.corner_radius);

    float alpha = 1.0 - smoothstep(-1.0, 1.0, dist);

    if (alpha < 0.001) {
        discard;
    }

    return float4(quad.color, alpha);
}
