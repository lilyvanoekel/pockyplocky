use nih_plug::prelude::*;

pub const MAX_BLOCK_SIZE: usize = 64;

// // Resonant filter for pinged filter effect
// pub struct ResonantFilter {
//     // Filter coefficients
//     b0: f32,
//     b1: f32,
//     a1: f32,
//     a2: f32,
//     // State variables
//     y1: f32,
//     y2: f32,
//     sample_rate: f32,
// }

// impl ResonantFilter {
//     pub fn new(sample_rate: f32) -> Self {
//         Self {
//             b0: 0.0,
//             b1: 0.0,
//             a1: 0.0,
//             a2: 0.0,
//             y1: 0.0,
//             y2: 0.0,
//             sample_rate,
//         }
//     }

//     pub fn set_frequency(&mut self, frequency: f32, resonance: f32) {
//         let omega_0 = frequency * 2.0 * std::f32::consts::PI;
//         let alpha = (omega_0 / self.sample_rate).sin() / (2.0 * resonance);
//         let cosw0 = (omega_0 / self.sample_rate).cos();

//         self.b0 = alpha;
//         self.b1 = 0.0;
//         self.a1 = -2.0 * cosw0;
//         self.a2 = 1.0 - alpha;
//     }

//     pub fn process(&mut self, input: f32) -> f32 {
//         let yn = self.b0 * input - self.a1 * self.y1 - self.a2 * self.y2;
//         self.y2 = self.y1;
//         self.y1 = yn;
//         yn
//     }

//     pub fn reset(&mut self) {
//         self.y1 = 0.0;
//         self.y2 = 0.0;
//     }
// }

pub struct ResonantFilter {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
    sample_rate: f32,
}

impl ResonantFilter {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            b0: 0.0,
            b1: 0.0,
            b2: 0.0,
            a1: 0.0,
            a2: 0.0,
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
            sample_rate,
        }
    }

    pub fn set_frequency(&mut self, frequency: f32, q: f32) {
        let w0 = 2.0 * std::f32::consts::PI * frequency / self.sample_rate;
        let cos_w0 = w0.cos();
        let sin_w0 = w0.sin();

        let alpha = sin_w0 / (2.0 * q);

        // Constant peak gain form
        let b0 = q * alpha;
        let b1 = 0.0;
        let b2 = -q * alpha;
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * cos_w0;
        let a2 = 1.0 - alpha;

        self.b0 = b0 / a0;
        self.b1 = b1 / a0;
        self.b2 = b2 / a0;
        self.a1 = a1 / a0;
        self.a2 = a2 / a0;
    }

    pub fn process(&mut self, input: f32) -> f32 {
        let y = self.b0 * input + self.b1 * self.x1 + self.b2 * self.x2
            - self.a1 * self.y1
            - self.a2 * self.y2;

        self.x2 = self.x1;
        self.x1 = input;
        self.y2 = self.y1;
        self.y1 = y;
        y
    }

    pub fn reset(&mut self) {
        self.x1 = 0.0;
        self.x2 = 0.0;
        self.y1 = 0.0;
        self.y2 = 0.0;
    }
}

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
    pub phase: f32,
    pub phase_delta: f32,
    pub filter: ResonantFilter,
    pub start_time: u32,
    pub key_held: bool,
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
            phase: 0.0,
            phase_delta: 0.0,
            filter: ResonantFilter::new(44100.0), // Will be set properly when voice starts
            start_time: 0,
            key_held: false,
        }
    }
}

pub const NUM_VOICES: usize = 16;

pub struct Voices {
    voices: [Voice; NUM_VOICES],
    next_internal_voice_id: u64,
}

impl Default for Voices {
    fn default() -> Self {
        Self {
            voices: std::array::from_fn(|_| Voice::default()),
            next_internal_voice_id: 0,
        }
    }
}

impl Voices {
    /// Find a free voice slot
    pub fn find_free_slot(&self) -> Option<usize> {
        self.voices.iter().position(|voice| !voice.active)
    }

    /// Find the oldest voice slot (lowest internal_voice_id)
    pub fn find_oldest_slot(&self) -> Option<usize> {
        self.voices
            .iter()
            .enumerate()
            .filter(|(_, voice)| voice.active)
            .min_by_key(|(_, voice)| voice.internal_voice_id)
            .map(|(idx, _)| idx)
    }

    /// Initialize a voice slot with the given data
    pub fn init_voice(
        &mut self,
        slot: usize,
        voice_id: i32,
        channel: u8,
        note: u8,
        internal_voice_id: u64,
        velocity_sqrt: f32,
        amp_envelope: Smoother<f32>,
    ) {
        let voice = &mut self.voices[slot];
        voice.active = true;
        voice.voice_id = voice_id;
        voice.channel = channel;
        voice.note = note;
        voice.internal_voice_id = internal_voice_id;
        voice.velocity_sqrt = velocity_sqrt;
        voice.releasing = false;
        voice.amp_envelope = amp_envelope;
    }

    /// Deactivate a voice slot
    pub fn deactivate_voice(&mut self, slot: usize) {
        self.voices[slot].active = false;
    }

