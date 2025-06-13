fn main() {
    #[cfg(feature = "espidf")]
    embuild::espidf::sysenv::output();
    
    // Compile Slint UI files with configuration optimized for e-paper
    slint_build::compile_with_config(
        "ui/chat.slint",
        slint_build::CompilerConfiguration::new()
            .embed_resources(slint_build::EmbedResourcesKind::EmbedForSoftwareRenderer)
            .with_sdf_fonts(true), // Enable SDF fonts for better text rendering
    )
    .expect("Failed to compile Slint UI");
}