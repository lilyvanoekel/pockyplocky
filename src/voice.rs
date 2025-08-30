use std::sync::Arc;

use nih_plug::prelude::*;

use crate::{
    constants::MAX_BLOCK_SIZE,
    envelope::Envelope,
    exciter::Exciter,
    modes::ModeCalculator,
    params::{ParamBuffers, PockyplockyParams},
    resonator::ModalResonator,
    wave_folder::WaveFolder,
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
    pub calculator: ModeCalculator,
    pub resonator: ModalResonator,
    pub exciter: Exciter,
    pub envelope: Envelope,
    pub wave_folder: WaveFolder,
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
            calculator: ModeCalculator::new(params.clone()),
            resonator: ModalResonator::new(),
            exciter: Exciter::new(params.clone()),
            envelope: Envelope::new(),
            wave_folder: WaveFolder::new(),
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.resonator.set_sample_rate(sample_rate);
        self.exciter.set_sample_rate(sample_rate);
        self.envelope.set_sample_rate(sample_rate);
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
        // let velocity_normalized = velocity.sqrt();
        let decay = self.params.decay.value();

        self.resonator.reset();
        self.calculator.set_frequency(frequency, decay);
        let modes = self.calculator.get_modes();
        self.resonator.set_modes(modes);
        let mut max_decay_time = 0.0;
        for mode in modes {
            if mode.decay > max_decay_time {
                max_decay_time = mode.decay;
            }
        }
        self.total_duration = (self.sample_rate * max_decay_time) as usize;

        self.active = true;

        self.exciter.start();
        self.envelope.start();
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

        let gain_buffer = param_buffers.get_gain_buffer();
        let exciter_block = self.exciter.process_block(block_len, param_buffers);
        let envelope_block = self.envelope.process_block(block_len);

        for i in 0..block_len {
            let envelope_value = envelope_block[i];
            let filtered_noise = self.resonator.process(exciter_block[i]);
            let voice_sample = filtered_noise * envelope_value;
            output[i] = voice_sample;
        }

        // Apply wave folding if enabled
        if self.params.wave_folder_enabled.value() {
            let amount = self.params.wave_folder_amount.value();
            self.wave_folder.set_amount(amount);
            let folded_output = self
                .wave_folder
                .process_block(&output[..block_len], block_len);
            for i in 0..block_len {
                output[i] = folded_output[i];
            }
        }

        // Apply gain as final volume control
        for i in 0..block_len {
            output[i] *= gain_buffer[i];
        }

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
