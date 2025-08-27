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

const NUM_MODES: usize = 8;

pub struct ModalFilter {
    filters: [ModalResonator; NUM_MODES],
    amplitudes: [f32; NUM_MODES],
    sample_rate: f32,
}

impl ModalFilter {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            filters: std::array::from_fn(|_| ModalResonator::new(sample_rate)),
            amplitudes: [0.0; NUM_MODES],
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

    pub fn set_frequency(&mut self, fundamental_freq: f32, decay: f32) {
        // Base modal frequencies (for A4 = 440Hz)
        const BASE_FREQS: [f32; 8] = [
            488.42, 1414.28, 2587.91, 2848.72, 4270.11, 4883.01, 5156.85, 5834.95,
        ];
        const MODAL_AMPS: [f32; 8] = [0.595, 0.115, 0.083, 1.000, 0.133, 0.044, 0.180, 0.338];
        const MODAL_DECAYS: [f32; 8] = [0.461, 0.085, 0.063, 0.044, 0.018, 0.075, 0.023, 0.022];

        // const BASE_FREQS: [f32; 16] = [
        //     484.07, 1451.23, 2863.93, 4254.90, 4874.31, 5189.46, 5526.33, 5863.21, 6678.23,
        //     7308.51, 8427.80, 8992.88, 9699.23, 10253.45, 11459.68, 12644.17,
        // ];

        // const MODAL_AMPS: [f32; 16] = [
        //     0.917, 0.058, 1.000, 0.244, 0.070, 0.087, 0.123, 0.384, 0.061, 0.043, 0.058, 0.052,
        //     0.026, 0.018, 0.060, 0.040,
        // ];

        // const MODAL_DECAYS: [f32; 16] = [
        //     0.476, 0.077, 0.040, 0.020, 0.081, 0.021, 0.036, 0.015, 0.041, 0.020, 0.020, 0.018,
        //     0.015, 0.028, 0.016, 0.012,
        // ];

        // const BASE_FREQS: [f32; 8] = [
        //     488.42, 1450.0, 2595.0, 2860.0, 4285.0, 4895.0, 5180.0, 5870.0,
        // ];

        // const MODAL_AMPS: [f32; 8] = [
        //     1.0,  // fundamental strong
        //     0.3,  // 2nd mode moderate
        //     0.2,  // 3rd
        //     0.25, // 4th
        //     0.1,  // 5th low sparkle
        //     0.05, // 6th very low
        //     0.1,  // 7th soft sparkle
        //     0.08, // 8th faint shimmer
        // ];

        // Scale factor based on fundamental frequency
        let scale_factor = fundamental_freq / 488.42;
        let decay_factor = decay / 0.461;

        for i in 0..NUM_MODES {
            let freq = BASE_FREQS[i] * scale_factor;
            let amp = MODAL_AMPS[i];
            let decay = MODAL_DECAYS[i] * decay_factor;

            self.filters[i].set_sample_rate(self.sample_rate);

            // Disable modes above 20kHz or set amplitude to 0
            if freq > 20000.0 {
                self.amplitudes[i] = 0.0;
                self.filters[i].set_mode(20000.0, 1.0);
            } else {
                self.amplitudes[i] = amp;
                self.filters[i].set_mode(freq, decay);
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
