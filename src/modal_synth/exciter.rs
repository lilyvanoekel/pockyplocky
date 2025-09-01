use rand::Rng;
use rand_pcg::Pcg32;
use std::sync::Arc;

use crate::{
    constants::DEFAULT_SAMPLE_RATE,
    modal_synth::envelope::Envelope,
    params::{ParamBuffers, PockyplockyParams},
};

pub struct HannBurst {
    len: usize,
    pos: usize,
    scale: f32,
    inv_len: f32,
}

impl HannBurst {
    pub fn new() -> Self {
        Self {
            len: 0,
            pos: 0,
            scale: 0.0,
            inv_len: 0.0,
        }
    }

    pub fn start(
        &mut self,
        sample_rate: f32,
        fundamental: f32,
        gain: f32,
        hardness: f32,
        velocity: f32,
    ) {
        let period_s = 1.0 / fundamental;
        let hardness_factor = hardness * velocity;
        let cycles = 0.4 - (hardness_factor * 0.36);
        self.len = ((period_s * cycles * sample_rate).round() as usize).max(3);
        self.scale = gain / (self.len as f32).sqrt();
        self.pos = 0;
        self.inv_len = 1.0 / (self.len as f32 - 1.0);
    }

    pub fn process(&mut self) -> f32 {
        if self.pos < self.len {
            let normalized_pos = self.pos as f32 * self.inv_len;
            let v = 0.5 * (1.0 - (2.0 * std::f32::consts::PI * normalized_pos).cos()) * self.scale;
            self.pos += 1;
            v
        } else {
            0.0
        }
    }

    pub fn reset(&mut self) {
        self.scale = 0.0;
        self.inv_len = 0.0;
        self.len = 0;
        self.pos = 0;
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
    velocity_sqrt: f32,
}

impl Exciter {
    pub fn new(params: Arc<PockyplockyParams>) -> Self {
        Self {
            params,
            sample_rate: DEFAULT_SAMPLE_RATE,
            breath_envelope: Envelope::new(),
            trigger: 0.0,
            hann: HannBurst::new(),
            prng: Pcg32::new(12345, 67890),
            render_noise: false,
            velocity_sqrt: 0.0,
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.breath_envelope.set_sample_rate(sample_rate);
    }

    pub fn reset(&mut self) {
        self.breath_envelope.reset();
        self.trigger = 0.0;
        self.hann.reset();
        self.render_noise = false;
        self.velocity_sqrt = 0.0;
    }

    pub fn start(&mut self, fundamental: f32, velocity: f32) {
        self.breath_envelope
            .set_attack_time(self.params.breath_attack.value());
        self.breath_envelope
            .set_attack_curve(self.params.breath_attack_shape.value());
        self.breath_envelope
            .set_decay_time(self.params.breath_decay.value());
        self.breath_envelope
            .set_decay_curve(self.params.breath_decay_shape.value());
        self.breath_envelope.start();

        self.velocity_sqrt = velocity.sqrt();
        self.trigger = if self.params.strike.value() {
            self.velocity_sqrt
        } else {
            0.0
        };

        if self.params.mallet.value() {
            self.hann.start(
                self.sample_rate,
                fundamental,
                self.velocity_sqrt,
                self.params.mallet_hardness.value(),
                velocity,
            );
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
                let noise_sample = self.prng.gen_range(-1.0..=1.0)
                    * envelope_values[i]
                    * noise_level_buffer[i]
                    * self.velocity_sqrt;

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
                output[i] += self.hann.process();
            }
        }
    }
}
