import primitives_common;
import triangle_primitive;
import quad_primitive;
import circle_primitive;

[[vk::binding(0, 0)]] StructuredBuffer<PrimitiveData> primitiveBuffer;

[shader("vertex")]
VertexOutput vertexMain(VertexInput input) {
    VertexOutput output;

    // Calculate which primitive to read from
    uint primitiveIndex = input.vertexIndex / 3;
    uint localVertexIndex = input.vertexIndex % 3;

    // Fetch primitive data from buffer
    PrimitiveData primData = primitiveBuffer[primitiveIndex];

    output.primitiveType = input.primitiveType;
    output.primitiveIndex = primitiveIndex;

    if (input.primitiveType == PrimitiveType::Triangle) {
        triangleVertex(primData, primitiveIndex, localVertexIndex, output.position, output.color);
    }
    else if (input.primitiveType == PrimitiveType::Quad) {
        quadVertex(primData, primitiveIndex, localVertexIndex, output.position, output.color);
    }
    else if (input.primitiveType == PrimitiveType::Circle) {
        circleVertex(primData, primitiveIndex, localVertexIndex, output.position, output.color);
    }

    return output;
}

[shader("fragment")]
float4 fragmentMain(VertexOutput input) : SV_Target {
    if (input.primitiveType == uint(PrimitiveType::Triangle)) {
        return triangleFragment(input);
    }
    else if (input.primitiveType == uint(PrimitiveType::Quad)) {
        return quadFragment(input);
    }
    else if (input.primitiveType == uint(PrimitiveType::Circle)) {
        return circleFragment(input);
    }
}
