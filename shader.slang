// Main shader - dispatches to primitive-specific modules
import primitives_common;
import ellipse;
import rectangle;

[shader("vertex")]
VertexOutput vertexMain(VertexInput input, uint vertexIndex: SV_VulkanVertexID) {
    VertexOutput output;
    output.primitiveType = input.primitiveType;

    // Dispatch to primitive-specific vertex function using primitive index from input
    if (input.primitiveType == uint(PrimitiveType::Quad)) {
        rectangle_vertex(input.primitiveIndex, vertexIndex, output.position, output.uv, output.color);
    }
    else if (input.primitiveType == uint(PrimitiveType::Ellipse)) {
        ellipse_vertex(input.primitiveIndex, vertexIndex, output.position, output.uv, output.color);
    }

    return output;
}

[shader("fragment")]
float4 fragmentMain(VertexOutput input): SV_Target {
    // Dispatch to primitive-specific fragment function
    if (input.primitiveType == uint(PrimitiveType::Quad)) {
        return rectangle_fragment(input);
    }
    else if (input.primitiveType == uint(PrimitiveType::Ellipse)) {
        return ellipse_fragment(input);
    }
}
