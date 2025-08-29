use std::sync::Arc;

use nih_plug::prelude::*;
use rand::Rng;
use rand_pcg::Pcg32;

use crate::{
    constants::MAX_BLOCK_SIZE,
    params::{ParamBuffers, PockyplockyParams},
};

pub struct Exciter {
    params: Arc<PockyplockyParams>,
    sample_rate: f32,
    output_buffer: [f32; MAX_BLOCK_SIZE],
    noise_envelope: Smoother<f32>,
    noise_envelope_values: [f32; MAX_BLOCK_SIZE],
    trigger: f32,
    prng: Pcg32,
}

impl Exciter {
    pub fn new(params: Arc<PockyplockyParams>) -> Self {
        Self {
            params,
            sample_rate: 44100.0,
            output_buffer: [0.0; MAX_BLOCK_SIZE],
            noise_envelope: Smoother::none(),
            noise_envelope_values: [0.0; MAX_BLOCK_SIZE],
            trigger: 0.0,
            prng: Pcg32::new(12345, 67890),
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
    }

    pub fn start(&mut self) {
        self.noise_envelope.style = SmoothingStyle::Exponential(self.params.noise_decay.value());
        self.noise_envelope.reset(1.0);
        self.noise_envelope.set_target(self.sample_rate, 0.0);
        self.trigger = if self.params.click.value() { 1.0 } else { 0.0 };
    }

    pub fn process_block(&mut self, block_len: usize, param_buffers: &ParamBuffers) -> &[f32] {
        let noise_level_buffer = param_buffers.get_noise_level_buffer();

        self.noise_envelope
            .next_block(&mut self.noise_envelope_values[..block_len], block_len);

        for i in 0..block_len {
            let noise_envelope_value = self.noise_envelope_values[i];
            let noise_sample =
                self.prng.gen_range(-1.0..=1.0) * noise_envelope_value * noise_level_buffer[i];

            self.output_buffer[i] = self.trigger + noise_sample;

            self.trigger = 0.0;
        }

        &self.output_buffer[..block_len]
    }
}
