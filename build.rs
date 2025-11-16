fn main() {
    // compile Slint
    slint_build::compile("src/ui/ui.slint").unwrap();

    // Windows-only: embed icon via winres
    #[cfg(target_os = "windows")]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("src/ui/icons/icon.ico");
        res.compile().unwrap();
    }
}
