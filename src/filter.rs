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
        let scale = 0.1 * self.sample_rate / frequency;

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

// Modal synthesis using multiple resonant filters
pub struct ModalFilter {
    filters: [ResonantFilter; 4], // 4 modal filters
    amplitudes: [f32; 4],         // Amplitude for each mode
    sample_rate: f32,
}

impl ModalFilter {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            filters: std::array::from_fn(|_| ResonantFilter::new(sample_rate)),
            amplitudes: [1.0, 0.5, 0.3, 0.2], // Default modal amplitudes
            sample_rate,
        }
    }

    pub fn set_frequency(&mut self, fundamental_freq: f32, q: f32, amplitudes: [f32; 4]) {
        // Xylophone modal frequency ratios (relative to fundamental)
        // Based on: 480, 968, 1424, 2870 Hz
        const MODAL_RATIOS: [f32; 4] = [1.0, 2.017, 2.967, 5.979]; // f0, f1, f2, f3 ratios

        for (i, (filter, &ratio)) in self.filters.iter_mut().zip(MODAL_RATIOS.iter()).enumerate() {
            let modal_freq = fundamental_freq * ratio;
            filter.set_sample_rate(self.sample_rate);

            // Disable modes above 20kHz or set amplitude to 0
            if modal_freq > 20000.0 {
                self.amplitudes[i] = 0.0;
                filter.set_frequency(20000.0, q);
            } else {
                self.amplitudes[i] = amplitudes[i];
                filter.set_frequency(modal_freq, q);
            }
        }
    }

    pub fn set_physics_frequency(
        &mut self,
        fundamental_freq: f32,
        q: f32,
        modes: &[crate::physics::Mode; 4],
    ) {
        for (i, (filter, mode)) in self.filters.iter_mut().zip(modes.iter()).enumerate() {
            let modal_freq = mode.freq as f32;
            filter.set_sample_rate(self.sample_rate);

            // Disable modes above 20kHz or set amplitude to 0
            if modal_freq > 20000.0 {
                self.amplitudes[i] = 0.0;
                filter.set_frequency(20000.0, q);
            } else {
                self.amplitudes[i] = mode.amp as f32;
                filter.set_frequency(modal_freq, q);
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
