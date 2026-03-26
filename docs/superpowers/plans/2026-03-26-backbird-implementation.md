# BackBird Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a macOS menu bar app in Rust (Tauri v2) that plays ambient bird songs with multi-track mixing, crossfade looping, and a minimal monochromatic UI.

**Architecture:** Tauri v2 with Rust backend (rodio for audio, serde for persistence) and Svelte frontend. The app lives in the system tray with a popover window. No Dock icon. Audio engine manages multiple Sinks on a shared OutputStream, each with independent volume. State persists to `~/Library/Application Support/BackBird/config.json`.

**Tech Stack:** Tauri v2, Rust, rodio 0.20, Svelte 5, Vite, tauri-plugin-positioner, tauri-plugin-dialog, tauri-plugin-autostart

---

## File Map

### Rust Backend (`src-tauri/`)

| File | Responsibility |
|------|---------------|
| `src/main.rs` | Entry point, `main()` calls `lib::run()` |
| `src/lib.rs` | Tauri Builder setup: plugins, state, commands, tray icon, window config |
| `src/audio.rs` | `AudioEngine` struct: manages `OutputStream`, per-sound `Sink`s, crossfade looping, volume, pause/resume |
| `src/state.rs` | `AppState`, `Sound`, `PersistedState` structs; load/save JSON; sound discovery (bundled + user) |
| `src/commands.rs` | All `#[tauri::command]` handlers: `get_state`, `toggle_sound`, `set_volume`, `set_master_volume`, `pause_all`, `resume_all`, `import_sound`, `remove_sound` |
| `Cargo.toml` | Dependencies: tauri, rodio, serde, serde_json, uuid, tauri-plugin-* |
| `tauri.conf.json` | App config: no decorations, small window, bundle identifier, tray icon path |
| `capabilities/default.json` | Permissions for dialog, autostart, positioner |

### Svelte Frontend (`src/`)

| File | Responsibility |
|------|---------------|
| `main.js` | Svelte mount point |
| `App.svelte` | Root: fetches state on mount, listens for backend events, routes to main view or settings |
| `lib/Header.svelte` | Title "BackBird", pause/resume toggle, settings icon |
| `lib/Mixer.svelte` | Vertical faders for active sounds + horizontal master volume slider |
| `lib/SoundList.svelte` | Scrollable list of all sounds with toggle dots, "＋ Hinzufügen" button |
| `lib/Settings.svelte` | About info, autostart toggle, crossfade duration slider |
| `app.css` | Minimal Mono theme: monochromatic, `prefers-color-scheme`, `-apple-system` font |

### Root

| File | Responsibility |
|------|---------------|
| `index.html` | Vite entry HTML |
| `package.json` | Node deps: @tauri-apps/api, @tauri-apps/plugin-dialog, @tauri-apps/plugin-autostart |
| `vite.config.js` | Vite + Svelte plugin config |
| `svelte.config.js` | Svelte compiler config |

---

## Task 1: Project Scaffolding

**Files:**
- Create: `package.json`, `vite.config.js`, `svelte.config.js`, `index.html`, `src/main.js`, `src/App.svelte`, `src/app.css`
- Create: `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, `src-tauri/build.rs`, `src-tauri/src/main.rs`, `src-tauri/src/lib.rs`
- Create: `src-tauri/capabilities/default.json`
- Create: `.gitignore`

- [ ] **Step 1: Initialize Tauri v2 project with Svelte**

```bash
cd /Users/witt/Developer/BackBird2
npm create tauri-app@latest . -- --template svelte --manager npm
```

If the directory is non-empty, run from parent and move files, or use the interactive CLI. The key is: Tauri v2 + Svelte + npm.

After scaffolding, verify the structure exists:
```bash
ls src-tauri/src/main.rs src-tauri/Cargo.toml src/App.svelte package.json
```

- [ ] **Step 2: Add Rust dependencies to Cargo.toml**

In `src-tauri/Cargo.toml`, add these under `[dependencies]`:

```toml
rodio = "0.20"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
uuid = { version = "1", features = ["v4"] }
tauri-plugin-positioner = { version = "2", features = ["tray-icon"] }
tauri-plugin-dialog = "2"
tauri-plugin-autostart = "2"
```

- [ ] **Step 3: Add frontend dependencies**

```bash
cd /Users/witt/Developer/BackBird2
npm install @tauri-apps/plugin-dialog @tauri-apps/plugin-autostart @tauri-apps/plugin-positioner
```

- [ ] **Step 4: Configure tauri.conf.json for menu bar app**

Replace the contents of `src-tauri/tauri.conf.json` with:

```json
{
  "productName": "BackBird",
  "version": "0.1.0",
  "identifier": "com.backbird.app",
  "build": {
    "beforeBuildCommand": "npm run build",
    "beforeDevCommand": "npm run dev",
    "devUrl": "http://localhost:5173",
    "frontendDist": "../dist"
  },
  "app": {
    "windows": [
      {
        "label": "main",
        "title": "BackBird",
        "width": 280,
        "height": 420,
        "resizable": false,
        "decorations": false,
        "visible": false,
        "skipTaskbar": true,
        "alwaysOnTop": true
      }
    ],
    "trayIcon": {
      "iconPath": "icons/trayIconWhite.png",
      "iconAsTemplate": true
    },
    "security": {
      "csp": null
    }
  },
  "bundle": {
    "active": true,
    "targets": ["dmg", "app"],
    "icon": [
      "icons/icon.icns",
      "icons/icon.png"
    ],
    "macOS": {
      "minimumSystemVersion": "10.15"
    }
  }
}
```

- [ ] **Step 5: Copy existing assets into src-tauri/icons**

The tray icons need to be in `src-tauri/icons/` for Tauri to bundle them:

```bash
cp assets/icons/trayIconWhite.png src-tauri/icons/trayIconWhite.png
cp assets/icons/trayIconWhite@2x.png src-tauri/icons/trayIconWhite@2x.png
cp assets/icons/app-icon.png src-tauri/icons/icon.png
```

Generate the `.icns` file from the 512x512 PNG for macOS bundling:

```bash
mkdir -p /tmp/backbird_icon.iconset
sips -z 16 16 assets/icons/app-icon.png --out /tmp/backbird_icon.iconset/icon_16x16.png
sips -z 32 32 assets/icons/app-icon.png --out /tmp/backbird_icon.iconset/icon_16x16@2x.png
sips -z 32 32 assets/icons/app-icon.png --out /tmp/backbird_icon.iconset/icon_32x32.png
sips -z 64 64 assets/icons/app-icon.png --out /tmp/backbird_icon.iconset/icon_32x32@2x.png
sips -z 128 128 assets/icons/app-icon.png --out /tmp/backbird_icon.iconset/icon_128x128.png
sips -z 256 256 assets/icons/app-icon.png --out /tmp/backbird_icon.iconset/icon_128x128@2x.png
sips -z 256 256 assets/icons/app-icon.png --out /tmp/backbird_icon.iconset/icon_256x256.png
sips -z 512 512 assets/icons/app-icon.png --out /tmp/backbird_icon.iconset/icon_256x256@2x.png
sips -z 512 512 assets/icons/app-icon.png --out /tmp/backbird_icon.iconset/icon_512x512.png
cp assets/icons/app-icon.png /tmp/backbird_icon.iconset/icon_512x512@2x.png
iconutil -c icns /tmp/backbird_icon.iconset -o src-tauri/icons/icon.icns
```

- [ ] **Step 6: Set up capabilities**

Create `src-tauri/capabilities/default.json`:

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Default permissions for BackBird",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "dialog:default",
    "autostart:default",
    "positioner:default"
  ]
}
```

