pub struct ModalResonator {
    b0: f32,
    a1: f32,
    a2: f32,
    y1: f32,
    y2: f32,
    sample_rate: f32,
}

impl ModalResonator {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            b0: 0.0,
            a1: 0.0,
            a2: 0.0,
            y1: 0.0,
            y2: 0.0,
            sample_rate,
        }
    }

    pub fn set_mode(&mut self, frequency: f32, t60: f32) {
        if frequency <= 0.0 || t60 <= 0.0 {
            self.b0 = 0.0;
            self.a1 = 0.0;
            self.a2 = 0.0;
            return;
        }

        let omega = 2.0 * std::f32::consts::PI * frequency / self.sample_rate;
        let r = (-6.91 / (t60 * self.sample_rate)).exp();
        self.a1 = -2.0 * r * omega.cos();
        self.a2 = r * r;
        self.b0 = frequency / self.sample_rate;
    }

    pub fn process(&mut self, input: f32) -> f32 {
        let y = self.b0 * input - self.a1 * self.y1 - self.a2 * self.y2;
        self.y2 = self.y1;
        self.y1 = y;
        y
    }

    pub fn reset(&mut self) {
        self.y1 = 0.0;
        self.y2 = 0.0;
    }

    pub fn set_sample_rate(&mut self, sr: f32) {
        self.sample_rate = sr;
    }
}

pub struct ModalFilter {
    filters: [ModalResonator; 8],
    amplitudes: [f32; 8],
    sample_rate: f32,
}

impl ModalFilter {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            filters: std::array::from_fn(|_| ModalResonator::new(sample_rate)),
            amplitudes: [1.0, 0.5, 0.3, 0.2, 0.0, 0.0, 0.0, 0.0],
            sample_rate,
        }
    }

    pub fn set_modes(&mut self, modes: &[crate::data::Mode; 8]) {
        for (i, (filter, mode)) in self.filters.iter_mut().zip(modes.iter()).enumerate() {
            filter.set_sample_rate(self.sample_rate);

            if mode.f > 20000.0 {
                self.amplitudes[i] = 0.0;
                filter.set_mode(20000.0, 1.0);
            } else {
                self.amplitudes[i] = mode.amp;
                filter.set_mode(mode.f, mode.t60);
            }
        }
    }

    pub fn process(&mut self, input: f32) -> f32 {
        let mut output = 0.0;

        for (filter, &amplitude) in self.filters.iter_mut().zip(self.amplitudes.iter()) {
            output += filter.process(input) * amplitude;
        }

        output
    }

    pub fn reset(&mut self) {
        for filter in &mut self.filters {
            filter.reset();
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        for filter in &mut self.filters {
            filter.set_sample_rate(sample_rate);
        }
    }
}
