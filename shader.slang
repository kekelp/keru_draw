// Main shader - dispatches to primitive-specific modules
import primitives_common;
import triangle_primitive;
import quad_primitive;
import circle_primitive;

[shader("vertex")]
VertexOutput vertexMain(VertexInput input) {
    VertexOutput output;

    output.primitiveType = input.primitiveType;
    output.primitiveIndex = input.primitiveIndex;

    // Dispatch to primitive-specific vertex function
    if (input.primitiveType == uint(PrimitiveType::Triangle)) {
        triangleVertex(input.primitiveIndex, input.localVertexIndex, output.position, output.color);
    } else if (input.primitiveType == uint(PrimitiveType::Quad)) {
        quadVertex(input.primitiveIndex, input.localVertexIndex, output.position, output.color);
    } else if (input.primitiveType == uint(PrimitiveType::Circle)) {
        circleVertex(input.primitiveIndex, input.localVertexIndex, output.position, output.color);
    } else {
        // Default case
        output.position = float4(0.0, 0.0, 0.0, 1.0);
        output.color = float3(1.0, 0.0, 1.0);  // Magenta for error
    }

    return output;
}

[shader("fragment")]
float4 fragmentMain(VertexOutput input): SV_Target {
    // Dispatch to primitive-specific fragment function
    if (input.primitiveType == uint(PrimitiveType::Triangle)) {
        return triangleFragment(input);
    } else if (input.primitiveType == uint(PrimitiveType::Quad)) {
        return quadFragment(input);
    } else if (input.primitiveType == uint(PrimitiveType::Circle)) {
        return circleFragment(input);
    } else {
        return float4(1.0, 0.0, 1.0, 1.0);  // Magenta for error
    }
}
