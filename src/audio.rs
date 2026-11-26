//! Source-shaped, platform-neutral PCM mixer for the Street Chaves audio host.
//!
//! The Pascal implementation uses 48 voices, one active voice per sample, oldest-voice
//! stealing, 44.1 kHz signed 16-bit stereo PCM, looped music, and saturating summation.
//! This module keeps those rules testable without requiring an audio device or SDL.

use std::sync::Arc;

pub const MIXER_RATE_HZ: u32 = 44_100;
pub const MIXER_CHANNELS: usize = 2;
pub const MAX_AUDIO_VOICES: usize = 48;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SampleId(pub usize);

#[derive(Debug)]
struct Voice {
    sample_id: SampleId,
    pcm: Arc<[i16]>,
    position_frames: usize,
    looped: bool,
    sequence: u64,
}

/// Deterministic PCM voice mixer. It deliberately knows nothing about SDL or files.
#[derive(Debug)]
pub struct AudioMixer {
    voices: [Option<Voice>; MAX_AUDIO_VOICES],
    next_sequence: u64,
    callback_buffer: Vec<i16>,
}

impl Default for AudioMixer {
    fn default() -> Self {
        Self {
            voices: std::array::from_fn(|_| None),
            next_sequence: 0,
            callback_buffer: Vec::new(),
        }
    }
}

impl AudioMixer {
    /// Start a sample from frame zero.
    ///
    /// Re-triggering the same sample stops its previous voice. If all 48 voices are occupied,
    /// the oldest active voice is stolen, matching the Pascal mixer.
    pub fn play(&mut self, sample_id: SampleId, pcm: Arc<[i16]>, looped: bool) -> bool {
        if pcm.len() < MIXER_CHANNELS {
            return false;
        }

        for voice in &mut self.voices {
            if voice
                .as_ref()
                .map(|active| active.sample_id == sample_id)
                .unwrap_or(false)
            {
                *voice = None;
            }
        }

        let slot = self
            .voices
            .iter()
            .position(Option::is_none)
            .unwrap_or_else(|| {
                self.voices
                    .iter()
                    .enumerate()
                    .filter_map(|(index, voice)| {
                        voice.as_ref().map(|active| (index, active.sequence))
                    })
                    .min_by_key(|(_, sequence)| *sequence)
                    .map(|(index, _)| index)
                    .unwrap_or(0)
            });

        self.next_sequence = self.next_sequence.wrapping_add(1);
        self.voices[slot] = Some(Voice {
            sample_id,
            pcm,
            position_frames: 0,
            looped,
            sequence: self.next_sequence,
        });
        true
    }

    /// Stop every voice belonging to one sample.
    pub fn stop(&mut self, sample_id: SampleId) {
        for voice in &mut self.voices {
            if voice
                .as_ref()
                .map(|active| active.sample_id == sample_id)
                .unwrap_or(false)
            {
                *voice = None;
            }
        }
    }

    /// Number of currently active voices, useful for diagnostics and tests.
    pub fn active_voice_count(&self) -> usize {
        self.voices.iter().filter(|voice| voice.is_some()).count()
    }

    /// Mix interleaved stereo samples into the caller's buffer.
    ///
    /// The caller owns the output buffer. An odd trailing sample is zeroed because the mixer
    /// operates on complete stereo frames only.
    pub fn mix_into(&mut self, output: &mut [i16]) {
        mix_voices_into(&mut self.voices, output);
    }

    /// Reuse a persistent buffer in SDL's real-time callback to avoid per-callback allocation.
    pub(crate) fn render_callback_buffer(&mut self, sample_count: usize) -> &[i16] {
        self.callback_buffer.resize(sample_count, 0);
        mix_voices_into(&mut self.voices, &mut self.callback_buffer);
        &self.callback_buffer
    }
}

