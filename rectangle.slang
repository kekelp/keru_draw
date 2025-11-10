struct QuadData {
    float2 top_left;
    float2 size;
    float3 color;
    float corner_radius;
    float border_width;
    float3 border_color;
}

struct RectangleVertexOutput {
    float2 uv;
    float2 size;
    float3 color;
    float corner_radius;
    float border_width;
    float3 border_color;
    float _padding;
}

struct RectangleResources {
    StructuredBuffer<QuadData> buffer;
}

ParameterBlock<RectangleResources> rectangleResources;

RectangleVertexOutput rectangle_vertex(
    uint instanceIndex,
    uint vertexIndex,
    float2 resolution,
    out float4 position
) {
    var quad: QuadData = rectangleResources.buffer[instanceIndex];

    float2 uv = float2(
        vertexIndex & 1u,
        vertexIndex >> 1u & 1u
    );

    float2 p = quad.top_left + uv * quad.size;
    float2 ndc = (p / resolution) * 2.0 - 1.0;

    position = float4(ndc.x, -ndc.y, 0.0, 1.0); // flip y

    RectangleVertexOutput output;
    output.uv = uv;
    output.size = quad.size;
    output.color = quad.color;
    output.corner_radius = quad.corner_radius;
    output.border_width = quad.border_width;
    output.border_color = quad.border_color;
    output._padding = 0.0;
    return output;
}


// SDF for a rounded rectangle
// p: point relative to center
// size: half-extents (half width, half height)
// r: corner radius
float sd_rounded_box(float2 p, float2 size, float r) {
    float2 q = abs(p) - size + r;
    return length(max(q, 0.0)) + min(max(q.x, q.y), 0.0) - r;
}

float4 rectangle_fragment(RectangleVertexOutput data): SV_Target {
    // Convert UV [0,1] to position relative to center
    float2 p = (data.uv - 0.5) * data.size;

    // Calculate SDF
    float2 half_size = data.size * 0.5;
    float dist = sd_rounded_box(p, half_size, data.corner_radius);

    // Antialiasing using derivative-based smoothstep
    float edge_softness = length(float2(ddx(dist), ddy(dist)));
    float alpha = 1.0 - smoothstep(-edge_softness, edge_softness, dist);

    // Handle border
    float3 final_color = data.color;
    if (data.border_width > 0.0) {
        float border_dist = abs(dist + data.border_width * 0.5) - data.border_width * 0.5;
        float border_alpha = 1.0 - smoothstep(-edge_softness, edge_softness, border_dist);

        // Mix border color with fill color
        final_color = lerp(data.color, data.border_color, border_alpha);
    }

    // Discard fully transparent pixels
    if (alpha < 0.001) {
        discard;
    }

    return float4(final_color, alpha);
}
