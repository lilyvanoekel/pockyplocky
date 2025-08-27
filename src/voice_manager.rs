use std::sync::Arc;

use crate::{params::PockyplockyParams, voice::Voice};
use nih_plug::prelude::*;

pub const NUM_VOICES: usize = 16;

pub struct VoiceManager {
    voices: [Voice; NUM_VOICES],
    next_internal_voice_id: u64,
}

impl VoiceManager {
    pub fn new(params: Arc<PockyplockyParams>) -> Self {
        Self {
            voices: std::array::from_fn(|_| Voice::new(params.clone())),
            next_internal_voice_id: 0,
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        for voice in &mut self.voices {
            voice.set_sample_rate(sample_rate);
        }
    }

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
    ) {
        let voice = &mut self.voices[slot];
        voice.active = true;
        voice.voice_id = voice_id;
        voice.channel = channel;
        voice.note = note;
        voice.internal_voice_id = internal_voice_id;
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

    /// Start a new voice with the given voice ID. If all voices are currently in use, the oldest
    /// voice will be stolen. Returns the slot index of the new voice.
    pub fn start_voice(
        &mut self,
        context: &mut impl ProcessContext<crate::Pockyplocky>,
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
                );
                oldest_slot
            }
        }
    }

    /// Immediately terminate one or more voice, removing it from the pool and informing the host
    /// that the voice has ended. If `voice_id` is not provided, then this will terminate all
    /// matching voices.
    pub fn choke_voices(
        &mut self,
        context: &mut impl ProcessContext<crate::Pockyplocky>,
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
        self.next_internal_voice_id = 0;
        for v in &mut self.voices {
            v.reset();
        }
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