fn mix_voices_into(voices: &mut [Option<Voice>; MAX_AUDIO_VOICES], output: &mut [i16]) {
    output.fill(0);
    let frame_count = output.len() / MIXER_CHANNELS;

    for frame in 0..frame_count {
        let mut left = 0_i32;
        let mut right = 0_i32;

        for voice_slot in voices.iter_mut() {
            let mut ended = false;

            if let Some(voice) = voice_slot.as_mut() {
                let sample_frames = voice.pcm.len() / MIXER_CHANNELS;

                if sample_frames == 0 {
                    ended = true;
                } else {
                    if voice.position_frames >= sample_frames {
                        if voice.looped {
                            voice.position_frames = 0;
                        } else {
                            ended = true;
                        }
                    }

                    if !ended {
                        let offset = voice.position_frames * MIXER_CHANNELS;
                        left += i32::from(voice.pcm[offset]);
                        right += i32::from(voice.pcm[offset + 1]);
                        voice.position_frames += 1;

                        if voice.position_frames >= sample_frames {
                            if voice.looped {
                                voice.position_frames = 0;
                            } else {
                                ended = true;
                            }
                        }
                    }
                }
            }

            if ended {
                *voice_slot = None;
            }
        }

        output[frame * MIXER_CHANNELS] = left.clamp(i16::MIN as i32, i16::MAX as i32) as i16;
        output[frame * MIXER_CHANNELS + 1] = right.clamp(i16::MIN as i32, i16::MAX as i32) as i16;
    }

    // A non-stereo-aligned tail must never leak stale samples to the device.
    if output.len() % MIXER_CHANNELS != 0 {
        let last = output.len() - 1;
        output[last] = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pcm(frames: &[(i16, i16)]) -> Arc<[i16]> {
        let samples: Vec<i16> = frames
            .iter()
            .flat_map(|(left, right)| [*left, *right])
            .collect();
        Arc::from(samples)
    }

    #[test]
    fn different_samples_mix_concurrently() {
        let mut mixer = AudioMixer::default();
        assert!(mixer.play(SampleId(1), pcm(&[(100, -100)]), false));
        assert!(mixer.play(SampleId(2), pcm(&[(250, -250)]), false));

        let mut output = [0; 2];
        mixer.mix_into(&mut output);

        assert_eq!(output, [350, -350]);
        assert_eq!(mixer.active_voice_count(), 0);
    }

    #[test]
    fn replaying_the_same_sample_cuts_and_restarts_its_voice() {
        let mut mixer = AudioMixer::default();
        let sample = pcm(&[(100, -100), (200, -200), (300, -300)]);

        assert!(mixer.play(SampleId(7), sample.clone(), false));
        let mut first_frame = [0; 2];
        mixer.mix_into(&mut first_frame);
        assert_eq!(first_frame, [100, -100]);

        assert!(mixer.play(SampleId(7), sample, false));
        let mut restarted = [0; 4];
        mixer.mix_into(&mut restarted);

        assert_eq!(restarted, [100, -100, 200, -200]);
        assert_eq!(mixer.active_voice_count(), 1);
    }

    #[test]
    fn looped_samples_repeat_without_an_empty_frame() {
        let mut mixer = AudioMixer::default();
        assert!(mixer.play(SampleId(3), pcm(&[(1, 10), (2, 20)]), true));

        let mut output = [0; 6];
        mixer.mix_into(&mut output);

        assert_eq!(output, [1, 10, 2, 20, 1, 10]);
        assert_eq!(mixer.active_voice_count(), 1);
    }

    #[test]
    fn summed_pcm_saturates_to_signed_sixteen_bit() {
        let mut mixer = AudioMixer::default();
        assert!(mixer.play(SampleId(1), pcm(&[(20_000, -20_000)]), false));
        assert!(mixer.play(SampleId(2), pcm(&[(20_000, -20_000)]), false));

        let mut output = [0; 2];
        mixer.mix_into(&mut output);

        assert_eq!(output, [i16::MAX, i16::MIN]);
    }

    #[test]
    fn a_full_voice_pool_steals_the_oldest_voice() {
        let mut mixer = AudioMixer::default();

        for id in 0..MAX_AUDIO_VOICES {
            let value = (id + 1) as i16;
            assert!(mixer.play(SampleId(id), pcm(&[(value, value)]), false));
        }

        assert_eq!(mixer.active_voice_count(), MAX_AUDIO_VOICES);
        assert!(mixer.play(
            SampleId(MAX_AUDIO_VOICES),
            pcm(&[((MAX_AUDIO_VOICES + 1) as i16, (MAX_AUDIO_VOICES + 1) as i16)]),
            false
        ));

        assert_eq!(mixer.active_voice_count(), MAX_AUDIO_VOICES);
        let mut output = [0; 2];
        mixer.mix_into(&mut output);
        // Sample zero (value 1) was stolen; values 2 through 49 remain.
        assert_eq!(output, [1_224, 1_224]);
    }

    #[test]
    fn stopping_a_sample_only_stops_that_sample() {
        let mut mixer = AudioMixer::default();
        assert!(mixer.play(SampleId(1), pcm(&[(100, 100), (100, 100)]), true));
        assert!(mixer.play(SampleId(2), pcm(&[(50, 50), (50, 50)]), true));

        mixer.stop(SampleId(1));

        let mut output = [0; 2];
        mixer.mix_into(&mut output);
        assert_eq!(output, [50, 50]);
        assert_eq!(mixer.active_voice_count(), 1);
    }

    #[test]
    fn empty_samples_do_not_allocate_a_voice() {
        let mut mixer = AudioMixer::default();
        assert!(!mixer.play(SampleId(0), Arc::from(Vec::<i16>::new()), false));
        assert_eq!(mixer.active_voice_count(), 0);
    }
}
