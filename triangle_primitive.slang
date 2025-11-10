import primitives_common;

struct TriangleData {
    float2 positions[3];
    float3 color;
}

TriangleData decodeTriangle(PrimitiveData data) {
    TriangleData tri;
    float4 f0 = asfloat(data.data[0]);
    float4 f1 = asfloat(data.data[1]);

    tri.positions[0] = f0.xy;
    tri.positions[1] = f0.zw;
    tri.positions[2] = f1.xy;
    tri.color = f1.zwx;

    return tri;
}

void triangleVertex(
    PrimitiveData primData,
    uint primitiveIndex,
    uint localVertexIndex,
    out float4 position,
    out float3 color
) {
    TriangleData tri = decodeTriangle(primData);
    position = float4(tri.positions[localVertexIndex], 0.0, 1.0);
    color = tri.color;
}

float4 triangleFragment(VertexOutput input): SV_Target {
    return float4(input.color, 1.0);
}
