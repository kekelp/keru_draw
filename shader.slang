// Vertex input is just primitive type and index
struct VertexInput
{
    uint primitiveType : PRIMTYPE;
    uint vertexIndex : VERTEXID;
};

// Different primitive types
enum PrimitiveType : uint
{
    Triangle = 0,
    Quad = 1,
    Circle = 2
};

// Data stored in the primitive buffer
struct TriangleData
{
    float2 positions[3];
    float3 color;
};

struct QuadData
{
    float2 positions[4];
    float3 color;
};

struct CircleData
{
    float2 center;
    float radius;
    float3 color;
    uint segments;
};

// Union-like structure using Slang's bit casting capabilities
struct PrimitiveData
{
    uint4 data[4];  // Raw data storage - 64 bytes
};

struct VertexOutput
{
    float4 position : SV_Position;
    float3 color : COLOR;
    uint primitiveType : PRIMTYPE;
    uint primitiveIndex : PRIMINDEX;
};

// Primitive buffer - storage buffer
[[vk::binding(0, 0)]]
StructuredBuffer<PrimitiveData> primitiveBuffer;

// Helper to reinterpret data as TriangleData
TriangleData asTriangle(PrimitiveData data)
{
    TriangleData tri;
    // Manual bitcast from uint4 array to triangle data
    float4 f0 = asfloat(data.data[0]);
    float4 f1 = asfloat(data.data[1]);
    float4 f2 = asfloat(data.data[2]);

    tri.positions[0] = f0.xy;
    tri.positions[1] = f0.zw;
    tri.positions[2] = f1.xy;
    tri.color = f1.zwx;

    return tri;
}

QuadData asQuad(PrimitiveData data)
{
    QuadData quad;
    float4 f0 = asfloat(data.data[0]);
    float4 f1 = asfloat(data.data[1]);
    float4 f2 = asfloat(data.data[2]);

    quad.positions[0] = f0.xy;
    quad.positions[1] = f0.zw;
    quad.positions[2] = f1.xy;
    quad.positions[3] = f1.zw;
    quad.color = f2.xyz;

    return quad;
}

CircleData asCircle(PrimitiveData data)
{
    CircleData circle;
    float4 f0 = asfloat(data.data[0]);
    float4 f1 = asfloat(data.data[1]);

    circle.center = f0.xy;
    circle.radius = f0.z;
    circle.color = float3(f0.w, f1.xy);
    circle.segments = asuint(f1.z);

    return circle;
}

[shader("vertex")]
VertexOutput vertexMain(VertexInput input)
{
    VertexOutput output;

    // Calculate which primitive to read from
    uint primitiveIndex = input.vertexIndex / 3;  // Assuming triangles for now
    uint localVertexIndex = input.vertexIndex % 3;

    // Fetch primitive data from buffer
    PrimitiveData primData = primitiveBuffer[primitiveIndex];

    output.primitiveType = input.primitiveType;
    output.primitiveIndex = primitiveIndex;

    // Decode based on primitive type
    if (input.primitiveType == uint(PrimitiveType::Triangle))
    {
        TriangleData tri = asTriangle(primData);
        output.position = float4(tri.positions[localVertexIndex], 0.0, 1.0);
        output.color = tri.color;
    }
    else if (input.primitiveType == uint(PrimitiveType::Quad))
    {
        // For quad, we need to handle triangle fan or strip
        QuadData quad = asQuad(primData);
        // Convert quad to triangle (assuming two triangles)
        uint indices[6] = {0, 1, 2, 0, 2, 3};
        uint quadVertexIndex = indices[localVertexIndex];
        output.position = float4(quad.positions[quadVertexIndex], 0.0, 1.0);
        output.color = quad.color;
    }
    else if (input.primitiveType == uint(PrimitiveType::Circle))
    {
        CircleData circle = asCircle(primData);

        // Generate circle vertices procedurally
        float angle = (float(localVertexIndex) / float(circle.segments)) * 2.0 * 3.14159265;
        float2 offset = float2(cos(angle), sin(angle)) * circle.radius;
        float2 pos = circle.center + offset;

        output.position = float4(pos, 0.0, 1.0);
        output.color = circle.color;
    }
    else
    {
        // Default case
        output.position = float4(0.0, 0.0, 0.0, 1.0);
        output.color = float3(1.0, 0.0, 1.0);  // Magenta for error
    }

    return output;
}

[shader("fragment")]
float4 fragmentMain(VertexOutput input) : SV_Target
{
    // Could also pull from buffer here if needed
    // PrimitiveData primData = primitiveBuffer[input.primitiveIndex];

    return float4(input.color, 1.0);
}
