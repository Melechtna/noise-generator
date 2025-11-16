use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    /// Master volume (0.0–1.0)
    pub volume: f32,

    /// Band toggles
    pub enable_low: bool,
    pub enable_mid: bool,
    pub enable_high: bool,

    /// Smoothing factor for noise blending (0.9–0.9999)
    pub alpha: f32,

    /// Frequency bands in Hz (low, mid, high)
    pub band_low: [f32; 2],
    pub band_mid: [f32; 2],
    pub band_high: [f32; 2],

    /// Per-band gain multipliers (0.0–10.0)
    pub band_volume_low: f32,
    pub band_volume_mid: f32,
    pub band_volume_high: f32,

    /// Noise seed (0–65535)
    pub seed: u64,

    /// When true, ignores `seed` and generates a random seed each run
    pub random: bool,

    /// Bass enhancement strength (0.0–10.0)
    #[serde(default)]
    pub bass_boost: f32,
}

// Default config generated on first run
impl Default for Config {
    fn default() -> Self {
        Self {
            volume: 1.0,
            alpha: 0.996,

            enable_low: true,
            enable_mid: true,
            enable_high: true,

            band_low: [10.0, 500.0],
            band_mid: [60.0, 1000.0],
            band_high: [100.0, 10000.0],

            band_volume_low: 1.0,
            band_volume_mid: 0.5,
            band_volume_high: 0.2,

            seed: 0,
            random: true,
            bass_boost: 0.0,
        }
    }
}

impl Config {
    /// Clamp all fields to sane operating ranges and enforce band ordering
    pub fn sanitize(&mut self) {
        self.volume = self.volume.clamp(0.0, 1.0);
        self.alpha = self.alpha.clamp(0.9, 0.9999);

        #[inline]
        fn clamp_pair(p: &mut [f32; 2], lo: f32, hi: f32) {
            p[0] = p[0].clamp(lo, hi);
            p[1] = p[1].clamp(lo, hi);
            if p[0] > p[1] {
                p.swap(0, 1);
            }
        }

        // clamp each band's min and max
        clamp_pair(&mut self.band_low, 1.0, 1000.0);
        clamp_pair(&mut self.band_mid, 1.0, 5000.0);
        clamp_pair(&mut self.band_high, 1.0, 10000.0);

        // Prevents inverse band ranges
        if self.band_low[1] > self.band_mid[0] {
            self.band_mid[0] = self.band_low[1].clamp(1.0, 5000.0);
            if self.band_mid[0] > self.band_mid[1] {
                self.band_mid[1] = self.band_mid[0];
            }
        }
        if self.band_mid[1] > self.band_high[0] {
            self.band_high[0] = self.band_mid[1].clamp(1.0, 10000.0);
            if self.band_high[0] > self.band_high[1] {
                self.band_high[1] = self.band_high[0];
            }
        }

        // per-band volume + bass boost
        self.band_volume_low = self.band_volume_low.clamp(0.0, 10.0);
        self.band_volume_mid = self.band_volume_mid.clamp(0.0, 10.0);
        self.band_volume_high = self.band_volume_high.clamp(0.0, 10.0);
        self.bass_boost = self.bass_boost.clamp(0.0, 10.0);

        // seed range
        self.seed = self.seed.min(65535);
    }
}

//Create the config
pub fn load_or_create_config<P: AsRef<std::path::Path>>(path: P) -> Result<Config> {
    use std::fs;
    let path = path.as_ref();

    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).ok();
    }

    let mut cfg = if let Ok(content) = fs::read_to_string(path) {
        toml::from_str::<Config>(&content).map_err(anyhow::Error::from)?
    } else {
        let cfg = Config::default();
        let toml = toml::to_string_pretty(&cfg)?;
        fs::write(path, toml)?;
        println!("Created default config at {}", path.display());
        cfg
    };

    cfg.sanitize();
    Ok(cfg)
}
