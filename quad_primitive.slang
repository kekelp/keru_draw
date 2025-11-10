import primitives_common;

struct QuadData {
    float2 positions[4];
    float3 color;
}

QuadData decodeQuad(PrimitiveData data) {
    QuadData quad;
    float4 f0 = asfloat(data.data[0]);
    float4 f1 = asfloat(data.data[1]);
    float4 f2 = asfloat(data.data[2]);

    quad.positions[0] = f0.xy;
    quad.positions[1] = f0.zw;
    quad.positions[2] = f1.xy;
    quad.positions[3] = f1.zw;
    quad.color = f2.xyz;

    return quad;
}

void quadVertex(
    PrimitiveData primData,
    uint primitiveIndex,
    uint localVertexIndex,
    out float4 position,
    out float3 color
) {
    QuadData quad = decodeQuad(primData);

    // Convert quad to triangles (0,1,2) and (0,2,3)
    uint indices[6] = {0, 1, 2, 0, 2, 3};
    uint quadVertexIndex = indices[localVertexIndex];

    position = float4(quad.positions[quadVertexIndex], 0.0, 1.0);
    color = quad.color;
}

float4 quadFragment(VertexOutput input): SV_Target {
    return float4(input.color, 1.0);
}
