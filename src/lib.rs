use nih_plug::prelude::*;
use std::sync::Arc;

mod filter;
mod params;
mod voice;
use params::SinewhiskParams;
use voice::{MAX_BLOCK_SIZE, Voices};

struct Pockyplocky {
    params: Arc<SinewhiskParams>,
    voices: Voices,
}

impl Default for Pockyplocky {
    fn default() -> Self {
        Self {
            params: Arc::new(SinewhiskParams::default()),
            voices: Voices::default(),
        }
    }
}

impl Plugin for Pockyplocky {
    const NAME: &'static str = "Pockyplocky";
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
                                voice.start_time = block_start as u32;
                                voice.trigger = true;
                                voice.silence_counter = 0;

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
                    if !voice.active {
                        continue;
                    }

                    let input = if voice.trigger {
                        voice.trigger = false;
                        1.0
                    } else {
                        0.0
                    };

                    // Process through the resonant filter
                    let filtered_noise = voice.filter.process(input);
                    let envelope_value = voice.envelope_values[value_idx];
                    let voice_sample = filtered_noise * envelope_value * gain_buffer[value_idx];
                    sample += voice_sample;

                    // Count consecutive zero samples for voice termination
                    if voice_sample == 0.0 {
                        voice.silence_counter += 1;

                        if voice.silence_counter > 1000 {
                            context.send_event(NoteEvent::VoiceTerminated {
                                timing: block_end as u32,
                                voice_id: Some(voice.voice_id),
                                channel: voice.channel,
                                note: voice.note,
                            });
                            voice.active = false;
                        }
                    } else {
                        voice.silence_counter = 0;
                    }
                }

                output[0][sample_idx] = sample;
                output[1][sample_idx] = sample;
            }

            block_start = block_end;
            block_end = (block_start + MAX_BLOCK_SIZE).min(num_samples);
        }

        ProcessStatus::Normal
    }
}

impl ClapPlugin for Pockyplocky {
    const CLAP_ID: &'static str = "com.lilyvanoekel.pockyplocky";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("A xylophone synthesizer");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] =
        &[ClapFeature::Instrument, ClapFeature::Synthesizer];
}

nih_export_clap!(Pockyplocky);