- [ ] **Step 7: Create minimal lib.rs with plugins**

Replace `src-tauri/src/lib.rs` with:

```rust
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_positioner::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .setup(|app| {
            // Hide from Dock on macOS
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 8: Update .gitignore**

Append to `.gitignore`:

```
.superpowers/
```

- [ ] **Step 9: Verify it compiles and starts**

```bash
cd /Users/witt/Developer/BackBird2
cargo tauri dev
```

Expected: The app starts with no Dock icon. A tray icon appears in the menu bar. A small empty window may flash — that's OK, we'll fix the window management in Task 3.

- [ ] **Step 10: Commit**

```bash
git init
git add -A
git commit -m "feat: scaffold Tauri v2 + Svelte project with plugins"
```

---

## Task 2: State Management & Persistence

**Files:**
- Create: `src-tauri/src/state.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Create state.rs with data model**

Create `src-tauri/src/state.rs`:

```rust
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

/// The subset of state that gets persisted to config.json.
/// File paths for bundled sounds are not saved — they are resolved at startup.
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

/// Returns the app data directory: ~/Library/Application Support/BackBird/
pub fn app_data_dir() -> PathBuf {
    let home = dirs::home_dir().expect("Could not find home directory");
    home.join("Library")
        .join("Application Support")
        .join("BackBird")
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

/// Discover bundled sounds from a directory. Each .mp3 file becomes a Sound
/// with id = filename (without extension).
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

/// Discover user-imported sounds from ~/Library/Application Support/BackBird/sounds/
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
```

- [ ] **Step 2: Add `dirs` dependency**

Add to `src-tauri/Cargo.toml` under `[dependencies]`:

```toml
dirs = "6"
```

- [ ] **Step 3: Register state module in lib.rs**

Add `mod state;` at the top of `src-tauri/src/lib.rs`:

```rust
mod state;

use tauri::Manager;
// ... rest of lib.rs
```

- [ ] **Step 4: Verify it compiles**

```bash
cd /Users/witt/Developer/BackBird2
cargo build --manifest-path src-tauri/Cargo.toml
```

Expected: Compiles without errors.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/state.rs src-tauri/Cargo.toml src-tauri/src/lib.rs
git commit -m "feat: add state management with persistence and sound discovery"
```

---

## Task 3: Audio Engine

**Files:**
- Create: `src-tauri/src/audio.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Create audio.rs with AudioEngine**

Create `src-tauri/src/audio.rs`:

```rust
use rodio::{Decoder, OutputStream, Sink, Source};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::time::Duration;

pub struct AudioEngine {
    _stream: OutputStream,
    stream_handle: rodio::OutputStreamHandle,
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

    /// Start playing a sound in a crossfade loop.
    /// `crossfade_secs` is the duration of the fade at loop boundaries.
    pub fn play_sound(
        &mut self,
        id: &str,
        file_path: &Path,
        volume: f32,
        crossfade_secs: f32,
    ) -> Result<(), String> {
        // Stop existing sink for this id if any
        self.stop_sound(id);

        let sink = Sink::try_new(&self.stream_handle)
            .map_err(|e| format!("Failed to create sink: {}", e))?;

        // Load the file and create a crossfade-looping source
        let source = self
            .create_crossfade_source(file_path, crossfade_secs)
            .map_err(|e| format!("Failed to load audio: {}", e))?;

        // Apply fade-in on activation (1 second)
        let source = source.fade_in(Duration::from_secs(1));

        sink.append(source);
        sink.set_volume(volume * self.master_volume);

        self.sinks.insert(id.to_string(), sink);
        self.volumes.insert(id.to_string(), volume);

        Ok(())
    }

    fn create_crossfade_source(
        &self,
        file_path: &Path,
        _crossfade_secs: f32,
    ) -> Result<impl Source<Item = i16> + Send + 'static, String> {
        let file = File::open(file_path).map_err(|e| format!("File open error: {}", e))?;
        let reader = BufReader::new(file);
        let source = Decoder::new(reader).map_err(|e| format!("Decode error: {}", e))?;

        // For now, use repeat_infinite which loops without crossfade.
        // Crossfade looping requires a custom Source — we'll refine in Task 4.
        Ok(source.repeat_infinite())
    }

    /// Stop a sound with immediate stop (fade-out is handled by the caller
    /// setting volume to 0 before calling this after a delay, or we do it here).
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
        // Update all sinks with new effective volume
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

    pub fn is_sound_playing(&self, id: &str) -> bool {
        self.sinks.contains_key(id)
    }
}
```

