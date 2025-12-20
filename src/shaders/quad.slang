struct Aabb {
    float2 min;
    float2 max;
}

float2 quad_vertex(
    float2 top_left,
    float2 size,
    Aabb clip,
    uint vertexIndex,
    float2 resolution,
    out float4 position
) {
    float2 uv = float2(
        vertexIndex & 1u,
        vertexIndex >> 1u & 1u
    );

    // Calculate original quad position
    float2 quad_min = top_left;
    float2 quad_max = top_left + size;

    // Clip the quad to the Aabb
    float2 clipped_min = max(quad_min, clip.min);
    float2 clipped_max = min(quad_max, clip.max);

    // Calculate the clipped position for this vertex
    float2 p = clipped_min + uv * (clipped_max - clipped_min);

    // Calculate UV in the original quad's space [0,1]
    float2 adjusted_uv = (p - quad_min) / size;

    // Convert to NDC
    float2 ndc = (p / resolution) * 2.0 - 1.0;

    position = float4(ndc.x, -ndc.y, 0.0, 1.0); // flip y

    return adjusted_uv;
}
