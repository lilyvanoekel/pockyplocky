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

                                // Initialize filter with note frequency and resonance
                                let note_freq = util::midi_note_to_freq(note);
                                voice.filter = ResonantFilter::new(sample_rate);
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

            for (value_idx, sample_idx) in (block_start..block_end).enumerate() {
                let mut sample = 0.0;

                for voice in self.voices.voices_mut() {
                    if voice.active {
                        let input = if voice.key_held {
                            // Generate white noise only while key is held
                            (self.prng.gen_range(0.0..1.0) * 2.0 - 1.0)
                        } else {
                            // No input when key is released, but filter keeps running
                            0.0
                        };

                        // Process through the resonant filter
                        let filtered_noise = voice.filter.process(input);
                        sample += filtered_noise;
                    }
                }

                output[0][sample_idx] = sample;
                output[1][sample_idx] = sample;
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
