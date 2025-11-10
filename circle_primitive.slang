import primitives_common;

struct CircleData {
    float2 center;
    float radius;
    float3 color;
    uint segments;
}

[[vk::binding(2, 0)]]
StructuredBuffer<CircleData> circleBuffer;

void circleVertex(
    uint primitiveIndex,
    uint localVertexIndex,
    out float4 position,
    out float3 color
) {
    CircleData circle = circleBuffer[primitiveIndex];

    // Generate circle vertices procedurally
    float angle = (float(localVertexIndex) / float(circle.segments)) * 2.0 * 3.14159265;
    float2 offset = float2(cos(angle), sin(angle)) * circle.radius;
    float2 pos = circle.center + offset;

    position = float4(pos, 0.0, 1.0);
    color = circle.color;
}

float4 circleFragment(VertexOutput input): SV_Target {
    return float4(input.color, 1.0);
}
