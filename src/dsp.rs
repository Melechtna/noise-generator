use anyhow::Result;
use rand::{Rng, SeedableRng, rngs::SmallRng};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};

use crate::bass_boost::BassBoost;
use crate::biquad::Biquad;
use crate::config::Config;

// -------------------------
// Thread-local DSP State
// -------------------------
std::thread_local! {
    static FILTERS_L: std::cell::RefCell<Option<(Biquad, Biquad, Biquad)>> = std::cell::RefCell::new(None);
    static FILTERS_R: std::cell::RefCell<Option<(Biquad, Biquad, Biquad)>> = std::cell::RefCell::new(None);

    static BASS_BOOST_L: std::cell::RefCell<BassBoost> = std::cell::RefCell::new(BassBoost::new(48000.0));
    static BASS_BOOST_R: std::cell::RefCell<BassBoost> = std::cell::RefCell::new(BassBoost::new(48000.0));

    static NOISE_L: std::cell::RefCell<f32> = std::cell::RefCell::new(0.0);
    static NOISE_R: std::cell::RefCell<f32> = std::cell::RefCell::new(0.0);
}

// -------------------------
// Mute Controller
// -------------------------
#[derive(Clone)]
pub struct MuteController {
    muted: Arc<AtomicBool>,
}

impl MuteController {
    pub fn toggle(&self) -> bool {
        let now = !self.muted.load(Ordering::Relaxed);
        self.muted.store(now, Ordering::Relaxed);
        now
    }
    pub fn is_muted(&self) -> bool {
        self.muted.load(Ordering::Relaxed)
    }
}

// -------------------------
// Init audio stream
// -------------------------
pub fn init_stream(
    shared_cfg: Arc<Mutex<Config>>,
    rng_seeded: &mut SmallRng,
) -> Result<(tinyaudio::OutputDevice, MuteController)> {
    let channels = 2;

    let mut rng = SmallRng::from_rng(rng_seeded);
    let samplerate = 48000.0;

    let params = tinyaudio::OutputDeviceParameters {
        sample_rate: samplerate as usize,
        channels_count: channels,
        channel_sample_count: 1024,
    };

    let muted = Arc::new(AtomicBool::new(false));
    let mute_ctrl = MuteController {
        muted: muted.clone(),
    };

    let device = match tinyaudio::run_output_device(params, move |buffer: &mut [f32]| {
        if muted.load(Ordering::Relaxed) {
            for s in buffer.iter_mut() {
                *s = 0.0;
            }
            return;
        }

        let cfg = shared_cfg.lock().unwrap().clone();
        let alpha = cfg.alpha.clamp(0.9, 0.9999);

        // Ensure filters exist & update coefficients
        for filter_ref in [&FILTERS_L, &FILTERS_R] {
            filter_ref.with(|f| {
                let mut filters = f.borrow_mut();
                if filters.is_none() {
                    let mut low = Biquad::new();
                    let mut mid = Biquad::new();
                    let mut high = Biquad::new();
                    low.update_bandpass(samplerate, cfg.band_low[0], cfg.band_low[1]);
                    mid.update_bandpass(samplerate, cfg.band_mid[0], cfg.band_mid[1]);
                    high.update_bandpass(samplerate, cfg.band_high[0], cfg.band_high[1]);
                    *filters = Some((low, mid, high));
                } else {
                    let (low, mid, high) = &mut *filters.as_mut().unwrap();
                    low.update_bandpass(samplerate, cfg.band_low[0], cfg.band_low[1]);
                    mid.update_bandpass(samplerate, cfg.band_mid[0], cfg.band_mid[1]);
                    high.update_bandpass(samplerate, cfg.band_high[0], cfg.band_high[1]);
                }
            });
        }

        // Bass boost tracks cfg
        BASS_BOOST_L.with(|bb| bb.borrow_mut().set_boost(cfg.bass_boost));
        BASS_BOOST_R.with(|bb| bb.borrow_mut().set_boost(cfg.bass_boost));

        // Noise + filters
        NOISE_L.with(|bl| {
            NOISE_R.with(|br| {
                let mut brown_l = *bl.borrow();
                let mut brown_r = *br.borrow();

                for frame in buffer.chunks_mut(channels) {
                    let white_l = rng.random_range(-1.0..1.0);
                    let white_r = rng.random_range(-1.0..1.0);

                    brown_l = (1.0 - alpha) * white_l + alpha * brown_l;
                    brown_r = (1.0 - alpha) * white_r + alpha * brown_r;

                    let (low_l, mid_l, high_l) = FILTERS_L.with(|f| {
                        let mut filters = f.borrow_mut();
                        let (low_f, mid_f, high_f) = &mut *filters.as_mut().unwrap();
                        (
                            if cfg.enable_low {
                                low_f.process(brown_l) * cfg.band_volume_low
                            } else {
                                0.0
                            },
                            if cfg.enable_mid {
                                mid_f.process(brown_l) * cfg.band_volume_mid
                            } else {
                                0.0
                            },
                            if cfg.enable_high {
                                high_f.process(brown_l) * cfg.band_volume_high
                            } else {
                                0.0
                            },
                        )
                    });

                    let (low_r, mid_r, high_r) = FILTERS_R.with(|f| {
                        let mut filters = f.borrow_mut();
                        let (low_f, mid_f, high_f) = &mut *filters.as_mut().unwrap();
                        (
                            if cfg.enable_low {
                                low_f.process(brown_r) * cfg.band_volume_low
                            } else {
                                0.0
                            },
                            if cfg.enable_mid {
                                mid_f.process(brown_r) * cfg.band_volume_mid
                            } else {
                                0.0
                            },
                            if cfg.enable_high {
                                high_f.process(brown_r) * cfg.band_volume_high
                            } else {
                                0.0
                            },
                        )
                    });

                    let low_boost_l = if cfg.enable_low {
                        BASS_BOOST_L.with(|bb| bb.borrow_mut().process(low_l))
                    } else {
                        0.0
                    };

                    let low_boost_r = if cfg.enable_low {
                        BASS_BOOST_R.with(|bb| bb.borrow_mut().process(low_r))
                    } else {
                        0.0
                    };

                    let mixed_l = (mid_l + high_l) * 0.5 + low_boost_l;
                    let mixed_r = (mid_r + high_r) * 0.5 + low_boost_r;

                    frame[0] = (mixed_l * cfg.volume).clamp(-1.0, 1.0);
                    frame[1] = (mixed_r * cfg.volume).clamp(-1.0, 1.0);
                }

                *bl.borrow_mut() = brown_l;
                *br.borrow_mut() = brown_r;
            });
        });
    }) {
        Ok(dev) => dev,
        Err(e) => return Err(anyhow::anyhow!("tinyaudio failed: {}", e)),
    };

    Ok((device, mute_ctrl))
}
