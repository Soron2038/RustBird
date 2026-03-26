use crate::crossfade::CrossfadeLoop;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::time::Duration;

pub struct AudioEngine {
    _stream: OutputStream,
    stream_handle: OutputStreamHandle,
    sinks: HashMap<String, Sink>,
    master_volume: f32,
    volumes: HashMap<String, f32>,
}

impl AudioEngine {
    pub fn new() -> Result<Self, String> {
        let (_stream, stream_handle) =
            OutputStream::try_default().map_err(|e| format!("Audio output error: {}", e))?;

        Ok(Self {
            _stream,
            stream_handle,
            sinks: HashMap::new(),
            master_volume: 0.8,
            volumes: HashMap::new(),
        })
    }

    pub fn play_sound(
        &mut self,
        id: &str,
        file_path: &Path,
        volume: f32,
        crossfade_secs: f32,
    ) -> Result<(), String> {
        self.stop_sound(id);

        let sink = Sink::try_new(&self.stream_handle)
            .map_err(|e| format!("Failed to create sink: {}", e))?;

        let source = self.create_crossfade_source(file_path, crossfade_secs)?;

        // Convert i16 → f32 for fade_in, then append
        let source = source
            .convert_samples::<f32>()
            .fade_in(Duration::from_secs(1));

        sink.append(source);
        sink.set_volume(volume * self.master_volume);

        self.sinks.insert(id.to_string(), sink);
        self.volumes.insert(id.to_string(), volume);

        Ok(())
    }

    fn create_crossfade_source(
        &self,
        file_path: &Path,
        crossfade_secs: f32,
    ) -> Result<CrossfadeLoop, String> {
        let file = File::open(file_path).map_err(|e| format!("File open error: {}", e))?;
        let reader = BufReader::new(file);
        let source = Decoder::new(reader).map_err(|e| format!("Decode error: {}", e))?;

        let channels = source.channels();
        let sample_rate = source.sample_rate();

        let samples: Vec<i16> = source.collect();

        if samples.is_empty() {
            return Err("Audio file is empty".to_string());
        }

        Ok(CrossfadeLoop::new(samples, channels, sample_rate, crossfade_secs))
    }

    pub fn stop_sound(&mut self, id: &str) {
        if let Some(sink) = self.sinks.remove(id) {
            sink.stop();
        }
        self.volumes.remove(id);
    }

    pub fn set_volume(&mut self, id: &str, volume: f32) {
        self.volumes.insert(id.to_string(), volume);
        if let Some(sink) = self.sinks.get(id) {
            sink.set_volume(volume * self.master_volume);
        }
    }

    pub fn set_master_volume(&mut self, volume: f32) {
        self.master_volume = volume;
        for (id, sink) in &self.sinks {
            let individual = self.volumes.get(id).copied().unwrap_or(0.7);
            sink.set_volume(individual * self.master_volume);
        }
    }

    pub fn pause_all(&self) {
        for sink in self.sinks.values() {
            sink.pause();
        }
    }

    pub fn resume_all(&self) {
        for sink in self.sinks.values() {
            sink.play();
        }
    }

}
