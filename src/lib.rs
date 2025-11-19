use anyhow::{Context, Result};
use rand::{Rng, SeedableRng, rngs::SmallRng};
use slint::{CloseRequestResponse, ComponentHandle};
use std::sync::{Arc, Mutex};

use std::path::PathBuf;
#[allow(unused_imports)]
use std::sync::OnceLock;

mod bass_boost;
mod biquad;
mod config;
mod dsp;

use config::{Config, load_or_create_config};
use dsp::init_stream;

slint::include_modules!();

/// Android specific setups
#[cfg(target_os = "android")]
static ANDROID_FILES_DIR: OnceLock<PathBuf> = OnceLock::new();

/// Cross-platform: resolve the path to `config.toml`.
fn config_path() -> PathBuf {
    #[cfg(target_os = "android")]
    {
        if let Some(dir) = ANDROID_FILES_DIR.get() {
            return dir.join("config.toml");
        }
        // Last resort fallback (shouldn't happen if android_main set it)
        return PathBuf::from("/data/local/tmp/config.toml");
    }

    #[cfg(not(target_os = "android"))]
    {
        // Use OS-native locations:
        //  - Linux: ~/.config/noise-generator/config.toml
        //  - macOS: ~/Library/Application Support/Noise Generator/config.toml
        //  - Windows: %APPDATA%\noise-generator\config.toml
        use directories::ProjectDirs;
        if let Some(pd) = ProjectDirs::from("io", "melechtna", "Noise Generator") {
            let dir = pd.config_dir();
            let _ = std::fs::create_dir_all(dir);
            return dir.join("config.toml");
        }
        // Fallback if ProjectDirs fails
        let mut home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));
        home.push(".config");
        home.push("noise-generator");
        let _ = std::fs::create_dir_all(&home);
        home.join("config.toml")
    }
}

/// Snapshot all UI state into `cfg`, sanitize, and write config.toml
fn flush_ui_to_config(ui: &RootUI, cfg: &mut Config) {
    // top-level
    cfg.volume = ui.get_volume();
    cfg.alpha = ui.get_alpha();
    cfg.bass_boost = ui.get_bass_boost();
    cfg.enable_low = ui.get_enable_low();
    cfg.enable_mid = ui.get_enable_mid();
    cfg.enable_high = ui.get_enable_high();
    cfg.random = ui.get_random_seed();

    // Only take seed from UI when manual
    if !cfg.random {
        cfg.seed = ui.get_seed().round().clamp(0.0, 65535.0) as u64;
    }

    // per-band volumes
    cfg.band_volume_low = ui.get_band_volume_low();
    cfg.band_volume_mid = ui.get_band_volume_mid();
    cfg.band_volume_high = ui.get_band_volume_high();

    // band ranges
    cfg.band_low = [ui.get_band_low_lo(), ui.get_band_low_hi()];
    cfg.band_mid = [ui.get_band_mid_lo(), ui.get_band_mid_hi()];
    cfg.band_high = [ui.get_band_high_lo(), ui.get_band_high_hi()];

    // keep things sane, then persist
    cfg.sanitize();
    if let Ok(toml) = toml::to_string_pretty(&*cfg) {
        let _ = std::fs::write(config_path(), toml);
    }

    ui.set_volume(cfg.volume);
    ui.set_alpha(cfg.alpha);
    ui.set_bass_boost(cfg.bass_boost);

    ui.set_enable_low(cfg.enable_low);
    ui.set_enable_mid(cfg.enable_mid);
    ui.set_enable_high(cfg.enable_high);

    ui.set_band_volume_low(cfg.band_volume_low);
    ui.set_band_volume_mid(cfg.band_volume_mid);
    ui.set_band_volume_high(cfg.band_volume_high);

    ui.set_band_low_lo(cfg.band_low[0]);
    ui.set_band_low_hi(cfg.band_low[1]);
    ui.set_band_mid_lo(cfg.band_mid[0]);
    ui.set_band_mid_hi(cfg.band_mid[1]);
    ui.set_band_high_lo(cfg.band_high[0]);
    ui.set_band_high_hi(cfg.band_high[1]);
}

