use nih_plug::prelude::*;
use rand::Rng;
use rand_pcg::Pcg32;
use std::sync::Arc;

use crate::{
    constants::{DEFAULT_SAMPLE_RATE, MAX_BLOCK_SIZE},
    params::{ParamBuffers, PockyplockyParams},
};

pub struct HannBurst {
    window: [f32; MAX_BLOCK_SIZE],
    len: usize,
    pos: usize,
    gain: f32,
}

impl HannBurst {
    pub fn new(gain: f32) -> Self {
        Self {
            window: [0.0; MAX_BLOCK_SIZE],
            len: 0,
            pos: 0,
            gain,
        }
    }

    pub fn trigger(&mut self, sample_rate: f32, fundamental: f32) {
        let period_s = 1.0 / fundamental;
        let cycles = 1.0;
        let len = (period_s * cycles * sample_rate).round() as usize;
        self.len = len.min(MAX_BLOCK_SIZE);

        let scale = self.gain / (self.len as f32).sqrt();

        for n in 0..self.len {
            self.window[n] = 0.5
                * (1.0 - (2.0 * std::f32::consts::PI * n as f32 / (self.len as f32 - 1.0)))
                * scale;
        }
        for n in self.len..MAX_BLOCK_SIZE {
            self.window[n] = 0.0;
        }
        self.pos = 0;
    }

    pub fn next_sample(&mut self) -> f32 {
        if self.pos < self.len {
            let v = self.window[self.pos];
            self.pos += 1;
            v
        } else {
            0.0
        }
    }
}

pub struct Exciter {
    params: Arc<PockyplockyParams>,
    sample_rate: f32,
    noise_envelope: Smoother<f32>,
    noise_envelope_values: [f32; MAX_BLOCK_SIZE],
    trigger: f32,
    hann: HannBurst,
    prng: Pcg32,
}

impl Exciter {
    pub fn new(params: Arc<PockyplockyParams>) -> Self {
        Self {
            params,
            sample_rate: DEFAULT_SAMPLE_RATE,
            noise_envelope: Smoother::none(),
            noise_envelope_values: [0.0; MAX_BLOCK_SIZE],
            trigger: 0.0,
            hann: HannBurst::new(0.4),
            prng: Pcg32::new(12345, 67890),
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
    }

    pub fn start(&mut self, fundamental: f32) {
        self.noise_envelope.style = SmoothingStyle::Exponential(self.params.breath_decay.value());
        self.noise_envelope.reset(1.0);
        self.noise_envelope.set_target(self.sample_rate, 0.0);
        self.trigger = if self.params.strike.value() { 1.0 } else { 0.0 };

        if self.params.mallet.value() {
            self.hann.trigger(self.sample_rate, fundamental);
        }
    }

    pub fn process_block(
        &mut self,
        output: &mut [f32],
        block_len: usize,
        param_buffers: &ParamBuffers,
    ) {
        let noise_level_buffer = param_buffers.get_noise_level_buffer();

        self.noise_envelope
            .next_block(&mut self.noise_envelope_values[..block_len], block_len);

        for i in 0..block_len {
            let noise_sample = self.prng.gen_range(-1.0..=1.0)
                * self.noise_envelope_values[i]
                * noise_level_buffer[i];

            let hann_sample = if self.params.mallet.value() {
                self.hann.next_sample()
            } else {
                0.0
            };

            output[i] = self.trigger + noise_sample + hann_sample;
            self.trigger = 0.0;
        }
    }
}
