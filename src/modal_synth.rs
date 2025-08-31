use std::sync::Arc;

mod envelope;
mod exciter;
mod modes;
mod resonator;
mod wave_folder;

use crate::{
    modal_synth::exciter::Exciter,
    modal_synth::modes::ModeCalculator,
    modal_synth::resonator::ModalResonator,
    modal_synth::wave_folder::WaveFolder,
    params::{ParamBuffers, PockyplockyParams},
};

pub struct ModalSynth {
    params: Arc<PockyplockyParams>,
    pub velocity_sqrt: f32,
    pub sample_rate: f32,
    pub calculator: ModeCalculator,
    pub resonator: ModalResonator,
    pub exciter: Exciter,
    pub wave_folder: WaveFolder,
}

impl ModalSynth {
    pub fn new(params: Arc<PockyplockyParams>) -> Self {
        Self {
            params: params.clone(),
            velocity_sqrt: 0.0,
            sample_rate: 44100.0,
            calculator: ModeCalculator::new(params.clone()),
            resonator: ModalResonator::new(),
            exciter: Exciter::new(params.clone()),
            wave_folder: WaveFolder::new(),
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.resonator.set_sample_rate(sample_rate);
        self.exciter.set_sample_rate(sample_rate);
    }

    pub fn start(&mut self, frequency: f32, velocity: f32, decay: f32) -> f32 {
        self.velocity_sqrt = velocity.sqrt();
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

        self.exciter.start();
        max_decay_time
    }

    pub fn process_block(
        &mut self,
        output: &mut [f32],
        block_len: usize,
        param_buffers: &ParamBuffers,
    ) {
        let gain_buffer = param_buffers.get_gain_buffer();
        self.exciter.process_block(output, block_len, param_buffers);

        for i in 0..block_len {
            let filtered_noise = self.resonator.process(output[i]);
            let voice_sample = filtered_noise;
            output[i] = voice_sample;
        }

        if self.params.wave_folder_enabled.value() {
            let amount = self.params.wave_folder_amount.value();
            self.wave_folder.set_amount(amount);
            for i in 0..block_len {
                output[i] = self.wave_folder.process(output[i]);
            }
        }

        for i in 0..block_len {
            output[i] *= gain_buffer[i];
        }
    }
}
