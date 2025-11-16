// Dummy wrapper for Desktop
#[cfg(not(target_os = "android"))]
fn main() -> anyhow::Result<()> {
    noise_generator::run_app()
}

#[cfg(target_os = "android")]
fn main() {
}