    /// Get voice data for a specific slot (for debugging/logging)
    pub fn get_voice_info(&self, slot: usize) -> Option<(i32, u8, u8)> {
        let voice = &self.voices[slot];
        if voice.active {
            Some((voice.voice_id, voice.channel, voice.note))
        } else {
            None
        }
    }

    /// Check if a voice should be terminated (releasing and envelope at 0)
    pub fn should_terminate_voice(&self, slot: usize) -> bool {
        let voice = &self.voices[slot];
        voice.active && voice.releasing && voice.amp_envelope.previous_value() == 0.0
    }

    /// Start a new voice with the given voice ID. If all voices are currently in use, the oldest
    /// voice will be stolen. Returns the slot index of the new voice.
    pub fn start_voice(
        &mut self,
        context: &mut impl ProcessContext<crate::SineWhisk>,
        sample_offset: u32,
        voice_id: Option<i32>,
        channel: u8,
        note: u8,
    ) -> usize {
        let actual_voice_id = voice_id.unwrap_or_else(|| compute_fallback_voice_id(note, channel));
        self.next_internal_voice_id = self.next_internal_voice_id.wrapping_add(1);

        match self.find_free_slot() {
            Some(free_voice_idx) => {
                // Initialize with default values, will be set properly in the calling code
                self.init_voice(
                    free_voice_idx,
                    actual_voice_id,
                    channel,
                    note,
                    self.next_internal_voice_id,
                    1.0,
                    Smoother::none(),
                );
                free_voice_idx
            }
            None => {
                let oldest_slot = self.find_oldest_slot().unwrap();

                {
                    let oldest_voice_info = self.get_voice_info(oldest_slot).unwrap();
                    context.send_event(NoteEvent::VoiceTerminated {
                        timing: sample_offset,
                        voice_id: Some(oldest_voice_info.0),
                        channel: oldest_voice_info.1,
                        note: oldest_voice_info.2,
                    });
                }

                self.deactivate_voice(oldest_slot);
                self.init_voice(
                    oldest_slot,
                    actual_voice_id,
                    channel,
                    note,
                    self.next_internal_voice_id,
                    1.0,
                    Smoother::none(),
                );
                oldest_slot
            }
        }
    }

    /// Start the release process for one or more voice by changing their amplitude envelope. If
    /// `voice_id` is not provided, then this will terminate all matching voices.
    pub fn start_release_for_voices(
        &mut self,
        sample_rate: f32,
        voice_id: Option<i32>,
        channel: u8,
        note: u8,
        amp_release_ms: f32,
    ) {
        for voice in &mut self.voices {
            if !voice.active {
                continue;
            }

            let matches_voice_id = voice_id == Some(voice.voice_id);
            let matches_note = channel == voice.channel && note == voice.note;

            if matches_voice_id || matches_note {
                voice.releasing = true;
                voice.amp_envelope.style = SmoothingStyle::Exponential(amp_release_ms);
                voice.amp_envelope.set_target(sample_rate, 0.0);

                // If this targetted a single voice ID, we're done here. Otherwise there may be
                // multiple overlapping voices as we enabled support for that in the
                // `PolyModulationConfig`.
                if voice_id.is_some() {
                    return;
                }
            }
        }
    }

    /// Immediately terminate one or more voice, removing it from the pool and informing the host
    /// that the voice has ended. If `voice_id` is not provided, then this will terminate all
    /// matching voices.
    pub fn choke_voices(
        &mut self,
        context: &mut impl ProcessContext<crate::SineWhisk>,
        sample_offset: u32,
        voice_id: Option<i32>,
        channel: u8,
        note: u8,
    ) {
        let mut voices_to_terminate = [false; NUM_VOICES];
        for (voice_idx, voice) in self.voices.iter().enumerate() {
            if !voice.active {
                continue;
            }

            let matches_voice_id = voice_id == Some(voice.voice_id);
            let matches_note = channel == voice.channel && note == voice.note;

            if matches_voice_id || matches_note {
                context.send_event(NoteEvent::VoiceTerminated {
                    timing: sample_offset,
                    // Notice how we always send the terminated voice ID here
                    voice_id: Some(voice.voice_id),
                    channel: voice.channel,
                    note: voice.note,
                });
                voices_to_terminate[voice_idx] = true;

                if voice_id.is_some() {
                    break;
                }
            }
        }
        // Deactivate the voices after the loop to avoid borrow checker issues
        for (voice_idx, &should_terminate) in voices_to_terminate.iter().enumerate() {
            if should_terminate {
                self.voices[voice_idx].active = false;
            }
        }
    }

    /// Reset the voice data to initial state
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Get a reference to all voices for iteration
    pub fn voices(&self) -> &[Voice] {
        &self.voices
    }

    /// Get a mutable reference to all voices for iteration
    pub fn voices_mut(&mut self) -> &mut [Voice] {
        &mut self.voices
    }
}

// Compute a voice ID in case the host doesn't provide them. Polyphonic modulation will not work in
// this case, but playing notes will.
const fn compute_fallback_voice_id(note: u8, channel: u8) -> i32 {
    note as i32 | ((channel as i32) << 16)
}
