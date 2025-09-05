use crate::{
    constants::DEFAULT_SAMPLE_RATE,
    modal_synth::modes::{Mode, NUM_MODES},
};
use wide::f32x8;

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
        let sample_rate_inv = 1.0 / DEFAULT_SAMPLE_RATE;
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

    #[allow(clippy::needless_range_loop)]
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
        let input_vec = f32x8::splat(input);
        let b0_vec = f32x8::from(self.b0);
        let a1_vec = f32x8::from(self.a1);
        let a2_vec = f32x8::from(self.a2);
        let y1_vec = f32x8::from(self.y1);
        let y2_vec = f32x8::from(self.y2);
        let amp_vec = f32x8::from(self.amplitudes);

        let y_vec = b0_vec * input_vec - a1_vec * y1_vec - a2_vec * y2_vec;

        // Update state
        let new_y2 = y1_vec;
        let new_y1 = y_vec;

        // Store back to arrays
        self.y2 = new_y2.into();
        self.y1 = new_y1.into();

        // Multiply by amplitudes and sum
        let result_vec = y_vec * amp_vec;
        result_vec.reduce_add()
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
