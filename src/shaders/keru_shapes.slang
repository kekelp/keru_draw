import quad;

struct BoxData {
    float2 top_left;
    float2 size;
    float2 x_clip;
    float2 y_clip;
    float corner_radius;
    float border_thickness;
    float2 gradient_direction; // normalized direction for linear gradient
    float4 color_start;
    float4 color_end;
    uint gradient_type; // 0=solid, 1=linear
    float pad0;
    float pad1;
    float pad2;
}

struct CircleData {
    float2 center;
    float2 radii;
    float2 angles;
    float2 x_clip;
    float2 y_clip;
    float2 gradient_direction; // normalized direction for linear gradient
    float4 color_start;
    float4 color_end;
    uint gradient_type; // 0=solid, 1=linear, 2=radial
    float _pad0;
    float _pad1;
    float _pad2;
}

struct SegmentData {
    float2 start;
    float2 end;
    float2 x_clip;
    float2 y_clip;
    float4 color_start;
    float4 color_end;
    float4 thickness_dash;
    uint gradient_type; // 0=solid, 1=linear along segment
    float pad0;
    float pad1;
    float pad2;
}

struct GridData {
    float2 top_left;
    float2 size;
    float2 x_clip;
    float2 y_clip;
    float2 offset;
    float lattice_size;
    float line_thickness;
    float4 color;
    uint grid_type; // 0=square, 1=hex
    float pad0;
    float pad1;
    float pad2;
}

struct TriangleData {
    float2 p0;
    float2 p1;
    float2 p2;
    float2 x_clip;
    float2 y_clip;
    float2 gradient_direction; // normalized direction for linear gradient
    float4 color_start;
    float4 color_end;
    uint gradient_type; // 0=solid, 1=linear
    float pad0;
    float pad1;
    float pad2;
}

struct Transform {
    float m11; float m12;
    float m21; float m22;
    float m31; float m32;
}

struct ShapesData {
    StructuredBuffer<Transform> transforms;
    StructuredBuffer<BoxData> boxes;
    StructuredBuffer<CircleData> circles;
    StructuredBuffer<SegmentData> segments;
    StructuredBuffer<GridData> grids;
    StructuredBuffer<TriangleData> triangles;
}

ParameterBlock<ShapesData> shapes;

void box_vertex(
    uint instanceIndex,
    uint vertexIndex,
    float2 resolution,
    out float4 position,
    out float2 uv,
    out float2 screen_pos,
) {
    var box: BoxData = shapes.boxes[instanceIndex];
    quad_vertex(box.top_left, box.size, vertexIndex, resolution, position, uv);
    screen_pos = box.top_left + uv * box.size;
}

float linear(float a, float b, float x) { return (x-a)/(b-a); }
float linearstep(float a, float b, float x) { return clamp(linear(a,b,x), 0.0, 1.0); }

float srgb_to_linear(float x) {
    return x <= 0.04045 ? x / 12.92 : pow((x + 0.055) / 1.055, 2.4);
}
float3 srgb_to_linear(float3 srgb) {
    return float3(
        srgb_to_linear(srgb.r),
        srgb_to_linear(srgb.g),
        srgb_to_linear(srgb.b)
    );
}
float4 srgb_to_linear(float4 srgba) {
    return float4(srgb_to_linear(srgba.rgb), srgba.a);
}

// Oklab color space functions for perceptually uniform gradients
float3 linear_srgb_to_oklab(float3 c) {
    float l = 0.4122214708 * c.r + 0.5363325363 * c.g + 0.0514459929 * c.b;
    float m = 0.2119034982 * c.r + 0.6806995451 * c.g + 0.1073969566 * c.b;
    float s = 0.0883024619 * c.r + 0.2817188376 * c.g + 0.6299787005 * c.b;

    float l_ = pow(l, 1.0/3.0);
    float m_ = pow(m, 1.0/3.0);
    float s_ = pow(s, 1.0/3.0);

    return float3(
        0.2104542553 * l_ + 0.7936177850 * m_ - 0.0040720468 * s_,
        1.9779984951 * l_ - 2.4285922050 * m_ + 0.4505937099 * s_,
        0.0259040371 * l_ + 0.7827717662 * m_ - 0.8086757660 * s_
    );
}