- [ ] **Step 2: Register audio module in lib.rs**

Add `mod audio;` at the top of `src-tauri/src/lib.rs`:

```rust
mod audio;
mod state;
```

- [ ] **Step 3: Verify it compiles**

```bash
cd /Users/witt/Developer/BackBird2
cargo build --manifest-path src-tauri/Cargo.toml
```

Expected: Compiles. Warnings about unused code are fine at this stage.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/audio.rs src-tauri/src/lib.rs
git commit -m "feat: add audio engine with mixing, volume control, and looping"
```

---

## Task 4: Crossfade Looping Source

**Files:**
- Create: `src-tauri/src/crossfade.rs`
- Modify: `src-tauri/src/audio.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Create crossfade.rs with custom Source**

Create `src-tauri/src/crossfade.rs`:

```rust
use rodio::Source;
use std::time::Duration;

/// A Source that loops audio data with a crossfade at the loop boundary.
/// It plays the buffer, and near the end starts mixing in the beginning
/// of the next iteration with a linear crossfade.
pub struct CrossfadeLoop {
    /// The full decoded audio samples (interleaved)
    samples: Vec<i16>,
    /// Current position in samples array
    position: usize,
    /// Number of channels
    channels: u16,
    /// Sample rate
    sample_rate: u32,
    /// Number of samples (per channel) in the crossfade region
    crossfade_samples: usize,
    /// Total number of frames (samples / channels)
    total_frames: usize,
}

impl CrossfadeLoop {
    pub fn new(samples: Vec<i16>, channels: u16, sample_rate: u32, crossfade_secs: f32) -> Self {
        let total_frames = samples.len() / channels as usize;
        let crossfade_frames = (sample_rate as f32 * crossfade_secs) as usize;
        // Don't let crossfade exceed half the sound length
        let crossfade_frames = crossfade_frames.min(total_frames / 2);

        Self {
            samples,
            position: 0,
            channels,
            sample_rate,
            crossfade_samples: crossfade_frames * channels as usize,
            total_frames,
        }
    }

    fn total_samples(&self) -> usize {
        self.samples.len()
    }

    /// The number of samples before the crossfade region starts
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
            // We're in the crossfade region
            let cf_pos = self.position - cf_start;
            let progress = cf_pos as f32 / cf_len as f32; // 0.0 → 1.0

            let outgoing = self.samples[self.position] as f32 * (1.0 - progress);
            let incoming = self.samples[cf_pos % total] as f32 * progress;

            (outgoing + incoming) as i16
        } else {
            self.samples[self.position]
        };

        self.position += 1;

        // When we reach the end, jump back to where the crossfade region
        // would have blended into (skip the crossfade_samples at the start
        // since they were already mixed into the tail of the previous iteration).
        if self.position >= total {
            self.position = self.crossfade_samples;
        }

        Some(sample)
    }
}

impl Source for CrossfadeLoop {
    fn current_frame_len(&self) -> Option<usize> {
        None // infinite source
    }

    fn channels(&self) -> u16 {
        self.channels
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<Duration> {
        None // infinite loop
    }
}
```

- [ ] **Step 2: Update audio.rs to use CrossfadeLoop**

In `src-tauri/src/audio.rs`, add the import and replace `create_crossfade_source`:

Add at the top:
```rust
use crate::crossfade::CrossfadeLoop;
```

Replace the `create_crossfade_source` method:

```rust
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

        // Collect all samples into memory for gapless looping
        let samples: Vec<i16> = source.collect();

        if samples.is_empty() {
            return Err("Audio file is empty".to_string());
        }

        Ok(CrossfadeLoop::new(
            samples,
            channels,
            sample_rate,
            crossfade_secs,
        ))
    }
```

Also update the `play_sound` method's source type — since `CrossfadeLoop` already implements `Source<Item = i16>`, and `fade_in` needs `Source<Item = f32>`, we need to convert. Replace the source lines in `play_sound`:

```rust
        let source = self
            .create_crossfade_source(file_path, crossfade_secs)?;

        // Convert i16 → f32 for fade_in, then back
        let source = source
            .convert_samples::<f32>()
            .fade_in(Duration::from_secs(1));

        sink.append(source);
```

Remove the unused `Source` import if needed and add `convert_samples` — it's already available on `Source`.

- [ ] **Step 3: Register crossfade module in lib.rs**

Add to `src-tauri/src/lib.rs`:

```rust
mod audio;
mod crossfade;
mod state;
```

- [ ] **Step 4: Verify it compiles**

```bash
cargo build --manifest-path src-tauri/Cargo.toml
```

Expected: Compiles without errors.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/crossfade.rs src-tauri/src/audio.rs src-tauri/src/lib.rs
git commit -m "feat: add crossfade looping source for seamless audio loops"
```

---

## Task 5: Tauri Commands

**Files:**
- Create: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Create commands.rs**

Create `src-tauri/src/commands.rs`:

```rust
use crate::audio::AudioEngine;
use crate::state::{
    self, discover_user_sounds, save_state, user_sounds_dir, AppState, Sound,
};
use std::fs;
use std::sync::Mutex;
use tauri::State;
use uuid::Uuid;

