// Main shader - dispatches to primitive-specific modules
import primitives_common;
import quad_primitive_new;
import quad_primitive;

[shader("vertex")]
VertexOutput vertexMain(VertexInput input, uint vertexIndex: SV_VulkanVertexID, uint instanceIndex: SV_VulkanInstanceID) {
    VertexOutput output;
    output.primitiveType = input.primitiveType;

    // Dispatch to primitive-specific vertex function
    if (input.primitiveType == uint(PrimitiveType::Quad)) {
        quadVertex(instanceIndex, vertexIndex, output.position, output.uv, output.color);
    } else if (input.primitiveType == uint(PrimitiveType::Ellipse)) {
        ellipseVertex(instanceIndex, vertexIndex, output.position, output.uv, output.color);
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
        return quadFragment(input);
    } else if (input.primitiveType == uint(PrimitiveType::Ellipse)) {
        return ellipseFragment(input);
    } else {
        return float4(1.0, 0.0, 1.0, 1.0);  // Magenta for error
    }
}
