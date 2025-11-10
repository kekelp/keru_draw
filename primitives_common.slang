// Common types and structures shared across all primitives

enum PrimitiveType: uint {
    Triangle = 0,
    Quad = 1,
    Circle = 2,
}

// Raw primitive data storage - 64 bytes
struct PrimitiveData {
    uint4 data[4];
}

// Output from vertex processing
struct VertexOutput {
    float4 position: SV_Position;
    float3 color: COLOR;
    uint primitiveType: PRIMTYPE;
    uint primitiveIndex: PRIMINDEX;
}

// Input to vertex shader
struct VertexInput {
    uint primitiveType: PRIMTYPE;
    uint vertexIndex: VERTEXID;
}