pub struct AppStateMutex(pub Mutex<AppState>);
pub struct AudioEngineMutex(pub Mutex<AudioEngine>);

#[tauri::command]
pub fn get_state(state: State<'_, AppStateMutex>) -> AppState {
    state.0.lock().unwrap().clone()
}

#[tauri::command]
pub fn toggle_sound(
    id: String,
    state: State<'_, AppStateMutex>,
    engine: State<'_, AudioEngineMutex>,
) -> Result<AppState, String> {
    let mut app_state = state.0.lock().unwrap();
    let mut audio = engine.0.lock().unwrap();

    let sound = app_state
        .sounds
        .iter_mut()
        .find(|s| s.id == id)
        .ok_or_else(|| format!("Sound not found: {}", id))?;

    if sound.is_active {
        // Deactivate
        audio.stop_sound(&id);
        sound.is_active = false;
    } else {
        // Activate
        let file_path = sound.file_path.clone();
        let volume = sound.volume;
        let crossfade = app_state.crossfade_duration;
        audio.play_sound(&id, &file_path, volume, crossfade)?;
        sound.is_active = true;
    }

    let _ = save_state(&app_state);
    Ok(app_state.clone())
}

#[tauri::command]
pub fn set_volume(
    id: String,
    volume: f32,
    state: State<'_, AppStateMutex>,
    engine: State<'_, AudioEngineMutex>,
) -> Result<(), String> {
    let mut app_state = state.0.lock().unwrap();
    let mut audio = engine.0.lock().unwrap();

    let volume = volume.clamp(0.0, 1.0);

    if let Some(sound) = app_state.sounds.iter_mut().find(|s| s.id == id) {
        sound.volume = volume;
        audio.set_volume(&id, volume);
    }

    let _ = save_state(&app_state);
    Ok(())
}

#[tauri::command]
pub fn set_master_volume(
    volume: f32,
    state: State<'_, AppStateMutex>,
    engine: State<'_, AudioEngineMutex>,
) -> Result<(), String> {
    let mut app_state = state.0.lock().unwrap();
    let mut audio = engine.0.lock().unwrap();

    let volume = volume.clamp(0.0, 1.0);
    app_state.master_volume = volume;
    audio.set_master_volume(volume);

    let _ = save_state(&app_state);
    Ok(())
}

#[tauri::command]
pub fn pause_all(
    state: State<'_, AppStateMutex>,
    engine: State<'_, AudioEngineMutex>,
) -> Result<(), String> {
    let mut app_state = state.0.lock().unwrap();
    let mut audio = engine.0.lock().unwrap();

    app_state.is_paused = true;
    audio.pause_all();

    let _ = save_state(&app_state);
    Ok(())
}

#[tauri::command]
pub fn resume_all(
    state: State<'_, AppStateMutex>,
    engine: State<'_, AudioEngineMutex>,
) -> Result<(), String> {
    let mut app_state = state.0.lock().unwrap();
    let mut audio = engine.0.lock().unwrap();

    app_state.is_paused = false;
    audio.resume_all();

    let _ = save_state(&app_state);
    Ok(())
}

#[tauri::command]
pub fn import_sound(
    path: String,
    state: State<'_, AppStateMutex>,
) -> Result<Sound, String> {
    let source_path = std::path::Path::new(&path);

    // Validate file extension
    let ext = source_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    if ext != "mp3" && ext != "wav" {
        return Err("Only MP3 and WAV files are supported".to_string());
    }

    // Copy file to user sounds directory
    let dest_dir = user_sounds_dir();
    fs::create_dir_all(&dest_dir).map_err(|e| format!("Failed to create sounds dir: {}", e))?;

    let file_name = source_path
        .file_name()
        .ok_or("Invalid file path")?;
    let dest_path = dest_dir.join(file_name);

    // Don't overwrite existing files — add a UUID suffix if needed
    let dest_path = if dest_path.exists() {
        let stem = source_path.file_stem().unwrap().to_string_lossy();
        let unique = format!("{}_{}.{}", stem, &Uuid::new_v4().to_string()[..8], ext);
        dest_dir.join(unique)
    } else {
        dest_path
    };

    fs::copy(source_path, &dest_path).map_err(|e| format!("Failed to copy file: {}", e))?;

    let stem = dest_path.file_stem().unwrap().to_string_lossy().to_string();
    let name = {
        let mut chars = stem.chars();
        match chars.next() {
            None => String::new(),
            Some(c) => c.to_uppercase().to_string() + chars.as_str(),
        }
    };

    let sound = Sound {
        id: format!("user_{}", stem),
        name,
        file_path: dest_path,
        is_bundled: false,
        is_active: false,
        volume: 0.7,
    };

    let mut app_state = state.0.lock().unwrap();
    app_state.sounds.push(sound.clone());
    let _ = save_state(&app_state);

    Ok(sound)
}

#[tauri::command]
pub fn remove_sound(
    id: String,
    state: State<'_, AppStateMutex>,
    engine: State<'_, AudioEngineMutex>,
) -> Result<(), String> {
    let mut app_state = state.0.lock().unwrap();
    let mut audio = engine.0.lock().unwrap();

    let sound = app_state
        .sounds
        .iter()
        .find(|s| s.id == id)
        .ok_or_else(|| format!("Sound not found: {}", id))?;

    if sound.is_bundled {
        return Err("Cannot remove bundled sounds".to_string());
    }

    // Stop playback if active
    audio.stop_sound(&id);

    // Delete the file
    let file_path = sound.file_path.clone();
    if file_path.exists() {
        let _ = fs::remove_file(&file_path);
    }

    app_state.sounds.retain(|s| s.id != id);
    let _ = save_state(&app_state);

    Ok(())
}