float3 oklab_to_linear_srgb(float3 c) {
    float l_ = c.x + 0.3963377774 * c.y + 0.2158037573 * c.z;
    float m_ = c.x - 0.1055613458 * c.y - 0.0638541728 * c.z;
    float s_ = c.x - 0.0894841775 * c.y - 1.2914855480 * c.z;

    float l = l_ * l_ * l_;
    float m = m_ * m_ * m_;
    float s = s_ * s_ * s_;

    return float3(
        +4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s,
        -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s,
        -0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s
    );
}

// Interpolate between two colors in Oklab space
float4 gradient_oklab(float4 color_start, float4 color_end, float t) {
    float3 lab_start = linear_srgb_to_oklab(color_start.rgb);
    float3 lab_end = linear_srgb_to_oklab(color_end.rgb);
    float3 lab_mixed = lerp(lab_start, lab_end, t);
    float3 rgb = oklab_to_linear_srgb(lab_mixed);
    float alpha = lerp(color_start.a, color_end.a, t);
    return float4(rgb, alpha);
}

float sd_rounded_box(float2 p, float2 size, float r) {
    float2 q = abs(p) - size + r;
    return length(max(q, 0.0)) + min(max(q.x, q.y), 0.0) - r;
}

float4 box_fragment(uint primitiveIndex, float2 uv, float2 screen_pos, bool surface_is_srgb): SV_Target {
    var box: BoxData = shapes.boxes[primitiveIndex];

    if (screen_pos.x < box.x_clip.x || screen_pos.x > box.x_clip.y ||
        screen_pos.y < box.y_clip.x || screen_pos.y > box.y_clip.y) {
        discard;
    }

    float2 p = (uv - 0.5) * box.size;

    float2 half_size = box.size * 0.5;
    float dist = sd_rounded_box(p, half_size, box.corner_radius);

    // Handle border-only rendering when border_thickness > 0
    if (box.border_thickness > 0.001) {
        dist = abs(dist + box.border_thickness * 0.5) - box.border_thickness * 0.5;
    }

    float alpha = 1.0 - linearstep(-0.5, 0.5, dist);

    if (alpha < 0.001) {
        discard;
    }

    // Calculate gradient color
    float4 color;
    if (box.gradient_type == 1) {
        // Linear gradient
        float2 pos = (uv - 0.5) * box.size;
        float t = dot(pos, box.gradient_direction) / length(box.size * 0.5);
        t = t * 0.5 + 0.5; // Remap from [-1,1] to [0,1]
        color = gradient_oklab(box.color_start, box.color_end, clamp(t, 0.0, 1.0));
    } else {
        // Solid color (gradient_type == 0)
        color = box.color_start;
    }

    if (surface_is_srgb) {
        return srgb_to_linear(float4(color.rgb, color.a * alpha));
    } else {
        return float4(color.rgb, color.a * alpha);
    }
}

void circle_vertex(
    uint instanceIndex,
    uint vertexIndex,
    float2 resolution,
    out float4 position,
    out float2 uv,
    out float2 screen_pos,
) {
    var circle: CircleData = shapes.circles[instanceIndex];

    // Create bounding box for the circle
    float outer_radius = circle.radii.y;
    float2 top_left = circle.center - float2(outer_radius, outer_radius);
    float2 size = float2(outer_radius * 2.0, outer_radius * 2.0);

    quad_vertex(top_left, size, vertexIndex, resolution, position, uv);
    screen_pos = top_left + uv * size;
}

float normalize_angle(float angle) {
    const float TAU = 6.28318530718;
    float result = fmod(angle, TAU);
    if (result < 0.0) result += TAU;
    return result;
}

bool is_in_arc(float angle, float start, float end) {
    const float TAU = 6.28318530718;

    // Normalize all angles to [0, 2π]
    angle = normalize_angle(angle);
    start = normalize_angle(start);
    end = normalize_angle(end);

    // Handle wrap-around case
    if (start <= end) {
        return angle >= start && angle <= end;
    } else {
        return angle >= start || angle <= end;
    }
}

