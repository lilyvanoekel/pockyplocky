use nih_plug::prelude::*;
use rand::Rng;
use rand_pcg::Pcg32;

use crate::filter::ModalFilter;

pub const MAX_BLOCK_SIZE: usize = 64;

pub struct Voice {
    pub active: bool,
    pub voice_id: i32,
    pub channel: u8,
    pub note: u8,
    pub internal_voice_id: u64,
    pub velocity_sqrt: f32,
    pub releasing: bool,
    pub amp_envelope: Smoother<f32>,
    pub envelope_values: [f32; MAX_BLOCK_SIZE],
    pub filter: ModalFilter,
    pub start_time: u32,
    pub silence_counter: u32, // Count of consecutive samples below threshold
    pub noise_index: usize,   // Current position in noise burst
    pub noise_duration: usize, // Duration of noise burst in samples
    pub sample_rate: f32,     // Current sample rate
    pub total_duration: usize, // Total duration based on longest mode decay time
    pub sample_count: usize,  // Current sample count since start
    pub trigger: bool,
    pub noise_envelope: Smoother<f32>,
    pub noise_envelope_values: [f32; MAX_BLOCK_SIZE],
    pub prng: Pcg32,
}

impl Default for Voice {
    fn default() -> Self {
        Self {
            active: false,
            voice_id: 0,
            channel: 0,
            note: 0,
            internal_voice_id: 0,
            velocity_sqrt: 0.0,
            releasing: false,
            amp_envelope: Smoother::none(),
            envelope_values: [0.0; MAX_BLOCK_SIZE],
            filter: ModalFilter::new(44100.0),
            start_time: 0,
            silence_counter: 0,
            noise_index: 0,
            noise_duration: 0,
            sample_rate: 44100.0,
            total_duration: 0,
            sample_count: 0,
            trigger: false,
            noise_envelope: Smoother::none(),
            noise_envelope_values: [0.0; MAX_BLOCK_SIZE],
            prng: Pcg32::new(12345, 67890),
        }
    }
}

impl Voice {
    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.filter.set_sample_rate(sample_rate);
    }

    pub fn start(
        &mut self,
        voice_id: i32,
        channel: u8,
        note: u8,
        internal_voice_id: u64,
        velocity: f32,
        modes: &[crate::data::Mode; 8],
        material: crate::params::Material,
        noise_decay: f32,
        decay: f32,
    ) {
        self.voice_id = voice_id;
        self.channel = channel;
        self.note = note;
        self.internal_voice_id = internal_voice_id;
        self.velocity_sqrt = velocity.sqrt();
        self.noise_index = 0;
        self.silence_counter = 0;
        self.start_time = 0;
        self.sample_count = 0;
        self.trigger = true;

        let frequency = util::midi_note_to_freq(note);

        let velocity_normalized = velocity.sqrt();
        let max_duration = match material {
            crate::params::Material::Wood => 3.0,
            crate::params::Material::Glass => 6.0,
            crate::params::Material::Metal => 12.0,
        };
        let noise_duration_ms = 2.0 + (velocity_normalized * (max_duration - 2.0));
        self.noise_duration = (self.sample_rate * noise_duration_ms * 0.001) as usize;

        // Calculate total duration based on the longest mode decay time
        let mut max_decay_time = 0.0;
        for mode in modes {
            if mode.t60 > max_decay_time {
                max_decay_time = mode.t60;
            }
        }
        self.total_duration = (self.sample_rate * max_decay_time) as usize;

        // Configure modal filter
        self.filter.reset();
        // self.filter.set_modes(modes);
        self.filter.set_frequency(frequency, decay);

        // Set up envelope
        self.amp_envelope.style = SmoothingStyle::Exponential(10.0);
        self.amp_envelope.reset(0.0);
        self.amp_envelope.set_target(self.sample_rate, 1.0);

        self.noise_envelope.style = SmoothingStyle::Exponential(noise_decay);
        self.noise_envelope.reset(1.0);
        self.noise_envelope.set_target(self.sample_rate, 0.0);

        self.active = true;
    }

    pub fn process_block(
        &mut self,
        gain_buffer: &[f32],
        noise_level_buffer: &[f32],
        block_len: usize,
    ) -> [f32; MAX_BLOCK_SIZE] {
        let mut output = [0.0; MAX_BLOCK_SIZE];

        if !self.active {
            return output;
        }

        // Update envelope
        self.amp_envelope
            .next_block(&mut self.envelope_values[..block_len], block_len);

        // Update noise envelope
        self.noise_envelope
            .next_block(&mut self.noise_envelope_values[..block_len], block_len);

        for i in 0..block_len {
            let envelope_value = self.envelope_values[i];
            let noise_envelope_value = self.noise_envelope_values[i];

            // let input = if self.noise_index < self.noise_duration {
            //     // Get noise sample, loop the table if needed
            //     let noise_sample =
            //         crate::noise::NOISE_BURST[self.noise_index % crate::noise::NOISE_BURST.len()];
            //     self.noise_index += 1;
            //     noise_sample * self.velocity_sqrt * envelope_value
            // } else {
            //     0.0
            // };
            let input = if self.trigger {
                self.trigger = false;
                1.0
            } else {
                0.0
            };

            // Add noise to input (noise level will come from parameter)
            let noise_sample = if self.noise_index < self.noise_duration {
                // Generate noise between -1 and 1 using PRNG
                let noise = self.prng.gen_range(-1.0..=1.0);
                self.noise_index += 1;
                noise
            } else {
                0.0
            };
            let noise_input = noise_sample * noise_envelope_value * noise_level_buffer[i];
            let mixed_input = input + noise_input;

            let filtered_noise = self.filter.process(mixed_input);
            let voice_sample = filtered_noise * gain_buffer[i] * envelope_value;
            output[i] = voice_sample;

            self.sample_count += 1;
        }

        output
    }

    pub fn is_finished(&self) -> bool {
        self.sample_count >= self.total_duration
    }
}
