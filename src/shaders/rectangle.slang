import quad;

struct RectangleData {
    float2 top_left;
    float2 size;
    Aabb clip;
    float3 color;
    float corner_radius;
}

struct Rectangles {
    StructuredBuffer<RectangleData> buffer;
}

ParameterBlock<Rectangles> rectangles;

float2 rectangle_vertex(
    uint instanceIndex,
    uint vertexIndex,
    float2 resolution,
    out float4 position
) {
    var rect: RectangleData = rectangles.buffer[instanceIndex];
    return clipped_quad_vertex(rect.top_left, rect.size, rect.clip, vertexIndex, resolution, position);
}

float sd_rounded_box(float2 p, float2 size, float r) {
    float2 q = abs(p) - size + r;
    return length(max(q, 0.0)) + min(max(q.x, q.y), 0.0) - r;
}

float4 rectangle_fragment(uint primitiveIndex, float2 uv): SV_Target {
    var rect: RectangleData = rectangles.buffer[primitiveIndex];

    float2 p = (uv - 0.5) * rect.size;
\
    float2 half_size = rect.size * 0.5;
    float dist = sd_rounded_box(p, half_size, rect.corner_radius);

    float alpha = 1.0 - smoothstep(-1.0, 1.0, dist);

    if (alpha < 0.001) {
        discard;
    }

    return float4(rect.color, alpha);
}
