// The order of the imports has to match the binding indices in the Rust code!
// If there are issues, compile the shaders with the -reflection-json flag and see the parameterBlock fields.
import keru_shapes;
import text;
import keru_images;

enum PrimitiveType: uint {
    Box = 0,
    Circle = 1,
    Segment = 2,
    Text = 3,
    Image = 4,
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

    // grab the screen resolution from textslabs
    float2 resolution = textslabs.params.resolution;

    if (input.primitiveType == PrimitiveType::Box) {
        box_vertex(input.primitiveIndex, vertexIndex, resolution, output.position, output.uv);
    }
    else if (input.primitiveType == PrimitiveType::Circle) {
        circle_vertex(input.primitiveIndex, vertexIndex, resolution, output.position, output.uv);
    }
    else if (input.primitiveType == PrimitiveType::Segment) {
        segment_vertex(input.primitiveIndex, vertexIndex, resolution, output.position, output.uv);
    }
    else if (input.primitiveType == PrimitiveType::Text) {
        text_vertex(input.primitiveIndex, vertexIndex, resolution, output.position, output.uv);
    }
    else if (input.primitiveType == PrimitiveType::Image) {
        image_vertex(input.primitiveIndex, vertexIndex, resolution, output.position, output.uv);
    }

    return output;
}

[shader("fragment")]
float4 fragmentMain(VertexOutput input): SV_Target {

    if (input.primitiveType == PrimitiveType::Box) {
        return box_fragment(input.primitiveIndex, input.uv);
    }
    else if (input.primitiveType == PrimitiveType::Circle) {
        return circle_fragment(input.primitiveIndex, input.uv);
    }
    else if (input.primitiveType == PrimitiveType::Segment) {
        return segment_fragment(input.primitiveIndex, input.uv);
    }
    else if (input.primitiveType == PrimitiveType::Text) {
        return text_fragment(input.primitiveIndex, input.uv);
    }
    else if (input.primitiveType == PrimitiveType::Image) {
        return image_fragment(input.primitiveIndex, input.uv);
    }

    return float4(0.0, 1.0, 0.0, 1.0);  // Greenscreen for error
}
