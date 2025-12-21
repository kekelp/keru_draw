void clipped_quad_vertex(
    float2 top_left,
    float2 size,
    float2 x_clip,
    float2 y_clip,
    uint vertexIndex,
    float2 resolution,
    out float4 position,
    out float2 uv,
) {
    float2 uv01 = float2(
        vertexIndex & 1u,
        vertexIndex >> 1u & 1u
    );

    // Calculate original quad position
    float2 quad_min = top_left;
    float2 quad_max = top_left + size;

    // Construct clip min and max from x_clip and y_clip
    float2 clip_min = float2(x_clip.x, y_clip.x);
    float2 clip_max = float2(x_clip.y, y_clip.y);

    // Clip the quad
    float2 clipped_min = max(quad_min, clip_min);
    float2 clipped_max = min(quad_max, clip_max);

    // Calculate the clipped position for this vertex
    float2 p = clipped_min + uv01 * (clipped_max - clipped_min);

    // Convert to NDC
    float2 ndc = (p / resolution) * 2.0 - 1.0;

    position = float4(ndc.x, -ndc.y, 0.0, 1.0); // flip y
    uv = (p - quad_min) / size;
}