float sd_circle_variant(float2 p, CircleData circle) {
    const float TAU = 6.28318530718;

    float inner_radius = circle.radii.x;
    float outer_radius = circle.radii.y;
    float start_angle = circle.angles.x;
    float end_angle = circle.angles.y;

    float dist_from_center = length(p);

    // Check if this is a full circle (angles span full range)
    float angle_span = abs(end_angle - start_angle);
    bool is_full_circle = abs(angle_span - TAU) < 0.01;

    if (is_full_circle) {
        // Simple ring/circle case
        if (inner_radius < 0.001) {
            // Filled circle
            return dist_from_center - outer_radius;
        } else {
            // Ring (annulus)
            return abs(dist_from_center - (inner_radius + outer_radius) * 0.5) - (outer_radius - inner_radius) * 0.5;
        }
    } else {
        // Arc or pie case - need to check angle
        float angle = atan2(p.y, p.x);

        if (is_in_arc(angle, start_angle, end_angle)) {
            // Inside the angular range
            if (inner_radius < 0.001) {
                // Pie slice
                return dist_from_center - outer_radius;
            } else {
                // Arc
                return abs(dist_from_center - (inner_radius + outer_radius) * 0.5) - (outer_radius - inner_radius) * 0.5;
            }
        } else {
            // Outside the angular range - far away
            return 1000.0;
        }
    }
}

float4 circle_fragment(uint primitiveIndex, float2 uv, float2 screen_pos, bool srgb): SV_Target {
    var circle: CircleData = shapes.circles[primitiveIndex];

    if (screen_pos.x < circle.x_clip.x || screen_pos.x > circle.x_clip.y ||
        screen_pos.y < circle.y_clip.x || screen_pos.y > circle.y_clip.y) {
        discard;
    }

    // Convert UV to position relative to circle center
    float outer_radius = circle.radii.y;
    float2 top_left = circle.center - float2(outer_radius, outer_radius);
    float2 size = float2(outer_radius * 2.0, outer_radius * 2.0);

    float2 p = top_left + uv * size - circle.center;

    float dist = sd_circle_variant(p, circle);

    float alpha = 1.0 - linearstep(-0.5, 0.5, dist);

    if (alpha < 0.001) {
        discard;
    }

    // Calculate gradient color
    float4 color;
    if (circle.gradient_type == 1) {
        // Linear gradient
        float t = dot(p, circle.gradient_direction) / outer_radius;
        t = t * 0.5 + 0.5; // Remap from [-1,1] to [0,1]
        color = gradient_oklab(circle.color_start, circle.color_end, clamp(t, 0.0, 1.0));
    } else if (circle.gradient_type == 2) {
        // Radial gradient
        float t = length(p) / outer_radius;
        color = gradient_oklab(circle.color_start, circle.color_end, clamp(t, 0.0, 1.0));
    } else {
        // Solid color (gradient_type == 0)
        color = circle.color_start;
    }

    return float4(color.rgb, alpha * color.a);
}

void segment_vertex(
    uint instanceIndex,
    uint vertexIndex,
    float2 resolution,
    out float4 position,
    out float2 uv,
    out float2 screen_pos,
) {
    var seg: SegmentData = shapes.segments[instanceIndex];

    // Create bounding box for the segment
    float half_thickness = seg.thickness_dash[0] * 0.5;
    float2 min_point = min(seg.start, seg.end) - float2(half_thickness, half_thickness);
    float2 max_point = max(seg.start, seg.end) + float2(half_thickness, half_thickness);

    float2 top_left = min_point;
    float2 size = max_point - min_point;

    quad_vertex(top_left, size, vertexIndex, resolution, position, uv);
    screen_pos = top_left + uv * size;
}

// SDF for line segment from Inigo Quilez
float sd_segment(float2 p, float2 a, float2 b) {
    float2 pa = p - a;
    float2 ba = b - a;
    float h = clamp(dot(pa, ba) / dot(ba, ba), 0.0, 1.0);
    return length(pa - ba * h);
}