#[tauri::command]
pub fn set_crossfade_duration(
    duration: f32,
    state: State<'_, AppStateMutex>,
) -> Result<(), String> {
    let mut app_state = state.0.lock().unwrap();
    app_state.crossfade_duration = duration.clamp(0.5, 5.0);
    let _ = save_state(&app_state);
    Ok(())
}
```

- [ ] **Step 2: Wire commands into lib.rs**

Replace `src-tauri/src/lib.rs`:

```rust
mod audio;
mod commands;
mod crossfade;
mod state;

use audio::AudioEngine;
use commands::{AppStateMutex, AudioEngineMutex};
use state::{discover_bundled_sounds, discover_user_sounds, load_persisted_state, AppState};
use std::sync::Mutex;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_positioner::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .setup(|app| {
            // Hide from Dock on macOS
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            // Discover sounds
            let resource_path = app
                .path()
                .resource_dir()
                .expect("Failed to get resource dir");
            let bundled_sounds_dir = resource_path.join("sounds");

            let mut sounds = discover_bundled_sounds(&bundled_sounds_dir);
            let mut user_sounds = discover_user_sounds();
            sounds.append(&mut user_sounds);

            // Build initial state
            let mut app_state = AppState {
                sounds,
                ..AppState::default()
            };

            // Apply persisted state (volumes, active sounds, etc.)
            if let Some(persisted) = load_persisted_state() {
                app_state.apply_persisted(&persisted);
            }

            // Initialize audio engine
            let mut audio_engine =
                AudioEngine::new().expect("Failed to initialize audio engine");
            audio_engine.set_master_volume(app_state.master_volume);

            // Start playing any sounds that were active when the app was last closed
            if !app_state.is_paused {
                for sound in &app_state.sounds {
                    if sound.is_active {
                        let _ = audio_engine.play_sound(
                            &sound.id,
                            &sound.file_path,
                            sound.volume,
                            app_state.crossfade_duration,
                        );
                    }
                }
            }

            app.manage(AppStateMutex(Mutex::new(app_state)));
            app.manage(AudioEngineMutex(Mutex::new(audio_engine)));

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_state,
            commands::toggle_sound,
            commands::set_volume,
            commands::set_master_volume,
            commands::pause_all,
            commands::resume_all,
            commands::import_sound,
            commands::remove_sound,
            commands::set_crossfade_duration,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 3: Bundle sounds as Tauri resources**

Add to `src-tauri/tauri.conf.json` inside the `"bundle"` object:

```json
"resources": [
  "../assets/sounds/*"
]
```

This makes the sound files available at runtime via `app.path().resource_dir()`.

- [ ] **Step 4: Verify it compiles**

```bash
cargo build --manifest-path src-tauri/Cargo.toml
```

Expected: Compiles. There may be warnings about unused imports — that's OK.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/commands.rs src-tauri/src/lib.rs src-tauri/tauri.conf.json
git commit -m "feat: add Tauri commands for sound control, import, and state management"
```

---

## Task 6: Tray Icon & Popover Window

**Files:**
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Add tray icon click handler to toggle popover**

In `src-tauri/src/lib.rs`, add tray event handling inside the `.setup()` closure, after managing the state:

```rust
            // Set up tray icon click to toggle popover window
            let app_handle = app.handle().clone();
            app.on_tray_icon_event(move |_tray, event| {
                use tauri::tray::{MouseButton, MouseButtonState, TrayIconEvent};
                use tauri_plugin_positioner::{Position, WindowExt};

                if let TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                } = event
                {
                    if let Some(window) = app_handle.get_webview_window("main") {
                        if window.is_visible().unwrap_or(false) {
                            let _ = window.hide();
                        } else {
                            let _ = window.move_window(Position::TrayCenter);
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                }
            });
```

- [ ] **Step 2: Add window focus-lost handler to close popover**

Add after the tray event handler, still inside `.setup()`:

```rust
            // Close popover when it loses focus
            let main_window = app.get_webview_window("main").unwrap();
            main_window.on_window_event(move |event| {
                if let tauri::WindowEvent::Focused(false) = event {
                    // Window lost focus — hide the popover
                    // (The window variable is moved into the closure via the setup scope)
                }
            });
```

Actually, Tauri v2's `on_window_event` requires a different approach. Use the builder-level handler instead. Add this to the Builder chain (before `.run()`):

```rust
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::Focused(false) = event {
                if window.label() == "main" {
                    let _ = window.hide();
                }
            }
        })
```

- [ ] **Step 3: Verify tray and popover work**

```bash
cargo tauri dev
```

Expected:
- Tray icon visible in menu bar
- Left-click toggles a small window near the tray icon
- Clicking away from the window hides it
- No Dock icon visible

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/lib.rs
git commit -m "feat: add tray icon click handler and popover window management"
```

---

## Task 7: Frontend — CSS Theme & App Shell

**Files:**
- Modify: `src/app.css`
- Modify: `src/App.svelte`
- Modify: `index.html`

- [ ] **Step 1: Write Minimal Mono CSS theme**

Replace `src/app.css`:

```css
:root {
  --bg: #ffffff;
  --bg-secondary: #f2f2f7;
  --text: #1d1d1f;
  --text-secondary: #aeaeb2;
  --separator: #e5e5ea;
  --dot-active: #1d1d1f;
  --dot-inactive: #d1d1d6;
  --fader-track: #f2f2f7;
  --fader-fill: #1d1d1f;
}

@media (prefers-color-scheme: dark) {
  :root {
    --bg: #000000;
    --bg-secondary: #1c1c1e;
    --text: #f5f5f7;
    --text-secondary: #636366;
    --separator: #2c2c2e;
    --dot-active: #f5f5f7;
    --dot-inactive: #3a3a3c;
    --fader-track: #1c1c1e;
    --fader-fill: #f5f5f7;
  }
}

* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

html, body {
  font-family: -apple-system, BlinkMacSystemFont, system-ui, sans-serif;
  font-size: 13px;
  color: var(--text);
  background: var(--bg);
  overflow: hidden;
  user-select: none;
  -webkit-user-select: none;
  height: 100%;
}

#app {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.section-label {
  font-size: 9px;
  font-weight: 500;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: var(--text-secondary);
  margin-bottom: 6px;
}

button {
  background: none;
  border: none;
  color: var(--text-secondary);
  cursor: pointer;
  font-size: 13px;
  padding: 2px;
}

button:hover {
  color: var(--text);
}
```

- [ ] **Step 2: Update index.html**

Ensure `index.html` has a clean structure:

```html
<!doctype html>
<html lang="de">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>BackBird</title>
    <link rel="stylesheet" href="/src/app.css" />
  </head>
  <body>
    <div id="app"></div>
    <script type="module" src="/src/main.js"></script>
  </body>
</html>
```

- [ ] **Step 3: Write App.svelte shell**

Replace `src/App.svelte`:

```svelte
<script>
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';
  import Header from './lib/Header.svelte';
  import Mixer from './lib/Mixer.svelte';
  import SoundList from './lib/SoundList.svelte';
  import Settings from './lib/Settings.svelte';

  let state = $state(null);
  let showSettings = $state(false);

  onMount(async () => {
    state = await invoke('get_state');
  });

  async function refreshState() {
    state = await invoke('get_state');
  }

  function toggleSettings() {
    showSettings = !showSettings;
  }
</script>

{#if state}
  <Header
    isPaused={state.is_paused}
    {showSettings}
    onTogglePause={async () => {
      if (state.is_paused) {
        await invoke('resume_all');
      } else {
        await invoke('pause_all');
      }
      await refreshState();
    }}
    onToggleSettings={toggleSettings}
  />

  {#if showSettings}
    <Settings
      autostartEnabled={state.autostart_enabled}
      crossfadeDuration={state.crossfade_duration}
      onBack={() => showSettings = false}
    />
  {:else}
    {#if state.sounds.some(s => s.is_active)}
      <Mixer
        sounds={state.sounds.filter(s => s.is_active)}
        masterVolume={state.master_volume}
        onVolumeChange={async (id, volume) => {
          await invoke('set_volume', { id, volume });
          await refreshState();
        }}
        onMasterVolumeChange={async (volume) => {
          await invoke('set_master_volume', { volume });
          await refreshState();
        }}
      />
    {/if}

    <SoundList
      sounds={state.sounds}
      hasActiveSounds={state.sounds.some(s => s.is_active)}
      onToggle={async (id) => {
        state = await invoke('toggle_sound', { id });
      }}
      onImport={refreshState}
      onRemove={async (id) => {
        await invoke('remove_sound', { id });
        await refreshState();
      }}
    />
  {/if}
{/if}
```

- [ ] **Step 4: Create placeholder components**

Create `src/lib/Header.svelte`:

```svelte
<script>
  let { isPaused, showSettings, onTogglePause, onToggleSettings } = $props();
</script>

<header>
  <span class="title">BackBird</span>
  <div class="actions">
    <button onclick={onTogglePause} title={isPaused ? 'Resume' : 'Pause'}>
      {isPaused ? '▶' : '⏸'}
    </button>
    <button onclick={onToggleSettings} title="Settings" class:active={showSettings}>
      ⚙
    </button>
  </div>
</header>

<style>
  header {
    display: flex;
    align-items: center;
    padding: 12px 14px 8px;
  }
  .title {
    flex: 1;
    font-size: 13px;
    font-weight: 600;
  }
  .actions {
    display: flex;
    gap: 6px;
  }
  .active {
    color: var(--text);
  }
</style>
```

Create `src/lib/Mixer.svelte`:

```svelte
<script>
  let { sounds, masterVolume, onVolumeChange, onMasterVolumeChange } = $props();
</script>

<section class="mixer">
  <div class="section-label">Aktiver Mix</div>

  <div class="faders">
    {#each sounds as sound (sound.id)}
      <div class="fader-column">
        <span class="fader-label">{sound.name}</span>
        <div class="fader-track">
          <div class="fader-fill" style="height: {sound.volume * 100}%"></div>
        </div>
        <input
          type="range"
          min="0"
          max="100"
          value={Math.round(sound.volume * 100)}
          orient="vertical"
          oninput={(e) => onVolumeChange(sound.id, parseInt(e.target.value) / 100)}
        />
      </div>
    {/each}
  </div>

  <div class="master">
    <span class="master-icon">🔈</span>
    <input
      type="range"
      min="0"
      max="100"
      value={Math.round(masterVolume * 100)}
      oninput={(e) => onMasterVolumeChange(parseInt(e.target.value) / 100)}
    />
    <span class="master-icon">🔊</span>
  </div>
</section>

<style>
  .mixer {
    padding: 0 14px 8px;
    border-bottom: 0.5px solid var(--separator);
  }
  .faders {
    display: flex;
    gap: 16px;
    justify-content: center;
    padding: 8px 0;
  }
  .fader-column {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
    position: relative;
  }
  .fader-label {
    font-size: 11px;
    max-width: 70px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    text-align: center;
  }
  .fader-track {
    width: 3px;
    height: 60px;
    background: var(--fader-track);
    border-radius: 2px;
    position: relative;
    overflow: hidden;
  }
  .fader-fill {
    position: absolute;
    bottom: 0;
    width: 100%;
    background: var(--fader-fill);
    border-radius: 2px;
    transition: height 0.1s ease;
  }
  .fader-column input[type="range"] {
    position: absolute;
    top: 20px;
    width: 60px;
    height: 3px;
    opacity: 0;
    cursor: pointer;
    writing-mode: vertical-lr;
    direction: rtl;
  }
  .master {
    display: flex;
    align-items: center;
    gap: 6px;
    justify-content: center;
    padding-top: 8px;
  }
  .master-icon {
    font-size: 10px;
    color: var(--text-secondary);
  }
  .master input[type="range"] {
    width: 120px;
    height: 2px;
    -webkit-appearance: none;
    appearance: none;
    background: var(--fader-track);
    border-radius: 2px;
    outline: none;
    cursor: pointer;
  }
  .master input[type="range"]::-webkit-slider-thumb {
    -webkit-appearance: none;
    appearance: none;
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--fader-fill);
    cursor: pointer;
  }
</style>
```

Create `src/lib/SoundList.svelte`:

```svelte
<script>
  import { invoke } from '@tauri-apps/api/core';
  import { open } from '@tauri-apps/plugin-dialog';

  let { sounds, hasActiveSounds, onToggle, onImport, onRemove } = $props();

  async function handleImport() {
    const path = await open({
      filters: [{ name: 'Audio', extensions: ['mp3', 'wav'] }],
    });
    if (path) {
      await invoke('import_sound', { path });
      onImport();
    }
  }
</script>

<section class="sound-list" class:expanded={!hasActiveSounds}>
  <div class="section-label">Sounds</div>

  <div class="list">
    {#each sounds as sound (sound.id)}
      <button class="sound-row" onclick={() => onToggle(sound.id)}>
        <span class="sound-name" class:inactive={!sound.is_active}>
          {sound.name}
        </span>
        {#if !sound.is_bundled}
          <button
            class="remove-btn"
            onclick|stopPropagation={() => onRemove(sound.id)}
            title="Remove"
          >×</button>
        {/if}
        <span class="dot" class:active={sound.is_active}>
          {sound.is_active ? '●' : '○'}
        </span>
      </button>
    {/each}
  </div>

  <button class="import-btn" onclick={handleImport}>
    ＋ Hinzufügen
  </button>
</section>

<style>
  .sound-list {
    flex: 1;
    display: flex;
    flex-direction: column;
    padding: 8px 14px;
    min-height: 0;
  }
  .expanded {
    padding-top: 0;
  }
  .list {
    flex: 1;
    overflow-y: auto;
    min-height: 0;
  }
  .sound-row {
    display: flex;
    align-items: center;
    padding: 5px 0;
    border-bottom: 0.5px solid var(--separator);
    width: 100%;
    text-align: left;
    font-size: 12px;
  }
  .sound-name {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .sound-name.inactive {
    color: var(--text-secondary);
  }
  .remove-btn {
    font-size: 14px;
    padding: 0 4px;
    color: var(--text-secondary);
  }
  .remove-btn:hover {
    color: var(--text);
  }
  .dot {
    font-size: 10px;
    color: var(--dot-inactive);
    margin-left: 4px;
  }
  .dot.active {
    color: var(--dot-active);
  }
  .import-btn {
    display: block;
    width: 100%;
    text-align: center;
    padding: 8px;
    font-size: 11px;
    color: var(--text-secondary);
  }
</style>
```

Create `src/lib/Settings.svelte`:

```svelte
<script>
  import { invoke } from '@tauri-apps/api/core';
  import { enable, disable, isEnabled } from '@tauri-apps/plugin-autostart';

  let { autostartEnabled, crossfadeDuration, onBack } = $props();
  let autostart = $state(autostartEnabled);
  let cfDuration = $state(crossfadeDuration);

  async function toggleAutostart() {
    if (autostart) {
      await disable();
    } else {
      await enable();
    }
    autostart = !autostart;
  }

  async function updateCrossfade(value) {
    cfDuration = value;
    await invoke('set_crossfade_duration', { duration: value });
  }
</script>

<section class="settings">
  <button class="back-btn" onclick={onBack}>← Back</button>

  <div class="setting-row">
    <span>Autostart</span>
    <button class="toggle" onclick={toggleAutostart}>
      {autostart ? 'An' : 'Aus'}
    </button>
  </div>

  <div class="setting-row">
    <span>Crossfade</span>
    <div class="cf-control">
      <input
        type="range"
        min="50"
        max="500"
        value={Math.round(cfDuration * 100)}
        oninput={(e) => updateCrossfade(parseInt(e.target.value) / 100)}
      />
      <span class="cf-value">{cfDuration.toFixed(1)}s</span>
    </div>
  </div>

  <div class="about">
    <div class="about-title">BackBird</div>
    <div class="about-version">Version 0.1.0</div>
    <div class="about-desc">Ambient bird songs for focus & relaxation.</div>
  </div>
</section>

<style>
  .settings {
    flex: 1;
    padding: 8px 14px;
  }
  .back-btn {
    font-size: 11px;
    margin-bottom: 12px;
  }
  .setting-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 0;
    border-bottom: 0.5px solid var(--separator);
    font-size: 12px;
  }
  .toggle {
    font-size: 12px;
    padding: 2px 8px;
    border: 0.5px solid var(--separator);
    border-radius: 4px;
  }
  .cf-control {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .cf-control input[type="range"] {
    width: 80px;
    height: 2px;
    -webkit-appearance: none;
    appearance: none;
    background: var(--fader-track);
    border-radius: 2px;
    outline: none;
    cursor: pointer;
  }
  .cf-control input[type="range"]::-webkit-slider-thumb {
    -webkit-appearance: none;
    appearance: none;
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--fader-fill);
  }
  .cf-value {
    font-size: 11px;
    color: var(--text-secondary);
    width: 30px;
  }
  .about {
    margin-top: 24px;
    text-align: center;
    color: var(--text-secondary);
  }
  .about-title {
    font-size: 13px;
    font-weight: 600;
    color: var(--text);
  }
  .about-version {
    font-size: 10px;
    margin-top: 2px;
  }
  .about-desc {
    font-size: 10px;
    margin-top: 4px;
  }
</style>
```

- [ ] **Step 5: Verify frontend renders**

```bash
cargo tauri dev
```

Expected: Clicking the tray icon shows a small popover with "BackBird" header, a sound list with the 6 bundled sounds, and a "＋ Hinzufügen" button. No mixer section visible (no sounds active yet).

- [ ] **Step 6: Commit**

```bash
git add src/ index.html
git commit -m "feat: add Svelte frontend with Minimal Mono theme, all components"
```

---

## Task 8: Integration & End-to-End Testing

**Files:**
- Possibly fix: any files from previous tasks

- [ ] **Step 1: Test sound activation**

Run `cargo tauri dev`. Click a sound in the list (e.g., "Blackforest"). Expected:
- Sound starts playing with a 1-second fade-in
- The dot changes from ○ to ●
- The Mixer section appears with a vertical fader for "Blackforest"
- The Master Volume slider appears

- [ ] **Step 2: Test multi-track mixing**

Click a second sound (e.g., "Finland"). Expected:
- Both sounds play simultaneously
- Mixer shows two faders
- Adjusting one fader changes only that sound's volume

- [ ] **Step 3: Test master volume**

Drag the master volume slider. Expected:
- All active sounds change volume proportionally

- [ ] **Step 4: Test pause/resume**

Click the ⏸ button. Expected:
- All sounds pause
- Button changes to ▶
- Click ▶ — sounds resume

- [ ] **Step 5: Test deactivation**

Click an active sound in the list. Expected:
- Sound stops (ideally with fade-out)
- Dot changes back to ○
- Fader disappears from mixer
- If last active sound: mixer section hides

- [ ] **Step 6: Test sound import**

Click "＋ Hinzufügen". Expected:
- macOS file picker opens, filtered to MP3/WAV
- Select a file — it appears in the sound list
- File is copied to `~/Library/Application Support/BackBird/sounds/`

- [ ] **Step 7: Test state persistence**

Activate 2 sounds, adjust volumes, then quit the app (`Cmd+Q` or tray right-click → Quit). Relaunch with `cargo tauri dev`. Expected:
- Same sounds are active with same volumes
- Master volume is restored
- Sounds start playing automatically

- [ ] **Step 8: Test settings**

Click ⚙. Expected:
- Settings view shows with back button, autostart toggle, crossfade slider, about info
- Back button returns to main view
- Crossfade slider changes persist

- [ ] **Step 9: Test light/dark mode**

Switch macOS appearance in System Settings → Appearance. Expected:
- UI switches between light and dark theme immediately
- All colors update correctly (background, text, separators, fader fills)

- [ ] **Step 10: Fix any issues found during testing**

Address bugs or UI issues discovered during steps 1-9. Common issues:
- Volume slider not dragging smoothly → check range input styling
- Popover not positioning correctly → verify `tauri-plugin-positioner` setup
- Sound files not found → check resource bundling path in `tauri.conf.json`
- Crossfade click/pop at loop point → adjust `CrossfadeLoop` algorithm

- [ ] **Step 11: Commit all fixes**

```bash
git add -A
git commit -m "fix: integration testing fixes for audio, UI, and state persistence"
```

---

## Task 9: Polish & Final Build

**Files:**
- Modify: various files for polish

- [ ] **Step 1: Enable autostart on first launch**

In `src-tauri/src/lib.rs`, inside `.setup()`, after managing state, add:

```rust
            // Enable autostart if configured
            if app_state.autostart_enabled {
                let _ = app.autolaunch().enable();
            }
```

(Where `app_state` is the state before it's moved into the Mutex. You'll need to read the value before `app.manage()`.)

- [ ] **Step 2: Save state on quit**

Add to the Builder chain in `lib.rs`:

```rust
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::Destroyed = event {
                if window.label() == "main" {
                    if let Some(state) = window.try_state::<AppStateMutex>() {
                        let app_state = state.0.lock().unwrap();
                        let _ = crate::state::save_state(&app_state);
                    }
                }
            }
            // Also keep the focus-lost handler
            if let tauri::WindowEvent::Focused(false) = event {
                if window.label() == "main" {
                    let _ = window.hide();
                }
            }
        })
```

- [ ] **Step 3: Test full build**

```bash
cargo tauri build
```

Expected: Produces a `.app` bundle in `src-tauri/target/release/bundle/macos/`. The app should:
- Show tray icon
- Not appear in Dock
- Play sounds from bundled resources
- Persist state across launches

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "feat: add autostart, save-on-quit, production build config"
```

---

## Verification Checklist

After all tasks, verify against the spec:

1. ✅ `cargo tauri dev` starts the app with tray icon visible in menu bar
2. ✅ Clicking tray icon opens the popover
3. ✅ Toggling a sound in the list starts playback with fade-in; it appears in the mixer
4. ✅ Adjusting a fader changes that sound's volume in real-time
5. ✅ Master volume affects all active sounds proportionally
6. ✅ Deactivating a sound fades it out and removes it from the mixer
7. ✅ Looping a sound produces a seamless crossfade at the loop point
8. ✅ Multiple sounds play simultaneously without distortion
9. ✅ "＋ Hinzufügen" opens file picker, imported MP3/WAV appears in the list
10. ✅ Quitting and relaunching restores the exact same mix state
11. ✅ App starts automatically after macOS login
12. ✅ Light/Dark mode switch updates the UI immediately
