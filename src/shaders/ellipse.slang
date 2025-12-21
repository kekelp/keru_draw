import quad;

struct EllipseData {
    float2 top_left;
    float2 size;
    float3 color;
    float _padding;
    float2 x_clip;
    float2 y_clip;
}

struct Ellipses {
    StructuredBuffer<EllipseData> buffer;
}

ParameterBlock<Ellipses> ellipses;

void ellipse_vertex(
    uint instanceIndex,
    uint vertexIndex,
    float2 resolution,
    out float4 position,
    out float2 uv,
) {
    var ellipse: EllipseData = ellipses.buffer[instanceIndex];
    clipped_quad_vertex(ellipse.top_left, ellipse.size, ellipse.x_clip, ellipse.y_clip, vertexIndex, resolution, position, uv);
}

float4 ellipse_fragment(uint primitiveIndex, float2 uv): SV_Target {
    var ellipse: EllipseData = ellipses.buffer[primitiveIndex];

    float2 p = (uv - 0.5) * ellipse.size;

    float2 radius = ellipse.size * 0.5;
    float dist = length(p / radius) - 1.0;
    dist *= min(radius.x, radius.y);

    float alpha = 1.0 - smoothstep(-1.0, 1.0, dist);

    if (alpha < 0.001) {
        discard;
    }

    return float4(ellipse.color, alpha);
}