float4 segment_fragment(uint primitiveIndex, float2 uv, float2 screen_pos, bool srgb): SV_Target {
    var seg: SegmentData = shapes.segments[primitiveIndex];

    if (screen_pos.x < seg.x_clip.x || screen_pos.x > seg.x_clip.y ||
        screen_pos.y < seg.y_clip.x || screen_pos.y > seg.y_clip.y) {
        discard;
    }

    // Convert UV to world position
    float half_thickness = seg.thickness_dash[0] * 0.5;
    float2 min_point = min(seg.start, seg.end) - float2(half_thickness, half_thickness);
    float2 max_point = max(seg.start, seg.end) + float2(half_thickness, half_thickness);

    float2 top_left = min_point;
    float2 size = max_point - min_point;

    float2 p = top_left + uv * size;

    // Calculate distance to segment
    float dist = sd_segment(p, seg.start, seg.end) - half_thickness;

    float alpha = 1.0 - linearstep(-0.5, 0.5, dist);

    if (alpha < 0.001) {
        discard;
    }

    // Handle dashed lines
    float dash_length = seg.thickness_dash[1];
    if (dash_length > 0.0001) {
        float2 seg_vec = seg.end - seg.start;
        float seg_len = length(seg_vec);
        float t = 0.0;
        if (seg_len > 0.001) {
            t = dot(p - seg.start, seg_vec) / seg_len;
        }

        float dash_period = dash_length * 2.0;
        float pos_in_period = fmod(t, dash_period);

        float dist_to_start = pos_in_period;
        float dist_to_end = dash_length - pos_in_period;
        float dist_to_nearest_edge = min(dist_to_start, dist_to_end);

        float dash_alpha = linearstep(-0.5, 0.5, dist_to_nearest_edge);

        alpha *= dash_alpha;

        if (alpha < 0.001) {
            discard;
        }
    }

    // Calculate gradient color
    float4 color;
    if (seg.gradient_type == 1) {
        // Linear gradient along segment
        float2 seg_vec = seg.end - seg.start;
        float seg_len = length(seg_vec);
        float t = 0.5;
        if (seg_len > 0.001) {
            t = dot(p - seg.start, seg_vec) / (seg_len * seg_len);
        }
        color = gradient_oklab(seg.color_start, seg.color_end, clamp(t, 0.0, 1.0));
    } else {
        // Solid color (gradient_type == 0)
        color = seg.color_start;
    }

    return float4(color.rgb, alpha * color.a);
}

void grid_vertex(
    uint instanceIndex,
    uint vertexIndex,
    float2 resolution,
    out float4 position,
    out float2 uv,
    out float2 screen_pos,
) {
    var grid: GridData = shapes.grids[instanceIndex];
    quad_vertex(grid.top_left, grid.size, vertexIndex, resolution, position, uv);
    screen_pos = grid.top_left + uv * grid.size;
}

// Thanks leftofzen. https://www.shadertoy.com/view/wtdSzX
float4 grid_fragment(uint primitiveIndex, float2 uv, float2 screen_pos, bool surface_is_srgb): SV_Target {
    var grid: GridData = shapes.grids[primitiveIndex];

    // Apply clipping
    if (screen_pos.x < grid.x_clip.x || screen_pos.x > grid.x_clip.y ||
        screen_pos.y < grid.y_clip.x || screen_pos.y > grid.y_clip.y) {
        discard;
    }

    // Transform to grid space
    float2 p_pixels = screen_pos - grid.top_left + grid.offset;

    float dist;
    if (grid.grid_type == 0) {
        // Square grid
        float2 p = p_pixels / grid.lattice_size;
        float2 grid_pos = fmod(p, 1.0);
        float2 dist_to_line = min(grid_pos, 1.0 - grid_pos);
        dist = min(dist_to_line.x, dist_to_line.y) * grid.lattice_size;
    } else {
        // Scale by lattice_size to work in normalized hex space
        float2 p = p_pixels / grid.lattice_size;
        float2 s = float2(1.0, 1.7320508); // vec2(1, sqrt(3))

        // Two candidate hexagon centers
        float2 hC1 = floor(p / s) + 0.5;
        float2 hC2 = floor((p - float2(0.5, 1.0)) / s) + 0.5;

        // Positions relative to each center
        float2 h1 = p - hC1 * s;
        float2 h2 = p - (hC2 + 0.5) * s;

        // Pick nearest center
        float2 h = dot(h1, h1) < dot(h2, h2) ? h1 : h2;

        // Hexagon SDF
        float2 q = abs(h);
        float hexDist = max(dot(q, s * 0.5), q.x);

        // Distance to hex edge (edge is at hexDist = 0.5)
        dist = abs(hexDist - 0.5) * grid.lattice_size;
    }

    // Apply line thickness and antialiasing
    float half_thickness = grid.line_thickness * 0.5;
    float alpha = 1.0 - linearstep(half_thickness - 0.5, half_thickness + 0.5, dist);

    if (alpha < 0.001) {
        discard;
    }

    float4 color = grid.color;

    return float4(color.rgb, color.a * alpha);
}

