use rodio::Source;
use std::time::Duration;

pub struct CrossfadeLoop {
    samples: Vec<i16>,
    position: usize,
    channels: u16,
    sample_rate: u32,
    crossfade_samples: usize,
    _total_frames: usize,
}

impl CrossfadeLoop {
    pub fn new(samples: Vec<i16>, channels: u16, sample_rate: u32, crossfade_secs: f32) -> Self {
        let total_frames = samples.len() / channels as usize;
        let crossfade_frames = (sample_rate as f32 * crossfade_secs) as usize;
        let crossfade_frames = crossfade_frames.min(total_frames / 2);

        Self {
            samples,
            position: 0,
            channels,
            sample_rate,
            crossfade_samples: crossfade_frames * channels as usize,
            _total_frames: total_frames,
        }
    }

    fn total_samples(&self) -> usize {
        self.samples.len()
    }

    fn crossfade_start(&self) -> usize {
        self.total_samples() - self.crossfade_samples
    }
}

impl Iterator for CrossfadeLoop {
    type Item = i16;

    fn next(&mut self) -> Option<Self::Item> {
        if self.samples.is_empty() {
            return None;
        }

        let total = self.total_samples();
        let cf_start = self.crossfade_start();
        let cf_len = self.crossfade_samples;

        let sample = if self.position >= cf_start && cf_len > 0 {
            let cf_pos = self.position - cf_start;
            let progress = cf_pos as f32 / cf_len as f32;

            let outgoing = self.samples[self.position] as f32 * (1.0 - progress);
            let incoming = self.samples[cf_pos % total] as f32 * progress;

            (outgoing + incoming) as i16
        } else {
            self.samples[self.position]
        };

        self.position += 1;

        if self.position >= total {
            self.position = self.crossfade_samples;
        }

        Some(sample)
    }
}

impl Source for CrossfadeLoop {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        self.channels
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}
