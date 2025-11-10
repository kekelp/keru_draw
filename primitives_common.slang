enum PrimitiveType: uint {
    Quad = 0,
    Ellipse = 1,
}

// Output from vertex processing
struct VertexOutput {
    float4 position: SV_Position;
    float2 uv: TEXCOORD0;
    float3 color: COLOR;
    uint primitiveType: PRIMTYPE;
}

// Input to vertex shader
struct VertexInput {
    uint primitiveType: PRIMTYPE;
    uint primitiveIndex: PRIMINDEX;
}
