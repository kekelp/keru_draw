fn main() {
    println!("cargo:rerun-if-changed=src/shaders/textslabs.slang");
    println!("cargo:rerun-if-changed=src/shaders/keru_images.slang");

    let imported_textslabs_shader = include_str!("src/shaders/textslabs.slang");
    let original_textslabs_shader = textslabs::TextRenderer::composable_shader_source();
    assert!(imported_textslabs_shader == original_textslabs_shader, "Imported textslabs shader does not match original!");

    let imported_images_shader = include_str!("src/shaders/keru_images.slang");
    let original_images_shader = keru_images::ImageRenderer::composable_shader_source();
    assert!(imported_images_shader == original_images_shader, "Imported keru_images shader does not match original!");
}
