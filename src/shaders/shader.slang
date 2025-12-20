ParameterBlock<Globals> globals;
// The order of the imports has to match the binding indices in the Rust code!
// If there are issues, compile the shaders with the -reflection-json flag and see the parameterBlock fields.
import quad;
import ellipse;
import rectangle;

enum PrimitiveType: uint {
    Rectangle = 0,
    Ellipse = 1,
}

struct Globals {
    float2 resolution;
    float time;
    float _padding;
}


struct VertexOutput {
    float4 position: SV_Position;
    uint primitiveType: PRIMTYPE;
    uint primitiveIndex: PRIMINDEX;
    float2 uv: TEXCOORD0;
}

struct VertexInput {
    uint primitiveType: PRIMTYPE;
    uint primitiveIndex: PRIMINDEX;
}


[shader("vertex")]
VertexOutput vertexMain(
    VertexInput input,
    uint vertexIndex: SV_VulkanVertexID
) {
    VertexOutput output;
    output.primitiveType = input.primitiveType;
    output.primitiveIndex = input.primitiveIndex;

    if (input.primitiveType == PrimitiveType::Rectangle) {
        output.uv = rectangle_vertex(input.primitiveIndex, vertexIndex, globals.resolution, output.position);
    }
    else if (input.primitiveType == PrimitiveType::Ellipse) {
        output.uv = ellipse_vertex(input.primitiveIndex, vertexIndex, globals.resolution, output.position);
    }

    return output;
}

[shader("fragment")]
float4 fragmentMain(VertexOutput input): SV_Target {

    if (input.primitiveType == PrimitiveType::Rectangle) {
        return rectangle_fragment(input.primitiveIndex, input.uv);
    }
    else if (input.primitiveType == PrimitiveType::Ellipse) {
        return ellipse_fragment(input.primitiveIndex, input.uv);
    }

    return float4(0.0, 1.0, 0.0, 1.0);  // Greenscreen for error
}
