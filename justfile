compile_shaders:
    slangc shader.slang -target spirv -entry vertexMain -stage vertex -o shader.vert.spv && slangc shader.slang -target spirv -entry fragmentMain -stage fragment -o shader.frag.spv
get_bindings:
    jq '.entryPoints[0].parameters[] | select(.type.kind == "parameterBlock") | {name: .name, set: .binding.index, resources: [.type.elementType.fields[] | {name: .name, binding: .binding.index, type: .type.baseShape}]}' slangc_reflection.json