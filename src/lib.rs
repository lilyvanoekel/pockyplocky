use nih_plug::prelude::*;
use std::sync::Arc;

mod params;
mod voice;

use params::SinewhiskParams;
use voice::{MAX_BLOCK_SIZE, NUM_VOICES, Voices};

struct SineWhisk {
    params: Arc<SinewhiskParams>,
    voices: Voices,
}

impl Default for SineWhisk {
    fn default() -> Self {
        Self {
            params: Arc::new(SinewhiskParams::default()),
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
                                self.voices.velocity_sqrt[s] = velocity.sqrt();
                                self.voices.phase[s] = 0.0;
                                self.voices.phase_delta[s] =
                                    util::midi_note_to_freq(note) / sample_rate;
                                self.voices.amp_envelope[s].style =
                                    SmoothingStyle::Exponential(self.params.amp_attack_ms.value());
                                self.voices.amp_envelope[s].reset(0.0);
                                self.voices.amp_envelope[s].set_target(sample_rate, 1.0);
                            }
                            NoteEvent::NoteOff {
                                timing: _,
                                voice_id,
                                channel,
                                note,
                                velocity: _,
                            } => self.voices.start_release_for_voices(
                                sample_rate,
                                voice_id,
                                channel,
                                note,
                                self.params.amp_release_ms.value(),
                            ),
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
            let mut gain = [0.0; MAX_BLOCK_SIZE];
            self.params.gain.smoothed.next_block(&mut gain, block_len);

            for s in 0..NUM_VOICES {
                self.voices.amp_envelope[s]
                    .next_block(&mut self.voices.envelope_values[s], block_len);
            }

            for (value_idx, sample_idx) in (block_start..block_end).enumerate() {
                let mut sample = 0.0;

                for s in 0..NUM_VOICES {
                    if self.voices.active[s] {
                        let amp = self.voices.velocity_sqrt[s]
                            * gain[value_idx]
                            * self.voices.envelope_values[s][value_idx];

                        // Simple sine wave oscillator
                        let sine_sample = (self.voices.phase[s] * 2.0 * std::f32::consts::PI).sin();

                        sample += amp * sine_sample;

                        // Update phase
                        self.voices.phase[s] += self.voices.phase_delta[s];
                        self.voices.phase[s] %= 1.0;
                    }
                }

                output[0][sample_idx] = sample;
                output[1][sample_idx] = sample;
            }

            let mut voices_to_terminate = [false; NUM_VOICES as usize];
            for (voice_idx, &active) in self.voices.active.iter().enumerate() {
                if active && self.voices.should_terminate_voice(voice_idx) {
                    context.send_event(NoteEvent::VoiceTerminated {
                        timing: block_end as u32,
                        voice_id: Some(self.voices.voice_id[voice_idx]),
                        channel: self.voices.channel[voice_idx],
                        note: self.voices.note[voice_idx],
                    });
                    voices_to_terminate[voice_idx] = true;
                }
            }
            for (voice_idx, &should_terminate) in voices_to_terminate.iter().enumerate() {
                if should_terminate {
                    self.voices.active[voice_idx] = false;
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
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::Instrument,
        ClapFeature::Synthesizer,
        ClapFeature::Stereo,
    ];
}

nih_export_clap!(SineWhisk);
