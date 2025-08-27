use crate::modes::{Mode, NUM_MODES};

pub const T60_DECAY_FACTOR: f32 = -6.91; // -ln(1000) for 60dB decay
pub struct ModalResonator {
    b0: [f32; NUM_MODES],
    a1: [f32; NUM_MODES],
    a2: [f32; NUM_MODES],
    y1: [f32; NUM_MODES],
    y2: [f32; NUM_MODES],
    amplitudes: [f32; NUM_MODES],
    sample_rate_inv: f32,
    omega_factor: f32,
    decay_factor: f32,
}

impl ModalResonator {
    pub fn new() -> Self {
        let sample_rate = 44100.0;
        let sample_rate_inv = 1.0 / sample_rate;
        Self {
            b0: [0.0; NUM_MODES],
            a1: [0.0; NUM_MODES],
            a2: [0.0; NUM_MODES],
            y1: [0.0; NUM_MODES],
            y2: [0.0; NUM_MODES],
            amplitudes: [0.0; NUM_MODES],
            sample_rate_inv,
            omega_factor: 2.0 * std::f32::consts::PI * sample_rate_inv,
            decay_factor: T60_DECAY_FACTOR * sample_rate_inv,
        }
    }

    pub fn set_modes(&mut self, modes: &[Mode; NUM_MODES]) {
        for i in 0..NUM_MODES {
            let omega = self.omega_factor * modes[i].frequency;
            let r = (self.decay_factor / modes[i].decay).exp();
            self.a1[i] = -2.0 * r * omega.cos();
            self.a2[i] = r * r;
            self.b0[i] = modes[i].frequency * self.sample_rate_inv;
            self.amplitudes[i] = modes[i].amplitude;
        }
    }

    pub fn process(&mut self, input: f32) -> f32 {
        let mut result = 0.0;

        for i in 0..NUM_MODES {
            let y = self.b0[i] * input - self.a1[i] * self.y1[i] - self.a2[i] * self.y2[i];
            self.y2[i] = self.y1[i];
            self.y1[i] = y;
            result += y * self.amplitudes[i];
        }

        result
    }

    pub fn reset(&mut self) {
        self.y1.fill(0.0);
        self.y2.fill(0.0);
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate_inv = 1.0 / sample_rate;
        self.omega_factor = 2.0 * std::f32::consts::PI * self.sample_rate_inv;
        self.decay_factor = T60_DECAY_FACTOR * self.sample_rate_inv;
    }
}
