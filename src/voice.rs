use nih_plug::prelude::*;

pub const NUM_VOICES: usize = 16;
pub const MAX_BLOCK_SIZE: usize = 64;

pub struct Voices {
    /// Whether each voice slot is active
    pub active: [bool; NUM_VOICES],
    /// The voice ID for each voice
    pub voice_id: [i32; NUM_VOICES],
    /// The note's channel for each voice, in `0..16`
    pub channel: [u8; NUM_VOICES],
    /// The note's key/note for each voice, in `0..128`
    pub note: [u8; NUM_VOICES],
    /// The internal voice ID for each voice
    pub internal_voice_id: [u64; NUM_VOICES],
    /// The square root of the note's velocity for each voice
    pub velocity_sqrt: [f32; NUM_VOICES],
    pub releasing: [bool; NUM_VOICES],
    pub amp_envelope: [Smoother<f32>; NUM_VOICES],
    pub envelope_values: [[f32; MAX_BLOCK_SIZE]; NUM_VOICES],
    pub next_internal_voice_id: u64,
    pub phase: [f32; NUM_VOICES],
    pub phase_delta: [f32; NUM_VOICES],
}

impl Default for Voices {
    fn default() -> Self {
        Self {
            active: [false; NUM_VOICES],
            voice_id: [0; NUM_VOICES],
            channel: [0; NUM_VOICES],
            note: [0; NUM_VOICES],
            internal_voice_id: [0; NUM_VOICES],
            velocity_sqrt: [0.0; NUM_VOICES],
            releasing: [false; NUM_VOICES],
            amp_envelope: [0; NUM_VOICES].map(|_| Smoother::none()),
            envelope_values: [[0.0; MAX_BLOCK_SIZE]; NUM_VOICES],
            next_internal_voice_id: 0,
            phase: [0.0; NUM_VOICES],
            phase_delta: [0.0; NUM_VOICES],
        }
    }
}

impl Voices {
    /// Find a free voice slot
    pub fn find_free_slot(&self) -> Option<usize> {
        self.active.iter().position(|&active| !active)
    }

    /// Find the oldest voice slot (lowest internal_voice_id)
    pub fn find_oldest_slot(&self) -> Option<usize> {
        self.active
            .iter()
            .enumerate()
            .filter(|&(_, &active)| active)
            .min_by_key(|&(idx, _)| self.internal_voice_id[idx])
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
        self.active[slot] = true;
        self.voice_id[slot] = voice_id;
        self.channel[slot] = channel;
        self.note[slot] = note;
        self.internal_voice_id[slot] = internal_voice_id;
        self.velocity_sqrt[slot] = velocity_sqrt;
        self.releasing[slot] = false;
        self.amp_envelope[slot] = amp_envelope;
    }

    /// Deactivate a voice slot
    pub fn deactivate_voice(&mut self, slot: usize) {
        self.active[slot] = false;
    }

    /// Get voice data for a specific slot (for debugging/logging)
    pub fn get_voice_info(&self, slot: usize) -> Option<(i32, u8, u8)> {
        if self.active[slot] {
            Some((self.voice_id[slot], self.channel[slot], self.note[slot]))
        } else {
            None
        }
    }

    /// Get active voice indices for efficient iteration (returns array and count)
    pub fn get_active_voice_indices(&self) -> ([usize; NUM_VOICES], usize) {
        let mut active_indices = [usize::MAX; NUM_VOICES];
        let mut active_count = 0;

        for (idx, &active) in self.active.iter().enumerate() {
            if active {
                active_indices[active_count] = idx;
                active_count += 1;
            }
        }

        (active_indices, active_count)
    }

    /// Check if a voice should be terminated (releasing and envelope at 0)
    pub fn should_terminate_voice(&self, slot: usize) -> bool {
        self.active[slot] && self.releasing[slot] && self.amp_envelope[slot].previous_value() == 0.0
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
        for (voice_idx, &active) in self.active.iter().enumerate() {
            if !active {
                continue;
            }

            let matches_voice_id = voice_id == Some(self.voice_id[voice_idx]);
            let matches_note = channel == self.channel[voice_idx] && note == self.note[voice_idx];

            if matches_voice_id || matches_note {
                self.releasing[voice_idx] = true;
                self.amp_envelope[voice_idx].style = SmoothingStyle::Exponential(amp_release_ms);
                self.amp_envelope[voice_idx].set_target(sample_rate, 0.0);

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
        for (voice_idx, &active) in self.active.iter().enumerate() {
            if !active {
                continue;
            }

            let matches_voice_id = voice_id == Some(self.voice_id[voice_idx]);
            let matches_note = channel == self.channel[voice_idx] && note == self.note[voice_idx];

            if matches_voice_id || matches_note {
                context.send_event(NoteEvent::VoiceTerminated {
                    timing: sample_offset,
                    // Notice how we always send the terminated voice ID here
                    voice_id: Some(self.voice_id[voice_idx]),
                    channel: self.channel[voice_idx],
                    note: self.note[voice_idx],
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
                self.active[voice_idx] = false;
            }
        }
    }

    /// Reset the voice data to initial state
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

// Compute a voice ID in case the host doesn't provide them. Polyphonic modulation will not work in
// this case, but playing notes will.
const fn compute_fallback_voice_id(note: u8, channel: u8) -> i32 {
    note as i32 | ((channel as i32) << 16)
}
