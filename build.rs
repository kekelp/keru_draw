fn main() {
    println!("cargo:rerun-if-changed=src/shaders/text.slang");
    println!("cargo:rerun-if-changed=src/shaders/keru_images.slang");

    let imported_textslabs = include_str!("src/shaders/text.slang");
    let original_textslabs = textslabs::TextRenderer::composable_shader_source();
    assert!(imported_textslabs == original_textslabs, "Imported textslabs shader does not match original!");

    let imported_images = include_str!("src/shaders/keru_images.slang");
    let original_images = keru_images::ImageRenderer::composable_shader_source();
    assert!(imported_images == original_images, "Imported keru_images shader does not match original!");
}