// Shared app logic — used by both desktop and Android
pub fn run_app() -> Result<()> {
    let path = config_path();
    let mut config = load_or_create_config(&path)?;
    config.sanitize();
    let shared_cfg = Arc::new(Mutex::new(config));

    // Random or fixed seed
    let seed_value = {
        let cfg = shared_cfg.lock().unwrap();
        if cfg.random {
            let s: u16 = rand::rng().random();
            println!("Random seed: {s}");
            s as u64
        } else {
            cfg.seed.min(65535)
        }
    };

    let runtime_seed = Arc::new(Mutex::new(seed_value));
    let mut rng_seeded = SmallRng::seed_from_u64(seed_value);

    // Start audio
    let (audio_device, mute_ctrl) =
        init_stream(shared_cfg.clone(), &mut rng_seeded).context("Failed to start audio output")?;
    std::mem::forget(audio_device);

    println!("Noise generator running – edit config.toml or use UI");

    #[cfg(target_os = "linux")]
    if let Err(err) = slint::set_xdg_app_id("io.melechtna.noise-generator") {
        eprintln!("warning: unable to set XDG app id: {err}");
    }

    // Create UI
    let ui = RootUI::new().unwrap();

    ui.window().set_size(slint::LogicalSize::new(460.0, 720.0));

    // Load config into UI
    {
        let cfg = shared_cfg.lock().unwrap();
        ui.set_volume(cfg.volume);
        ui.set_alpha(cfg.alpha);
        ui.set_bass_boost(cfg.bass_boost);

        ui.set_enable_low(cfg.enable_low);
        ui.set_enable_mid(cfg.enable_mid);
        ui.set_enable_high(cfg.enable_high);
        ui.set_random_seed(cfg.random);

        ui.set_band_volume_low(cfg.band_volume_low);
        ui.set_band_volume_mid(cfg.band_volume_mid);
        ui.set_band_volume_high(cfg.band_volume_high);

        ui.set_band_low_lo(cfg.band_low[0]);
        ui.set_band_low_hi(cfg.band_low[1]);
        ui.set_band_mid_lo(cfg.band_mid[0]);
        ui.set_band_mid_hi(cfg.band_mid[1]);
        ui.set_band_high_lo(cfg.band_high[0]);
        ui.set_band_high_hi(cfg.band_high[1]);

        ui.set_seed(seed_value as f32);
    }

    // CENTRALIZED CONFIG WRITE
    {
        let sc = shared_cfg.clone();
        let ui_weak_cfg = ui.as_weak();
        let ui_weak_num = ui.as_weak();

        let prev_random = Arc::new(Mutex::new({
            let cfg = sc.lock().unwrap();
            cfg.random
        }));
        let started_random = Arc::new(Mutex::new({
            let cfg = sc.lock().unwrap();
            cfg.random
        }));
        let runtime_seed = runtime_seed.clone();

        ui.on_config_changed(move || {
            if let Some(ui) = ui_weak_cfg.upgrade() {
                let mut cfg = sc.lock().unwrap();

                let was_random = *prev_random.lock().unwrap();
                let now_random = ui.get_random_seed();

                if was_random && !now_random {
                    let session_started_random = *started_random.lock().unwrap();
                    let new_seed: u64 = if session_started_random {
                        *runtime_seed.lock().unwrap()
                    } else {
                        let stored = cfg.seed;
                        if stored != 0 {
                            stored
                        } else {
                            *runtime_seed.lock().unwrap()
                        }
                    };
                    ui.set_seed(new_seed as f32);
                }

                *prev_random.lock().unwrap() = now_random;

                flush_ui_to_config(&ui, &mut cfg);
            }
        });

        ui.on_number_text_committed(move |name: slint::SharedString, s: slint::SharedString| {
            if let Some(ui) = ui_weak_num.upgrade() {
                let field = name.as_str();

                fn parse_num(raw: &str) -> Option<f32> {
                    let mut t = raw.trim().to_lowercase();
                    if t.is_empty() {
                        return None;
                    }

                    let mut mul = 1.0_f32;
                    if t.ends_with('%') {
                        t.pop();
                    }
                    if t.ends_with("khz") {
                        t.truncate(t.len() - 3);
                        mul = 1000.0;
                    } else if t.ends_with('k') {
                        t.pop();
                        mul = 1000.0;
                    }
                    let v: f32 = t.trim().parse().ok()?;
                    Some(v * mul)
                }

                let v = parse_num(s.as_str());

                match (field, v) {
                    ("volume_pct", Some(pct)) => ui.set_volume((pct / 100.0).clamp(0.0, 1.0)),
                    ("alpha", Some(a)) => ui.set_alpha(a.clamp(0.9, 0.9999)),

                    ("band_low_lo", Some(x)) => ui.set_band_low_lo(x.clamp(1.0, 1000.0)),
                    ("band_low_hi", Some(x)) => ui.set_band_low_hi(x.clamp(1.0, 1000.0)),
                    ("band_mid_lo", Some(x)) => ui.set_band_mid_lo(x.clamp(1.0, 5000.0)),
                    ("band_mid_hi", Some(x)) => ui.set_band_mid_hi(x.clamp(1.0, 5000.0)),
                    ("band_high_lo", Some(x)) => ui.set_band_high_lo(x.clamp(1.0, 10000.0)),
                    ("band_high_hi", Some(x)) => ui.set_band_high_hi(x.clamp(1.0, 10000.0)),

                    ("band_volume_low", Some(x)) => ui.set_band_volume_low(x.clamp(0.0, 10.0)),
                    ("band_volume_mid", Some(x)) => ui.set_band_volume_mid(x.clamp(0.0, 10.0)),
                    ("band_volume_high", Some(x)) => ui.set_band_volume_high(x.clamp(0.0, 10.0)),

                    ("bass_boost", Some(x)) => ui.set_bass_boost(x.clamp(0.0, 10.0)),

                    ("seed", Some(x)) => ui.set_seed(x.round().clamp(0.0, 65535.0)),
                    _ => { /* no-op on parse error / unknown field */ }
                }

                ui.invoke_config_changed();
            }
        });
    }

    // Volume text entry
    {
        let ui_weak = ui.as_weak();
        ui.on_volume_text_committed(move |s: slint::SharedString| {
            if let Some(ui) = ui_weak.upgrade() {
                let raw = s.trim();
                if let Ok(mut pct) = raw.parse::<f32>() {
                    pct = pct.clamp(0.0, 100.0);
                    ui.set_volume(pct / 100.0);
                    ui.invoke_config_changed();
                } else {
                    let v = ui.get_volume();
                    ui.set_volume(v);
                }
            }
        });
    }

    // Play/pause
    ui.set_internal_playing(!mute_ctrl.is_muted());
    {
        let mc = mute_ctrl.clone();
        let ui_weak = ui.as_weak();
        ui.on_toggle_play(move || {
            let now_muted = mc.toggle();
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_internal_playing(!now_muted);
            }
            println!("Audio {}", if now_muted { "muted" } else { "unmuted" });
        });
    }

    // Close
    ui.window().on_close_requested(|| {
        let _ = slint::quit_event_loop();
        CloseRequestResponse::HideWindow
    });

    ui.run().unwrap();
    println!("UI exited, shutting down...");
    Ok(())
}

// Desktop entry point (called from bin)
#[cfg(not(target_os = "android"))]
pub fn run_desktop() -> Result<()> {
    run_app()
}

// ANDROID entry point (cargo-apk looks for this symbol in the cdylib)
#[cfg(target_os = "android")]
#[allow(improper_ctypes_definitions)]
#[unsafe(no_mangle)]
pub extern "C" fn android_main(app: slint::android::AndroidApp) {
    use android_logger::Config as LoggerConfig;
    use log::LevelFilter;

    android_logger::init_once(
        LoggerConfig::default()
            .with_tag("NoiseGenerator")
            .with_max_level(LevelFilter::Trace),
    );

    // Resolve and store the app's internal files dir (…/files)
    let files_dir = app
        .internal_data_path()
        .unwrap_or_else(|| PathBuf::from("/data/local/tmp"));
    let _ = std::fs::create_dir_all(&files_dir);
    let _ = ANDROID_FILES_DIR.set(files_dir);

    // Initialize Slint Android backend (handles surface & event loop)
    slint::android::init(app).expect("slint::android::init failed");

    if let Err(e) = run_app() {
        log::error!("App crashed: {e}");
    }
}
