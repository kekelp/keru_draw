import quad;

struct QuadData {
    float2 top_left;
    float2 size;
    AABB clip;
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


// SDF for a rounded rectangle
// p: point relative to center
// size: half-extents (half width, half height)
// r: corner radius
float sd_rounded_box(float2 p, float2 size, float r) {
    float2 q = abs(p) - size + r;
    return length(max(q, 0.0)) + min(max(q.x, q.y), 0.0) - r;
}

float4 rectangle_fragment(uint primitiveIndex, float2 uv): SV_Target {
    var quad: QuadData = rectangleResources.buffer[primitiveIndex];

    // Convert UV [0,1] to position relative to center
    float2 p = (uv - 0.5) * quad.size;

    // Calculate SDF
    float2 half_size = quad.size * 0.5;
    float dist = sd_rounded_box(p, half_size, quad.corner_radius);

    // Antialiasing using derivative-based smoothstep
    float edge_softness = 1.0;
    float alpha = 1.0 - smoothstep(-edge_softness, edge_softness, dist);

    // Handle border
    float3 final_color = quad.color;
    if (quad.border_width > 0.0) {
        float border_dist = abs(dist + quad.border_width * 0.5) - quad.border_width * 0.5;
        float border_alpha = 1.0 - smoothstep(-edge_softness, edge_softness, border_dist);

        // Mix border color with fill color
        final_color = lerp(quad.color, quad.border_color, border_alpha);
    }

    // Discard fully transparent pixels
    if (alpha < 0.001) {
        discard;
    }

    return float4(final_color, alpha);

    // return float4(1.0, 0.0, 0.0, 1.0);

}
