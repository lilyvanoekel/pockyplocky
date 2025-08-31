use rand::Rng;
use rand_pcg::Pcg32;
use std::sync::Arc;

use crate::{
    constants::{DEFAULT_SAMPLE_RATE, MAX_BLOCK_SIZE},
    modal_synth::envelope::Envelope,
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
    breath_envelope: Envelope,
    trigger: f32,
    hann: HannBurst,
    prng: Pcg32,
    render_noise: bool,
}

impl Exciter {
    pub fn new(params: Arc<PockyplockyParams>) -> Self {
        Self {
            params,
            sample_rate: DEFAULT_SAMPLE_RATE,
            breath_envelope: Envelope::new(),
            trigger: 0.0,
            hann: HannBurst::new(0.4),
            prng: Pcg32::new(12345, 67890),
            render_noise: false,
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.breath_envelope.set_sample_rate(sample_rate);
    }

    pub fn start(&mut self, fundamental: f32) {
        self.breath_envelope
            .set_attack_time(self.params.breath_attack.value());
        self.breath_envelope
            .set_attack_curve(self.params.breath_attack_shape.value());
        self.breath_envelope
            .set_decay_time(self.params.breath_decay.value());
        self.breath_envelope
            .set_decay_curve(self.params.breath_decay_shape.value());
        self.breath_envelope.start();

        self.trigger = if self.params.strike.value() { 1.0 } else { 0.0 };

        if self.params.mallet.value() {
            self.hann.trigger(self.sample_rate, fundamental);
        }

        self.render_noise = self.params.breath_level.value() > 0.0;
    }

    pub fn process_block(
        &mut self,
        output: &mut [f32],
        block_len: usize,
        param_buffers: &ParamBuffers,
    ) {
        let noise_level_buffer = param_buffers.get_noise_level_buffer();
        let envelope_values = self.breath_envelope.process_block(block_len);

        if self.render_noise {
            for i in 0..block_len {
                let noise_sample =
                    self.prng.gen_range(-1.0..=1.0) * envelope_values[i] * noise_level_buffer[i];

                output[i] = noise_sample + self.trigger;
                self.trigger = 0.0;
            }
        } else {
            for i in 0..block_len {
                output[i] = self.trigger;
                self.trigger = 0.0;
            }
        }

        if self.params.mallet.value() {
            for i in 0..block_len {
                output[i] += self.hann.next_sample();
            }
        }
    }
}
