fn main() {
    // compile Slint
    slint_build::compile("src/ui/ui.slint").unwrap();

    // embed Windows icon
    #[cfg(target_os = "windows")]
    {
        embed_resource::compile("src/ui/icons/icon.rc");
    }
}