void triangle_vertex(
    uint instanceIndex,
    uint vertexIndex,
    float2 resolution,
    out float4 position,
    out float2 uv,
    out float2 screen_pos,
) {
    var tri: TriangleData = shapes.triangles[instanceIndex];

    // Create bounding box for the triangle
    float2 min_point = min(tri.p0, min(tri.p1, tri.p2));
    float2 max_point = max(tri.p0, max(tri.p1, tri.p2));

    float2 top_left = min_point;
    float2 size = max_point - min_point;

    quad_vertex(top_left, size, vertexIndex, resolution, position, uv);
    screen_pos = top_left + uv * size;
}

// Compute barycentric coordinates for a point in a triangle
float3 barycentric(float2 p, float2 a, float2 b, float2 c) {
    float2 v0 = b - a;
    float2 v1 = c - a;
    float2 v2 = p - a;

    float d00 = dot(v0, v0);
    float d01 = dot(v0, v1);
    float d11 = dot(v1, v1);
    float d20 = dot(v2, v0);
    float d21 = dot(v2, v1);

    float denom = d00 * d11 - d01 * d01;
    float v = (d11 * d20 - d01 * d21) / denom;
    float w = (d00 * d21 - d01 * d20) / denom;
    float u = 1.0 - v - w;

    return float3(u, v, w);
}

// SDF for triangle - returns signed distance
float sd_triangle(float2 p, float2 p0, float2 p1, float2 p2) {
    float2 e0 = p1 - p0;
    float2 e1 = p2 - p1;
    float2 e2 = p0 - p2;

    float2 v0 = p - p0;
    float2 v1 = p - p1;
    float2 v2 = p - p2;

    float2 pq0 = v0 - e0 * clamp(dot(v0, e0) / dot(e0, e0), 0.0, 1.0);
    float2 pq1 = v1 - e1 * clamp(dot(v1, e1) / dot(e1, e1), 0.0, 1.0);
    float2 pq2 = v2 - e2 * clamp(dot(v2, e2) / dot(e2, e2), 0.0, 1.0);

    float s = sign(e0.x * e2.y - e0.y * e2.x);
    float2 d = min(min(float2(dot(pq0, pq0), s * (v0.x * e0.y - v0.y * e0.x)),
                       float2(dot(pq1, pq1), s * (v1.x * e1.y - v1.y * e1.x))),
                       float2(dot(pq2, pq2), s * (v2.x * e2.y - v2.y * e2.x)));

    return -sqrt(d.x) * sign(d.y);
}

float4 triangle_fragment(uint primitiveIndex, float2 uv, float2 screen_pos, bool surface_is_srgb): SV_Target {
    var tri: TriangleData = shapes.triangles[primitiveIndex];

    if (screen_pos.x < tri.x_clip.x || screen_pos.x > tri.x_clip.y ||
        screen_pos.y < tri.y_clip.x || screen_pos.y > tri.y_clip.y) {
        discard;
    }

    // Convert UV to world position
    float2 min_point = min(tri.p0, min(tri.p1, tri.p2));
    float2 max_point = max(tri.p0, max(tri.p1, tri.p2));
    float2 top_left = min_point;
    float2 size = max_point - min_point;
    float2 p = top_left + uv * size;

    // Calculate distance to triangle
    float dist = sd_triangle(p, tri.p0, tri.p1, tri.p2);

    float alpha = 1.0 - linearstep(-0.5, 0.5, dist);

    if (alpha < 0.001) {
        discard;
    }

    // Calculate gradient color
    float4 color;
    if (tri.gradient_type == 1) {
        // Linear gradient
        float2 centroid = (tri.p0 + tri.p1 + tri.p2) / 3.0;
        float2 pos = p - centroid;
        float tri_size = length(max_point - min_point);
        float t = dot(pos, tri.gradient_direction) / (tri_size * 0.5);
        t = t * 0.5 + 0.5; // Remap from [-1,1] to [0,1]
        color = gradient_oklab(tri.color_start, tri.color_end, clamp(t, 0.0, 1.0));
    } else {
        // Solid color (gradient_type == 0)
        color = tri.color_start;
    }

    if (surface_is_srgb) {
        return srgb_to_linear(float4(color.rgb, color.a * alpha));
    } else {
        return float4(color.rgb, color.a * alpha);
    }
}
