use nih_plug::prelude::*;
use std::sync::Arc;

mod constants;
mod modal_synth;
mod params;
mod voice;
mod voice_manager;

use constants::MAX_BLOCK_SIZE;
use params::PockyplockyParams;
use voice_manager::VoiceManager;

use crate::params::ParamBuffers;

struct Pockyplocky {
    params: Arc<PockyplockyParams>,
    param_buffers: ParamBuffers,
    voices: VoiceManager,
}

impl Default for Pockyplocky {
    fn default() -> Self {
        let params = Arc::new(PockyplockyParams::default());
        Self {
            params: params.clone(),
            param_buffers: ParamBuffers::new(params.clone()),
            voices: VoiceManager::new(params),
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
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        let sample_rate = buffer_config.sample_rate;
        self.voices.set_sample_rate(sample_rate);
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

                                voice.start(
                                    voice.voice_id,
                                    voice.channel,
                                    voice.note,
                                    voice.internal_voice_id,
                                    velocity,
                                );
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

            self.param_buffers.process_block(block_len);

            // Process all voices
            for voice in self.voices.voices_mut() {
                if !voice.active {
                    continue;
                }

                voice.process_block(block_start, block_len, &self.param_buffers, output);

                // Check for voice termination
                if voice.is_finished() {
                    context.send_event(NoteEvent::VoiceTerminated {
                        timing: block_end as u32,
                        voice_id: Some(voice.voice_id),
                        channel: voice.channel,
                        note: voice.note,
                    });
                    voice.active = false;
                }
            }

            block_start = block_end;
            block_end = (block_start + MAX_BLOCK_SIZE).min(num_samples);
        }

        ProcessStatus::Normal
    }
}

impl ClapPlugin for Pockyplocky {
    const CLAP_ID: &'static str = "com.lilyvanoekel.pockyplocky";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("For making plocky sounds");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] =
        &[ClapFeature::Instrument, ClapFeature::Synthesizer];
}

nih_export_clap!(Pockyplocky);
