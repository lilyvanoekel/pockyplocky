use nih_plug::prelude::*;
use std::sync::Arc;

mod params;
mod voice;
use params::SinewhiskParams;
use rand::Rng;
use rand_pcg::Pcg32;
use voice::{MAX_BLOCK_SIZE, ResonantFilter, Voices};

struct SineWhisk {
    params: Arc<SinewhiskParams>,
    prng: Pcg32,
    voices: Voices,
}

impl Default for SineWhisk {
    fn default() -> Self {
        Self {
            params: Arc::new(SinewhiskParams::default()),
            prng: Pcg32::new(420, 1337),
            voices: Voices::default(),
        }
    }
}

impl Plugin for SineWhisk {
    const NAME: &'static str = "Xylophone";
    const VENDOR: &'static str = "Lily's Nonexistent Company";
    const URL: &'static str = "https://lilyvanoekel.com";
    const EMAIL: &'static str = "why@doyouneed.this";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(2),
        main_output_channels: NonZeroU32::new(2),
        ..AudioIOLayout::const_default()
    }];

    const MIDI_INPUT: MidiConfig = MidiConfig::Basic;
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        _buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        true
    }

    fn reset(&mut self) {
        self.voices.reset();
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let num_samples = buffer.samples();
        let sample_rate = context.transport().sample_rate;
        let output = buffer.as_slice();

        let mut next_event = context.next_event();
        let mut block_start: usize = 0;
        let mut block_end: usize = MAX_BLOCK_SIZE.min(num_samples);

        while block_start < num_samples {
            'events: loop {
                match next_event {
                    Some(event) if (event.timing() as usize) <= block_start => {
                        match event {
                            NoteEvent::NoteOn {
                                timing,
                                voice_id,
                                channel,
                                note,
                                velocity,
                            } => {
                                let s = self
                                    .voices
                                    .start_voice(context, timing, voice_id, channel, note);
                                let voice = &mut self.voices.voices_mut()[s];
                                voice.velocity_sqrt = velocity.sqrt();
                                voice.phase = 0.0;
                                voice.phase_delta = util::midi_note_to_freq(note) / sample_rate;
                                voice.start_time = block_start as u32;
                                voice.key_held = true;

                                // Set up noise burst timing from parameter
                                let noise_burst_duration =
                                    (sample_rate * self.params.noise_burst_ms.value() / 1000.0)
                                        as u32;
                                voice.noise_burst_duration = noise_burst_duration;
                                voice.noise_samples_generated = 0;

                                // Seed the voice's PRNG with a unique seed based on note, channel, and timing
                                let seed = (note as u64)
                                    | ((channel as u64) << 8)
                                    | ((timing as u64) << 16);
                                voice.seed_prng(seed);

                                // Set up very short attack envelope for filter output smoothing
                                voice.amp_envelope.style = SmoothingStyle::Exponential(10.0);
                                voice.amp_envelope.reset(0.0);
                                voice.amp_envelope.set_target(sample_rate, 1.0);

                                // Reset and configure filter for new note
                                let note_freq = util::midi_note_to_freq(note);
                                voice.filter.reset();
                                voice.filter.set_sample_rate(sample_rate); // Update sample rate
                                voice
                                    .filter
                                    .set_frequency(note_freq, self.params.filter_resonance.value());
                            }
                            NoteEvent::NoteOff {
                                timing: _,
                                voice_id,
                                channel,
                                note,
                                velocity: _,
                            } => {
                                // Mark key as released but keep voice active for filter tail
                                for voice in self.voices.voices_mut() {
                                    if voice.active
                                        && voice.channel == channel
                                        && voice.note == note
                                    {
                                        if voice_id.is_none() || voice_id == Some(voice.voice_id) {
                                            voice.key_held = false;
                                        }
                                    }
                                }
                            }
                            NoteEvent::Choke {
                                timing,
                                voice_id,
                                channel,
                                note,
                            } => {
                                self.voices
                                    .choke_voices(context, timing, voice_id, channel, note);
                            }
                            _ => (),
                        };

                        next_event = context.next_event();
                    }
                    Some(event) if (event.timing() as usize) < block_end => {
                        block_end = event.timing() as usize;
                        break 'events;
                    }
                    _ => break 'events,
                }
            }

            output[0][block_start..block_end].fill(0.0);
            output[1][block_start..block_end].fill(0.0);

            let block_len = block_end - block_start;

            // Fill gain buffer with smoothed values
            let mut gain_buffer = [0.0; MAX_BLOCK_SIZE];
            self.params
                .gain
                .smoothed
                .next_block(&mut gain_buffer[..block_len], block_len);

            // Update envelopes for all active voices
            for voice in self.voices.voices_mut() {
                if voice.active {
                    voice
                        .amp_envelope
                        .next_block(&mut voice.envelope_values[..block_len], block_len);
                }
            }

            for (value_idx, sample_idx) in (block_start..block_end).enumerate() {
                let mut sample = 0.0;

                for voice in self.voices.voices_mut() {
                    if voice.active {
                        let input = if voice.noise_samples_generated < voice.noise_burst_duration {
                            // TEMPORARY: Single sample click instead of noise burst

                            let v = match voice.noise_samples_generated {
                                0 => 1.0, // first sample
                                1 => 0.0, // second sample cancels the DC/step
                                _ => 0.0,
                            };
                            voice.noise_samples_generated += 1;
                            v

                            // voice.noise_samples_generated += 1;
                            // input_value

                            // ORIGINAL NOISE BURST CODE (commented out for easy reversal):
                            /*
                            // Generate white noise only during the noise burst period
                            let ramp_samples = (sample_rate * 0.001) as u32; // 1 millisecond ramp
                            let ramp_up_end = ramp_samples.min(voice.noise_burst_duration / 2);
                            let ramp_down_start = voice.noise_burst_duration - ramp_samples;

                            let envelope_value = if voice.noise_samples_generated < ramp_up_end {
                                // Ramp up
                                voice.noise_samples_generated as f32 / ramp_up_end as f32
                            } else if voice.noise_samples_generated >= ramp_down_start {
                                // Ramp down
                                (voice.noise_burst_duration - voice.noise_samples_generated) as f32
                                    / ramp_samples as f32
                            } else {
                                // Full volume in the middle
                                1.0
                            };

                            voice.noise_samples_generated += 1;
                            (voice.prng.gen_range(0.0..1.0) * 2.0 - 1.0) * envelope_value
                            */
                        } else {
                            // No input after noise burst, but filter keeps running
                            0.0
                        };

                        // Process through the resonant filter
                        let filtered_noise = voice.filter.process(input);

                        // Apply envelope to filter output and mute first 2 samples
                        let envelope_value = voice.envelope_values[value_idx];
                        // let mute_factor = if voice.noise_samples_generated <= 2 {
                        //     0.0
                        // } else {
                        //     1.0
                        // };
                        sample += filtered_noise * envelope_value;
                    }
                }

                // Apply smoothed gain parameter to control volume
                output[0][sample_idx] = sample * gain_buffer[value_idx];
                output[1][sample_idx] = sample * gain_buffer[value_idx];
            }

            let mut voices_to_terminate = [false; voice::NUM_VOICES];
            for (voice_idx, voice) in self.voices.voices().iter().enumerate() {
                if voice.active {
                    // Terminate voice after 2 seconds
                    let voice_duration = block_end as u32 - voice.start_time;
                    let two_seconds = (2.0 * sample_rate) as u32;

                    if voice_duration > two_seconds {
                        context.send_event(NoteEvent::VoiceTerminated {
                            timing: block_end as u32,
                            voice_id: Some(voice.voice_id),
                            channel: voice.channel,
                            note: voice.note,
                        });
                        voices_to_terminate[voice_idx] = true;
                    }
                }
            }
            for (voice_idx, &should_terminate) in voices_to_terminate.iter().enumerate() {
                if should_terminate {
                    self.voices.voices_mut()[voice_idx].active = false;
                }
            }

            block_start = block_end;
            block_end = (block_start + MAX_BLOCK_SIZE).min(num_samples);
        }

        ProcessStatus::Normal
    }
}

impl ClapPlugin for SineWhisk {
    const CLAP_ID: &'static str = "com.lilyvanoekel.xylophone";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("A simple xylophone");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] =
        &[ClapFeature::Instrument, ClapFeature::Synthesizer];
}

nih_export_clap!(SineWhisk);
