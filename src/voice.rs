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
    pub modal_synth2: ModalSynth,
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
            modal_synth: ModalSynth::new(params.clone()),
            modal_synth2: ModalSynth::new(params),
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.modal_synth.set_sample_rate(sample_rate);
        self.modal_synth2.set_sample_rate(sample_rate);
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

        // Calculate detune factors based on percentage
        let detune = self.params.second_voice_detune.value();
        let detune_factor1 = 1.0 - detune * 0.02; // Max 2% detune
        let detune_factor2 = 1.0 + detune * 0.02;

        let max_decay_time = self
            .modal_synth
            .start(frequency * detune_factor1, velocity, decay);

        // Only start second voice if enabled
        if self.params.second_voice_enabled.value() {
            self.modal_synth2
                .start(frequency * detune_factor2, velocity, decay);
        }

        self.total_duration = (self.sample_rate * max_decay_time) as usize;
        self.active = true;
    }

    pub fn process_block(
        &mut self,
        block_start: usize,
        block_len: usize,
        param_buffers: &ParamBuffers,
        output: &mut [&mut [f32]],
    ) {
        let mut buffer = [0.0; MAX_BLOCK_SIZE];

        self.modal_synth
            .process_block(&mut buffer, block_len, param_buffers);

        if self.params.second_voice_enabled.value() {
            let stereo_spread = self.params.second_voice_stereo_spread.value();
            let left_gain = 0.5 - stereo_spread * 0.5;
            let right_gain = 0.5 + stereo_spread * 0.5;

            for i in 0..block_len {
                output[0][block_start + i] += buffer[i] * left_gain;
                output[1][block_start + i] += buffer[i] * right_gain;
            }

            self.modal_synth2
                .process_block(&mut buffer, block_len, param_buffers);

            for i in 0..block_len {
                output[0][block_start + i] += buffer[i] * right_gain;
                output[1][block_start + i] += buffer[i] * left_gain;
            }
        } else {
            for i in 0..block_len {
                output[0][block_start + i] += buffer[i];
                output[1][block_start + i] += buffer[i];
            }
        }

        self.sample_count += block_len;
    }

    pub fn is_finished(&self) -> bool {
        self.sample_count >= self.total_duration
    }

    pub fn reset(&mut self) {
        *self = Self::new(self.params.clone());
    }
}
