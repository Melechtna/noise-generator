pub struct BassBoost {
    low_shelf: super::biquad::Biquad,
    gain_db: f32,
    sample_rate: f32,
}

impl BassBoost {
    pub fn new(sample_rate: f32) -> Self {
        let mut low_shelf = super::biquad::Biquad::new();
        low_shelf.update_lowshelf(sample_rate, 100.0, 0.0);
        Self {
            low_shelf,
            gain_db: 0.0,
            sample_rate,
        }
    }

    pub fn set_boost(&mut self, gain_db: f32) {
        self.gain_db = gain_db;
        self.low_shelf
            .update_lowshelf(self.sample_rate, 100.0, gain_db);
    }

    pub fn process(&mut self, input: f32) -> f32 {
        if self.gain_db.abs() < 1e-6 {
            input
        } else {
            self.low_shelf.process(input)
        }
    }
}

pub trait BiquadExt {
    fn update_lowshelf(&mut self, sample_rate: f32, freq: f32, gain_db: f32);
}

impl BiquadExt for super::biquad::Biquad {
    fn update_lowshelf(&mut self, sample_rate: f32, freq: f32, gain_db: f32) {
        let a = 10.0_f32.powf(gain_db / 40.0);
        let omega = 2.0 * std::f32::consts::PI * freq / sample_rate;
        let sin_w = omega.sin();
        let cos_w = omega.cos();
        let beta = (a * a - 1.0).sqrt() / (a * 2.0);

        let b0 = a * ((a + 1.0) - (a - 1.0) * cos_w + beta * sin_w);
        let b1 = 2.0 * a * ((a - 1.0) - (a + 1.0) * cos_w);
        let b2 = a * ((a + 1.0) - (a - 1.0) * cos_w - beta * sin_w);
        let a0 = (a + 1.0) + (a - 1.0) * cos_w + beta * sin_w;
        let a1 = -2.0 * ((a - 1.0) + (a + 1.0) * cos_w);
        let a2 = (a + 1.0) + (a - 1.0) * cos_w - beta * sin_w;

        self.b0 = b0 / a0;
        self.b1 = b1 / a0;
        self.b2 = b2 / a0;
        self.a0 = 1.0;
        self.a1 = a1 / a0;
        self.a2 = a2 / a0;
    }
}
