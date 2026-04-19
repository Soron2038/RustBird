use crate::error::AppError;
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
    pub autopause_on_lock: bool,
    pub auto_update_enabled: bool,
    #[serde(skip)]
    pub dialog_open: bool,
    #[serde(skip)]
    pub lock_triggered_pause: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedState {
    pub sound_states: Vec<PersistedSoundState>,
    pub master_volume: f32,
    pub is_paused: bool,
    pub crossfade_duration: f32,
    pub autostart_enabled: bool,
    #[serde(default)]
    pub autopause_on_lock: bool,
    #[serde(default)]
    pub auto_update_enabled: bool,
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
            autopause_on_lock: false,
            auto_update_enabled: false,
            dialog_open: false,
            lock_triggered_pause: false,
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
            autopause_on_lock: self.autopause_on_lock,
            auto_update_enabled: self.auto_update_enabled,
        }
    }

    pub fn apply_persisted(&mut self, persisted: &PersistedState) {
        self.master_volume = persisted.master_volume;
        self.is_paused = persisted.is_paused;
        self.crossfade_duration = persisted.crossfade_duration;
        self.autostart_enabled = persisted.autostart_enabled;
        self.autopause_on_lock = persisted.autopause_on_lock;
        self.auto_update_enabled = persisted.auto_update_enabled;

        for ps in &persisted.sound_states {
            if let Some(sound) = self.sounds.iter_mut().find(|s| s.id == ps.id) {
                sound.is_active = ps.is_active;
                sound.volume = ps.volume;
            }
        }
    }
}

pub fn app_data_dir() -> Result<PathBuf, AppError> {
    dirs::data_dir()
        .ok_or(AppError::HomeDirNotFound)
        .map(|d| d.join("RustBird"))
}

pub fn config_path() -> Result<PathBuf, AppError> {
    Ok(app_data_dir()?.join("config.json"))
}

pub fn user_sounds_dir() -> Result<PathBuf, AppError> {
    Ok(app_data_dir()?.join("sounds"))
}

pub fn save_state(state: &AppState) -> Result<(), AppError> {
    let dir = app_data_dir()?;
    fs::create_dir_all(&dir)?;
    let persisted = state.to_persisted();
    let json = serde_json::to_string_pretty(&persisted)?;
    fs::write(config_path()?, json)?;
    Ok(())
}

pub fn load_persisted_state() -> Option<PersistedState> {
    let path = config_path().ok()?;
    if !path.exists() {
        return None;
    }
    let data = fs::read_to_string(path).ok()?;
    serde_json::from_str(&data).ok()
}

fn discover_sounds(dir: &Path, is_bundled: bool, id_prefix: &str) -> Vec<Sound> {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };
    let mut sounds: Vec<Sound> = entries
        .flatten()
        .filter_map(|entry| {
            let path = entry.path();
            let ext = path.extension()?;
            if ext != "mp3" && ext != "wav" {
                return None;
            }
            let stem = path.file_stem()?.to_string_lossy().to_string();
            let name = capitalize_first(&stem);
            let id = if id_prefix.is_empty() {
                stem
            } else {
                format!("{}{}", id_prefix, stem)
            };
            Some(Sound {
                id,
                name,
                file_path: path,
                is_bundled,
                is_active: false,
                volume: 0.7,
            })
        })
        .collect();
    sounds.sort_by(|a, b| a.name.cmp(&b.name));
    sounds
}

pub fn discover_bundled_sounds(sounds_dir: &Path) -> Vec<Sound> {
    discover_sounds(sounds_dir, true, "")
}

pub fn discover_user_sounds() -> Vec<Sound> {
    let dir = match user_sounds_dir() {
        Ok(d) => d,
        Err(_) => return Vec::new(),
    };
    if !dir.exists() {
        return Vec::new();
    }
    discover_sounds(&dir, false, "user_")
}

