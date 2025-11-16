pub struct Biquad {
    pub(crate) a0: f32,
    pub(crate) a1: f32,
    pub(crate) a2: f32,
    pub(crate) b0: f32,
    pub(crate) b1: f32,
    pub(crate) b2: f32,
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
}

impl Biquad {
    pub fn new() -> Self {
        Self {
            a0: 1.0,
            a1: 0.0,
            a2: 0.0,
            b0: 1.0,
            b1: 0.0,
            b2: 0.0,
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
        }
    }

    pub fn update_bandpass(&mut self, sample_rate: f32, low_hz: f32, high_hz: f32) {
        let center = (low_hz * high_hz).sqrt();
        let omega = 2.0 * std::f32::consts::PI * center / sample_rate;
        let alpha = omega.sin() / (2.0 * (center / (high_hz - low_hz)));
        let cos_w = omega.cos();

        self.b0 = alpha;
        self.b1 = 0.0;
        self.b2 = -alpha;
        self.a0 = 1.0 + alpha;
        self.a1 = -2.0 * cos_w;
        self.a2 = 1.0 - alpha;
        self.normalize();
    }

    fn normalize(&mut self) {
        let inv = 1.0 / self.a0;
        self.b0 *= inv;
        self.b1 *= inv;
        self.b2 *= inv;
        self.a1 *= inv;
        self.a2 *= inv;
        self.a0 = 1.0;
    }

    pub fn process(&mut self, input: f32) -> f32 {
        let out = self.b0 * input + self.b1 * self.x1 + self.b2 * self.x2
            - self.a1 * self.y1
            - self.a2 * self.y2;
        self.x2 = self.x1;
        self.x1 = input;
        self.y2 = self.y1;
        self.y1 = out;
        out
    }
}
