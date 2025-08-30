use std::sync::Arc;

use nih_plug::prelude::*;

use crate::{
    constants::MAX_BLOCK_SIZE,
    modal_synth::ModalSynth,
    params::{ParamBuffers, PockyplockyParams},
};

pub struct Voice {
    params: Arc<PockyplockyParams>,
    pub active: bool,
    pub voice_id: i32,
    pub channel: u8,
    pub note: u8,
    pub internal_voice_id: u64,
    pub velocity_sqrt: f32,
    pub sample_rate: f32,
    pub total_duration: usize, // Total duration based on longest mode decay time
    pub sample_count: usize,   // Current sample count since start
    pub modal_synth: ModalSynth,
}

impl Voice {
    pub fn new(params: Arc<PockyplockyParams>) -> Self {
        Self {
            params: params.clone(),
            active: false,
            voice_id: 0,
            channel: 0,
            note: 0,
            internal_voice_id: 0,
            velocity_sqrt: 0.0,
            sample_rate: 44100.0,
            total_duration: 0,
            sample_count: 0,
            modal_synth: ModalSynth::new(params),
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.modal_synth.set_sample_rate(sample_rate);
    }

    pub fn start(
        &mut self,
        voice_id: i32,
        channel: u8,
        note: u8,
        internal_voice_id: u64,
        velocity: f32,
    ) {
        self.voice_id = voice_id;
        self.channel = channel;
        self.note = note;
        self.internal_voice_id = internal_voice_id;
        self.velocity_sqrt = velocity.sqrt();
        self.sample_count = 0;
        let frequency = util::midi_note_to_freq(note);
        let decay = self.params.decay.value();
        let max_decay_time = self.modal_synth.start(frequency, velocity, decay);
        self.total_duration = (self.sample_rate * max_decay_time) as usize;
        self.active = true;
    }

    pub fn process_block(
        &mut self,
        block_len: usize,
        param_buffers: &ParamBuffers,
    ) -> [f32; MAX_BLOCK_SIZE] {
        let mut output = [0.0; MAX_BLOCK_SIZE];

        if !self.active {
            return output;
        }

        self.modal_synth
            .process_block(&mut output, block_len, param_buffers);

        self.sample_count += block_len;
        output
    }

    pub fn is_finished(&self) -> bool {
        self.sample_count >= self.total_duration
    }

    pub fn reset(&mut self) {
        *self = Self::new(self.params.clone());
    }
}