pub(crate) fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().to_string() + chars.as_str(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::path::PathBuf;

    /// Helper: build a Sound with the given id, is_active, and volume.
    fn make_sound(id: &str, active: bool, volume: f32) -> Sound {
        Sound {
            id: id.to_string(),
            name: capitalize_first(id),
            file_path: PathBuf::from(format!("/fake/{id}.mp3")),
            is_bundled: true,
            is_active: active,
            volume,
        }
    }

    /// Helper: create a unique temp directory for a test.
    fn test_tmp_dir(test_name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join("rustbird_tests").join(test_name);
        let _ = fs::remove_dir_all(&dir); // clean previous runs
        fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    // ── to_persisted / apply_persisted round-trip ────────────────────

    #[test]
    fn persisted_state_roundtrip() {
        let state = AppState {
            sounds: vec![
                make_sound("rain", true, 0.9),
                make_sound("wind", false, 0.3),
            ],
            master_volume: 0.65,
            is_paused: true,
            crossfade_duration: 3.5,
            autostart_enabled: false,
            autopause_on_lock: true,
            dialog_open: false,
            lock_triggered_pause: false,
        };

        // Round-trip through JSON
        let persisted = state.to_persisted();
        let json = serde_json::to_string(&persisted).expect("serialize");
        let restored: PersistedState = serde_json::from_str(&json).expect("deserialize");

        // Apply to a fresh AppState that already knows the same sound IDs
        let mut new_state = AppState {
            sounds: vec![
                make_sound("rain", false, 0.7),
                make_sound("wind", false, 0.7),
            ],
            ..AppState::default()
        };
        new_state.apply_persisted(&restored);

        assert_eq!(new_state.master_volume, state.master_volume);
        assert_eq!(new_state.is_paused, state.is_paused);
        assert_eq!(new_state.crossfade_duration, state.crossfade_duration);
        assert_eq!(new_state.autostart_enabled, state.autostart_enabled);
        assert_eq!(new_state.autopause_on_lock, state.autopause_on_lock);

        let rain = new_state.sounds.iter().find(|s| s.id == "rain").unwrap();
        assert!(rain.is_active);
        assert!((rain.volume - 0.9).abs() < f32::EPSILON);

        let wind = new_state.sounds.iter().find(|s| s.id == "wind").unwrap();
        assert!(!wind.is_active);
        assert!((wind.volume - 0.3).abs() < f32::EPSILON);
    }

    // ── apply_persisted with unknown IDs ─────────────────────────────

    #[test]
    fn apply_persisted_unknown_ids() {
        let mut state = AppState {
            sounds: vec![make_sound("rain", false, 0.7)],
            ..AppState::default()
        };

        let persisted = PersistedState {
            sound_states: vec![
                PersistedSoundState {
                    id: "nonexistent_1".to_string(),
                    is_active: true,
                    volume: 1.0,
                },
                PersistedSoundState {
                    id: "nonexistent_2".to_string(),
                    is_active: true,
                    volume: 0.5,
                },
            ],
            master_volume: 0.5,
            is_paused: true,
            crossfade_duration: 1.0,
            autostart_enabled: false,
            autopause_on_lock: false,
        };

        // Must not panic
        state.apply_persisted(&persisted);

        // Scalar fields are still applied
        assert_eq!(state.master_volume, 0.5);
        assert!(state.is_paused);

        // Existing sound is unchanged because persisted had no matching ID
        let rain = state.sounds.iter().find(|s| s.id == "rain").unwrap();
        assert!(!rain.is_active);
        assert!((rain.volume - 0.7).abs() < f32::EPSILON);
    }

    // ── discover_bundled_sounds: extension filtering ─────────────────

    #[test]
    fn discover_sounds_filters_extensions() {
        let dir = test_tmp_dir("discover_sounds_filters_extensions");
        for name in &[
            "bird.mp3",
            "wave.wav",
            "notes.txt",
            "photo.png",
            "song.flac",
        ] {
            File::create(dir.join(name)).expect("create file");
        }

        let sounds = discover_bundled_sounds(&dir);
        let names: Vec<&str> = sounds.iter().map(|s| s.name.as_str()).collect();

        assert_eq!(sounds.len(), 2);
        assert!(names.contains(&"Bird"));
        assert!(names.contains(&"Wave"));
    }

    // ── discover_bundled_sounds: empty directory ─────────────────────

    #[test]
    fn discover_sounds_empty_dir() {
        let dir = test_tmp_dir("discover_sounds_empty_dir");
        let sounds = discover_bundled_sounds(&dir);
        assert!(sounds.is_empty());
    }

    // ── discover_bundled_sounds: alphabetical sort ───────────────────

    #[test]
    fn discover_sounds_sorts_alphabetically() {
        let dir = test_tmp_dir("discover_sounds_sorts_alphabetically");
        // Create files whose stems sort differently than creation order
        for name in &["zebra.mp3", "alpha.wav", "middle.mp3"] {
            File::create(dir.join(name)).expect("create file");
        }

        let sounds = discover_bundled_sounds(&dir);
        let names: Vec<&str> = sounds.iter().map(|s| s.name.as_str()).collect();

        assert_eq!(names, vec!["Alpha", "Middle", "Zebra"]);
    }

    // ── autopause_on_lock persistence ────────────────────────────────

    #[test]
    fn autopause_on_lock_persisted() {
        let state = AppState {
            autopause_on_lock: true,
            ..AppState::default()
        };
        let persisted = state.to_persisted();
        assert!(persisted.autopause_on_lock);

        let json = serde_json::to_string(&persisted).expect("serialize");
        let restored: PersistedState = serde_json::from_str(&json).expect("deserialize");

        let mut new_state = AppState::default();
        new_state.apply_persisted(&restored);
        assert!(new_state.autopause_on_lock);
    }

    #[test]
    fn auto_update_enabled_defaults_false_and_persists() {
        // Default must be false
        let state = AppState::default();
        assert!(!state.auto_update_enabled);

        // Enabled=true round-trips through JSON
        let state_on = AppState {
            auto_update_enabled: true,
            ..AppState::default()
        };
        let persisted = state_on.to_persisted();
        assert!(persisted.auto_update_enabled);

        let json = serde_json::to_string(&persisted).expect("serialize");
        let restored: PersistedState = serde_json::from_str(&json).expect("deserialize");

        let mut new_state = AppState::default();
        new_state.apply_persisted(&restored);
        assert!(new_state.auto_update_enabled);
    }

    #[test]
    fn auto_update_enabled_defaults_false_on_missing_field() {
        // Old config JSON without the field should deserialize with default=false
        let json = r#"{"sound_states":[],"master_volume":0.8,"is_paused":false,"crossfade_duration":2.0,"autostart_enabled":true,"autopause_on_lock":false}"#;
        let restored: PersistedState = serde_json::from_str(json).expect("deserialize");
        assert!(!restored.auto_update_enabled);
    }

    #[test]
    fn lock_triggered_pause_not_persisted() {
        let state = AppState {
            lock_triggered_pause: true,
            ..AppState::default()
        };
        let persisted = state.to_persisted();
        let json = serde_json::to_string(&persisted).expect("serialize");
        // lock_triggered_pause must not appear in the JSON
        assert!(!json.contains("lock_triggered_pause"));
    }

    // ── capitalize_first ─────────────────────────────────────────────

    #[test]
    fn capitalize_first_empty() {
        assert_eq!(capitalize_first(""), "");
    }

    #[test]
    fn capitalize_first_single_char() {
        assert_eq!(capitalize_first("a"), "A");
    }

    #[test]
    fn capitalize_first_unicode() {
        assert_eq!(capitalize_first("über"), "Über");
    }
}
