// Main shader - dispatches to primitive-specific modules
import primitives_common;
import ellipse;
import rectangle;

enum PrimitiveType: uint {
    Quad = 0,
    Ellipse = 1,
}

[shader("vertex")]
VertexOutput vertexMain(VertexInput input, uint vertexIndex: SV_VulkanVertexID, uint instanceIndex: SV_VulkanInstanceID) {
    VertexOutput output;
    output.primitiveType = input.primitiveType;

    // Dispatch to primitive-specific vertex function
    if (input.primitiveType == uint(PrimitiveType::Quad)) {
        rectangle_vertex(instanceIndex, vertexIndex, output.position, output.uv, output.color);
    } else if (input.primitiveType == uint(PrimitiveType::Ellipse)) {
        ellipse_vertex(instanceIndex, vertexIndex, output.position, output.uv, output.color);
    } else {
        // Default case
        output.position = float4(0.0, 0.0, 0.0, 1.0);
        output.uv = float2(0.0, 0.0);
        output.color = float3(1.0, 0.0, 1.0);  // Magenta for error
    }

    return output;
}

[shader("fragment")]
float4 fragmentMain(VertexOutput input): SV_Target {
    // Dispatch to primitive-specific fragment function
    if (input.primitiveType == uint(PrimitiveType::Quad)) {
        return rectangle_fragment(input);
    } else if (input.primitiveType == uint(PrimitiveType::Ellipse)) {
        return ellipse_fragment(input);
    } else {
        return float4(1.0, 0.0, 1.0, 1.0);  // Magenta for error
    }
}
