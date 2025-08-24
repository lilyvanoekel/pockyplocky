pub struct ResonantFilter {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
    sample_rate: f32,
}

impl ResonantFilter {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            b0: 0.0,
            b1: 0.0,
            b2: 0.0,
            a1: 0.0,
            a2: 0.0,
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
            sample_rate,
        }
    }

    pub fn set_frequency(&mut self, frequency: f32, q: f32) {
        let w0 = 2.0 * std::f32::consts::PI * frequency / self.sample_rate;
        let cos_w0 = w0.cos();
        let sin_w0 = w0.sin();

        let alpha = sin_w0 / (2.0 * q);

        // Constant peak gain form
        let b0 = q * alpha;
        let b1 = 0.0;
        let b2 = -q * alpha;
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * cos_w0;
        let a2 = 1.0 - alpha;

        // frequency-dependent scaling (6 dB per octave)
        let scale = self.sample_rate / frequency;

        self.b0 = (b0 / a0) * scale;
        self.b1 = (b1 / a0) * scale;
        self.b2 = (b2 / a0) * scale;
        self.a1 = a1 / a0;
        self.a2 = a2 / a0;
    }

    // pub fn set_frequency(&mut self, frequency: f32, q: f32) {
    //     let w0 = 2.0 * std::f32::consts::PI * frequency / self.sample_rate;
    //     let cos_w0 = w0.cos();
    //     let sin_w0 = w0.sin();

    //     let alpha = sin_w0 / (2.0 * q);

    //     // Constant peak gain form
    //     let b0 = q * alpha;
    //     let b1 = 0.0;
    //     let b2 = -q * alpha;
    //     let a0 = 1.0 + alpha;
    //     let a1 = -2.0 * cos_w0;
    //     let a2 = 1.0 - alpha;

    //     self.b0 = b0 / a0;
    //     self.b1 = b1 / a0;
    //     self.b2 = b2 / a0;
    //     self.a1 = a1 / a0;
    //     self.a2 = a2 / a0;
    // }

    pub fn process(&mut self, input: f32) -> f32 {
        let y = self.b0 * input + self.b1 * self.x1 + self.b2 * self.x2
            - self.a1 * self.y1
            - self.a2 * self.y2;

        self.x2 = self.x1;
        self.x1 = input;
        self.y2 = self.y1;
        self.y1 = y;
        y
    }

    pub fn reset(&mut self) {
        self.x1 = 0.0;
        self.x2 = 0.0;
        self.y1 = 0.0;
        self.y2 = 0.0;
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
    }
}
