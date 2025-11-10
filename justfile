compile_shaders:
    slangc shader.slang -target spirv -entry vertexMain -stage vertex -o shader.vert.spv && slangc shader.slang -target spirv -entry fragmentMain -stage fragment -o shader.frag.spv
show_bindings:
    jq -r '.parameters[] | select(.type.kind == "parameterBlock") | "\(.binding.index): \(.name)"' slangc_reflection.json