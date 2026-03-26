use rodio::{Decoder, Source};
use std::collections::VecDeque;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::time::Duration;

fn open_decoder(path: &Path) -> Result<Decoder<BufReader<File>>, String> {
    let file = File::open(path).map_err(|e| format!("File open error: {e}"))?;
    let reader = BufReader::new(file);
    Decoder::new(reader).map_err(|e| format!("Decode error: {e}"))
}

enum StreamState {
    Streaming(Decoder<BufReader<File>>),
    Crossfading {
        tail_snapshot: Vec<i16>,
        incoming: Decoder<BufReader<File>>,
        cf_pos: usize,
    },
    Transitioning,
}

pub struct StreamingCrossfadeLoop {
    file_path: PathBuf,
    channels: u16,
    sample_rate: u32,
    crossfade_samples: usize,
    tail: VecDeque<i16>,
    state: StreamState,
}

impl StreamingCrossfadeLoop {
    pub fn new(path: &Path, crossfade_secs: f32) -> Result<Self, String> {
        let meta = open_decoder(path)?;
        let channels = meta.channels();
        let sample_rate = meta.sample_rate();
        drop(meta);

        let crossfade_samples = (sample_rate as f32 * crossfade_secs) as usize * channels as usize;

        let decoder = open_decoder(path)?;

        Ok(Self {
            file_path: path.to_path_buf(),
            channels,
            sample_rate,
            crossfade_samples,
            tail: VecDeque::with_capacity(crossfade_samples.max(1)),
            state: StreamState::Streaming(decoder),
        })
    }
}

impl Iterator for StreamingCrossfadeLoop {
    type Item = i16;

    fn next(&mut self) -> Option<i16> {
        // Transition: crossfade complete → back to Streaming
        let done = matches!(
            &self.state,
            StreamState::Crossfading { cf_pos, .. } if *cf_pos >= self.crossfade_samples
        );
        if done {
            if let StreamState::Crossfading { incoming, .. } =
                std::mem::replace(&mut self.state, StreamState::Transitioning)
            {
                self.state = StreamState::Streaming(incoming);
            }
        }

        match &mut self.state {
            StreamState::Transitioning => unreachable!(),

            StreamState::Streaming(decoder) => match decoder.next() {
                Some(sample) => {
                    if self.crossfade_samples > 0 {
                        if self.tail.len() == self.crossfade_samples {
                            self.tail.pop_front();
                        }
                        self.tail.push_back(sample);
                    }
                    Some(sample)
                }
                None => {
                    if self.crossfade_samples == 0 {
                        match open_decoder(&self.file_path) {
                            Ok(d) => {
                                self.state = StreamState::Streaming(d);
                                self.next()
                            }
                            Err(_) => None,
                        }
                    } else {
                        let tail_snapshot = self.tail.iter().copied().collect();
                        match open_decoder(&self.file_path) {
                            Ok(incoming) => {
                                self.state = StreamState::Crossfading {
                                    tail_snapshot,
                                    incoming,
                                    cf_pos: 0,
                                };
                                self.next()
                            }
                            Err(_) => None,
                        }
                    }
                }
            },

            StreamState::Crossfading {
                tail_snapshot,
                incoming,
                cf_pos,
            } => {
                let outgoing = tail_snapshot.get(*cf_pos).copied().unwrap_or(0) as f32;
                let in_sample = incoming.next().unwrap_or(0);

                if self.crossfade_samples > 0 {
                    if self.tail.len() == self.crossfade_samples {
                        self.tail.pop_front();
                    }
                    self.tail.push_back(in_sample);
                }

                let progress = *cf_pos as f32 / self.crossfade_samples as f32;
                let blended = (outgoing * (1.0 - progress) + in_sample as f32 * progress) as i16;
                *cf_pos += 1;
                Some(blended)
            }
        }
    }
}

impl Source for StreamingCrossfadeLoop {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_test_wav(path: &Path, sample_rate: u32, samples: &[i16]) {
        let channels: u16 = 1;
        let bits_per_sample: u16 = 16;
        let data_size = (samples.len() * 2) as u32;
        let file_size = 36 + data_size;
        let byte_rate = sample_rate * channels as u32 * bits_per_sample as u32 / 8;
        let block_align = channels * bits_per_sample / 8;

        let mut f = File::create(path).unwrap();
        f.write_all(b"RIFF").unwrap();
        f.write_all(&file_size.to_le_bytes()).unwrap();
        f.write_all(b"WAVE").unwrap();
        f.write_all(b"fmt ").unwrap();
        f.write_all(&16u32.to_le_bytes()).unwrap();
        f.write_all(&1u16.to_le_bytes()).unwrap();
        f.write_all(&channels.to_le_bytes()).unwrap();
        f.write_all(&sample_rate.to_le_bytes()).unwrap();
        f.write_all(&byte_rate.to_le_bytes()).unwrap();
        f.write_all(&block_align.to_le_bytes()).unwrap();
        f.write_all(&bits_per_sample.to_le_bytes()).unwrap();
        f.write_all(b"data").unwrap();
        f.write_all(&data_size.to_le_bytes()).unwrap();
        for s in samples {
            f.write_all(&s.to_le_bytes()).unwrap();
        }
    }

    #[test]
    fn loops_indefinitely() {
        let path = std::env::temp_dir().join("rustbird_test_loop.wav");
        let samples: Vec<i16> = (0..4410).map(|i| i as i16).collect();
        write_test_wav(&path, 44100, &samples);

        let looper = StreamingCrossfadeLoop::new(&path, 0.02).unwrap();
        let output: Vec<i16> = looper.take(4410 * 3).collect();
        assert_eq!(
            output.len(),
            4410 * 3,
            "must produce 3x the file's sample count via looping"
        );

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn zero_crossfade_loops_cleanly() {
        let path = std::env::temp_dir().join("rustbird_test_zero_cf.wav");
        let samples = vec![1000i16; 2000];
        write_test_wav(&path, 44100, &samples);

        let looper = StreamingCrossfadeLoop::new(&path, 0.0).unwrap();
        let output: Vec<i16> = looper.take(4000).collect();
        assert_eq!(output.len(), 4000);

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn reports_correct_channels_and_sample_rate() {
        let path = std::env::temp_dir().join("rustbird_test_meta.wav");
        let samples = vec![0i16; 4410];
        write_test_wav(&path, 44100, &samples);

        let looper = StreamingCrossfadeLoop::new(&path, 0.02).unwrap();
        assert_eq!(looper.channels(), 1);
        assert_eq!(looper.sample_rate(), 44100);

        std::fs::remove_file(&path).ok();
    }
}
