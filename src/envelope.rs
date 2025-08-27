use nih_plug::prelude::*;

use crate::voice::MAX_BLOCK_SIZE;

pub struct Envelope {
    sample_rate: f32,
    envelope: Smoother<f32>,
    envelope_values: [f32; MAX_BLOCK_SIZE],
}

impl Envelope {
    pub fn new() -> Self {
        Self {
            sample_rate: 44100.0,
            envelope: Smoother::none(),
            envelope_values: [0.0; MAX_BLOCK_SIZE],
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
    }

    pub fn start(&mut self) {
        self.envelope.style = SmoothingStyle::Exponential(10.0);
        self.envelope.reset(0.0);
        self.envelope.set_target(self.sample_rate, 1.0);
    }

    pub fn process_block(&mut self, block_len: usize) -> &[f32] {
        self.envelope
            .next_block(&mut self.envelope_values[..block_len], block_len);

        &self.envelope_values[..block_len]
    }
}
