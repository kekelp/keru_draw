// Main shader - dispatches to primitive-specific modules
import primitives_common;
import ellipse;
import rectangle;

enum PrimitiveType: uint {
    Quad = 0,
    Ellipse = 1,
}


struct Union {
    float4 data0;
    float4 data1;
}

struct VertexOutput {
    float4 position: SV_Position;
    uint primitiveType: PRIMTYPE;
    Union bytes: TEXCOORD0;
}

struct VertexInput {
    uint primitiveType: PRIMTYPE;
    uint primitiveIndex: PRIMINDEX;
}


[shader("vertex")]
VertexOutput vertexMain(VertexInput input, uint vertexIndex: SV_VulkanVertexID) {
    VertexOutput output;
    output.primitiveType = input.primitiveType;

    if (input.primitiveType == uint(PrimitiveType::Quad)) {
        RectangleVertexOutput rectData = rectangle_vertex(input.primitiveIndex, vertexIndex, output.position);
        output.bytes = reinterpret<Union>(rectData);
    }
    else if (input.primitiveType == uint(PrimitiveType::Ellipse)) {
        EllipseVertexOutput ellipseData = ellipse_vertex(input.primitiveIndex, vertexIndex, output.position);
        output.bytes = reinterpret<Union>(ellipseData);
    }

    return output;
}

[shader("fragment")]
float4 fragmentMain(VertexOutput input): SV_Target {

    if (input.primitiveType == uint(PrimitiveType::Quad)) {
        RectangleVertexOutput rectData = reinterpret<RectangleVertexOutput>(input.bytes);
        return rectangle_fragment(rectData);
    }
    else if (input.primitiveType == uint(PrimitiveType::Ellipse)) {
        EllipseVertexOutput ellipseData = reinterpret<EllipseVertexOutput>(input.bytes);
        return ellipse_fragment(ellipseData);
    }

    return float4(0.0, 1.0, 0.0, 1.0);  // Greenscreen for error
}
