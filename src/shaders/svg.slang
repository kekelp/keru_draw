struct SvgQuad {
    float pos_x;
    float pos_y;
    float width;
    float height;
    float uv_x;
    float uv_y;
    float uv_width;
    float uv_height;
    uint page;
    float depth;
    uint _padding0;
    uint _padding1;
}

struct Params {
    float2 resolution;
    uint _pad1;
    uint _pad2;
}

struct SvgSlabs {
    Texture2DArray<float4> atlas_texture;
    SamplerState atlas_sampler;
    ConstantBuffer<Params> params;
    StructuredBuffer<SvgQuad> quads;
}

ParameterBlock<SvgSlabs> svgslabs;

void svg_vertex(
    uint instanceIndex,
    uint vertexIndex,
    float2 resolution,
    out float4 position,
    out float2 uv,
) {
    SvgQuad quad = svgslabs.quads[instanceIndex];

    // Calculate vertex position in quad (0,0) to (1,1)
    float2 uv01 = float2(
        vertexIndex & 1u,
        vertexIndex >> 1u & 1u
    );

    // Calculate screen position
    float2 screen_pos = float2(quad.pos_x, quad.pos_y) + float2(quad.width, quad.height) * uv01;

    // Convert to NDC
    float2 ndc = (screen_pos / resolution) * 2.0 - 1.0;
    position = float4(ndc.x, -ndc.y, quad.depth, 1.0);

    // Calculate UV coordinates for texture atlas
    uint atlas_width, atlas_height, atlas_elements;
    svgslabs.atlas_texture.GetDimensions(atlas_width, atlas_height, atlas_elements);
    float2 atlas_size = float2(float(atlas_width), float(atlas_height));

    float2 uv_origin = float2(quad.uv_x, quad.uv_y);
    float2 uv_size = float2(quad.uv_width, quad.uv_height);

    uv = (uv_origin + uv_size * uv01) / atlas_size;
}

float4 svg_fragment_pure(SvgQuad quad, float2 uv): SV_Target {
    // Sample from texture array
    float4 tex_color = svgslabs.atlas_texture.SampleLevel(
        svgslabs.atlas_sampler,
        float3(uv, float(quad.page)),
        0.0
    );

    return tex_color;
}

float4 svg_fragment(uint primitiveIndex, float2 uv): SV_Target {
    SvgQuad quad = svgslabs.quads[primitiveIndex];
    return svg_fragment_pure(quad, uv);
}
