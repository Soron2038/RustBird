use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sound {
    pub id: String,
    pub name: String,
    pub file_path: PathBuf,
    pub is_bundled: bool,
    pub is_active: bool,
    pub volume: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppState {
    pub sounds: Vec<Sound>,
    pub master_volume: f32,
    pub is_paused: bool,
    pub crossfade_duration: f32,
    pub autostart_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedState {
    pub sound_states: Vec<PersistedSoundState>,
    pub master_volume: f32,
    pub is_paused: bool,
    pub crossfade_duration: f32,
    pub autostart_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedSoundState {
    pub id: String,
    pub is_active: bool,
    pub volume: f32,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            sounds: Vec::new(),
            master_volume: 0.8,
            is_paused: false,
            crossfade_duration: 2.0,
            autostart_enabled: true,
        }
    }
}

impl AppState {
    pub fn to_persisted(&self) -> PersistedState {
        PersistedState {
            sound_states: self
                .sounds
                .iter()
                .map(|s| PersistedSoundState {
                    id: s.id.clone(),
                    is_active: s.is_active,
                    volume: s.volume,
                })
                .collect(),
            master_volume: self.master_volume,
            is_paused: self.is_paused,
            crossfade_duration: self.crossfade_duration,
            autostart_enabled: self.autostart_enabled,
        }
    }

    pub fn apply_persisted(&mut self, persisted: &PersistedState) {
        self.master_volume = persisted.master_volume;
        self.is_paused = persisted.is_paused;
        self.crossfade_duration = persisted.crossfade_duration;
        self.autostart_enabled = persisted.autostart_enabled;

        for ps in &persisted.sound_states {
            if let Some(sound) = self.sounds.iter_mut().find(|s| s.id == ps.id) {
                sound.is_active = ps.is_active;
                sound.volume = ps.volume;
            }
        }
    }
}

pub fn app_data_dir() -> PathBuf {
    let home = dirs::home_dir().expect("Could not find home directory");
    home.join("Library")
        .join("Application Support")
        .join("RustBird")
}

pub fn config_path() -> PathBuf {
    app_data_dir().join("config.json")
}

pub fn user_sounds_dir() -> PathBuf {
    app_data_dir().join("sounds")
}

pub fn save_state(state: &AppState) -> Result<(), String> {
    let dir = app_data_dir();
    fs::create_dir_all(&dir).map_err(|e| format!("Failed to create app data dir: {}", e))?;

    let persisted = state.to_persisted();
    let json =
        serde_json::to_string_pretty(&persisted).map_err(|e| format!("Serialize error: {}", e))?;
    fs::write(config_path(), json).map_err(|e| format!("Write error: {}", e))?;
    Ok(())
}

pub fn load_persisted_state() -> Option<PersistedState> {
    let path = config_path();
    if !path.exists() {
        return None;
    }
    let data = fs::read_to_string(path).ok()?;
    serde_json::from_str(&data).ok()
}

pub fn discover_bundled_sounds(sounds_dir: &Path) -> Vec<Sound> {
    let mut sounds = Vec::new();
    if let Ok(entries) = fs::read_dir(sounds_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(ext) = path.extension() {
                if ext == "mp3" || ext == "wav" {
                    let stem = path.file_stem().unwrap().to_string_lossy().to_string();
                    let name = capitalize_first(&stem);
                    sounds.push(Sound {
                        id: stem,
                        name,
                        file_path: path,
                        is_bundled: true,
                        is_active: false,
                        volume: 0.7,
                    });
                }
            }
        }
    }
    sounds.sort_by(|a, b| a.name.cmp(&b.name));
    sounds
}

pub fn discover_user_sounds() -> Vec<Sound> {
    let dir = user_sounds_dir();
    if !dir.exists() {
        return Vec::new();
    }
    let mut sounds = Vec::new();
    if let Ok(entries) = fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(ext) = path.extension() {
                if ext == "mp3" || ext == "wav" {
                    let stem = path.file_stem().unwrap().to_string_lossy().to_string();
                    let name = capitalize_first(&stem);
                    sounds.push(Sound {
                        id: format!("user_{}", stem),
                        name,
                        file_path: path,
                        is_bundled: false,
                        is_active: false,
                        volume: 0.7,
                    });
                }
            }
        }
    }
    sounds.sort_by(|a, b| a.name.cmp(&b.name));
    sounds
}

fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().to_string() + chars.as_str(),
    }
}
