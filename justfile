compile_shaders:
    slangc shader.slang -target spirv -entry vertexMain -stage vertex -o shader.vert.spv && slangc shader.slang -target spirv -entry fragmentMain -stage fragment -o shader.frag.spv
