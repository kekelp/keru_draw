import primitives_common;

struct CircleData
{
    float2 center;
    float radius;
    float3 color;
    uint segments;
};

CircleData decodeCircle(PrimitiveData data)
{
    CircleData circle;
    float4 f0 = asfloat(data.data[0]);
    float4 f1 = asfloat(data.data[1]);

    circle.center = f0.xy;
    circle.radius = f0.z;
    circle.color = float3(f0.w, f1.xy);
    circle.segments = asuint(f1.z);

    return circle;
}

void circleVertex(
    PrimitiveData primData,
    uint primitiveIndex,
    uint localVertexIndex,
    out float4 position,
    out float3 color)
{
    CircleData circle = decodeCircle(primData);

    // Generate circle vertices procedurally
    float angle = (float(localVertexIndex) / float(circle.segments)) * 2.0 * 3.14159265;
    float2 offset = float2(cos(angle), sin(angle)) * circle.radius;
    float2 pos = circle.center + offset;

    position = float4(pos, 0.0, 1.0);
    color = circle.color;
}

float4 circleFragment(VertexOutput input) : SV_Target
{
    return float4(input.color, 1.0);
}
