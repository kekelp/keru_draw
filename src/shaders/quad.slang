// Shared quad vertex generation logic

struct AABB {
    float2 min;
    float2 max;
}

// Generates a quad using vertex index with AABB clipping support
// vertexIndex layout:
//   0: (0, 0) - top-left
//   1: (1, 0) - top-right
//   2: (0, 1) - bottom-left
//   3: (1, 1) - bottom-right
//
// AABB clipping:
//   clip: defines the clipping rectangle in pixel space (min, max)
//   When the quad is clipped, vertex positions are adjusted and UV coordinates
//   are modified to maintain correct texture mapping
float2 quad_vertex(
    float2 top_left,
    float2 size,
    AABB clip,
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

    // Clip the quad to the AABB
    float2 clipped_min = max(quad_min, clip.min);
    float2 clipped_max = min(quad_max, clip.max);

    // Calculate the clipped position for this vertex
    float2 p = clipped_min + uv * (clipped_max - clipped_min);

    // Adjust UV coordinates based on clipping
    // Map the clipped region back to the original quad's UV space
    float2 uv_min = (clipped_min - quad_min) / size;
    float2 uv_max = (clipped_max - quad_min) / size;
    float2 adjusted_uv = uv_min + uv * (uv_max - uv_min);

    // Convert to NDC
    float2 ndc = (p / resolution) * 2.0 - 1.0;

    position = float4(ndc.x, -ndc.y, 0.0, 1.0); // flip y

    return adjusted_uv;
}
