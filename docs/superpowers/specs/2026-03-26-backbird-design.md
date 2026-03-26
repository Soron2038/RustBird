# BackBird -- Design Specification

## Overview

BackBird is a macOS menu bar application that plays ambient bird song recordings in the background to help the user relax while working. It lives entirely in the system tray and provides a popover UI for controlling a multi-track sound mixer.

## Core Requirements

- **Menu bar app** -- no Dock icon, no main window. Only a tray icon and popover.
- **Multi-track mixing** -- multiple sounds can play simultaneously, each with individual volume control.
- **Looping with crossfade** -- sounds loop seamlessly with a crossfade transition to avoid hard cuts.
- **Sound import** -- users can add their own MP3/WAV files via a native file dialog.
- **State persistence** -- active sounds, volumes, and master volume are saved and restored on launch.
- **Autostart** -- app starts automatically on macOS login (configurable in settings).
- **Light/Dark mode** -- follows macOS system appearance automatically.

## Architecture

**Framework:** Tauri v2 (Rust backend + WebView frontend)

### Rust Backend (src-tauri/)

Handles audio playback, state management, and persistence.

**Key crates:**
- `tauri` v2 -- app framework, tray, window management
- `rodio` -- audio decoding (MP3/WAV), playback, mixing
- `serde` / `serde_json` -- state serialization
- `tauri-plugin-autostart` -- login item registration

**Modules:**
- `audio.rs` -- Audio engine: mixer, per-sound Sink management, crossfade logic, master volume
- `state.rs` -- AppState struct, load/save to JSON, sound registry
- `commands.rs` -- Tauri command handlers exposed to the frontend

### Svelte Frontend (src/)

Renders the popover UI. Communicates with the backend via Tauri `invoke()` calls and listens to backend events.

**Components:**
- `App.svelte` -- Root component, state binding, event listeners
- `Header.svelte` -- App title ("BackBird"), pause/resume button, settings icon
- `Mixer.svelte` -- Vertical faders for each active sound + horizontal master volume slider
- `SoundList.svelte` -- Scrollable list of all available sounds with toggle indicators

## Tauri Commands (Backend API)

| Command | Parameters | Returns | Description |
|---------|-----------|---------|-------------|
| `get_state` | -- | `AppState` | Full current state on app init |
| `toggle_sound` | `id: String` | `AppState` | Activate/deactivate a sound (with fade in/out) |
| `set_volume` | `id: String, volume: f32` | -- | Set individual sound volume (0.0 - 1.0) |
| `set_master_volume` | `volume: f32` | -- | Set master volume (0.0 - 1.0) |
| `pause_all` | -- | -- | Pause all active sounds |
| `resume_all` | -- | -- | Resume all paused sounds |
| `import_sound` | `path: String` | `Sound` | Import a sound file (copies to app data dir) |
| `remove_sound` | `id: String` | -- | Remove a user-imported sound |

## Data Model

```rust
struct AppState {
    sounds: Vec<Sound>,
    master_volume: f32,   // 0.0 - 1.0
    is_paused: bool,
}

struct Sound {
    id: String,           // UUID for user sounds, filename-based for bundled
    name: String,         // Display name (derived from filename)
    file_path: PathBuf,   // Absolute path to audio file
    is_bundled: bool,     // true = shipped with app, false = user-imported
    is_active: bool,      // currently playing
    volume: f32,          // 0.0 - 1.0
}
```

## Audio Engine

### Mixing
- Each active sound gets its own `rodio::Sink` on a shared `OutputStream`
- Individual volume is applied per Sink
- Master volume is applied as a multiplier across all Sinks

### Crossfade Looping
- Each sound loops continuously while active
- Loop transition uses a ~2 second crossfade: the ending of the current iteration fades out while the beginning of the next iteration fades in, overlapping
- Sound files are held in memory for gapless looping (~5-10 MB per file is acceptable)

### Activation/Deactivation
- Activating a sound: 1 second fade-in from silence
- Deactivating a sound: 1 second fade-out to silence, then Sink is stopped

## UI Design

### Style: Minimal Mono
- Monochromatic -- no accent colors. Faders, indicators, and text in black (light mode) or white (dark mode)
- Follows macOS system appearance via CSS `prefers-color-scheme`
- Apple system font (`-apple-system`)
- Separator lines instead of card backgrounds
- Subtle, professional, unobtrusive

### Layout (Popover)

```
┌──────────────────────────┐
│ BackBird          ⏸  ⚙  │  ← Header: title, pause, settings
├──────────────────────────┤
│ AKTIVER MIX              │
│                          │
│  Schwarzwald  Finnland   │  ← Vertical faders per active sound
│      ║          ║        │
│      ║          │        │
│      │          │        │
│                          │
│  🔈 ━━━━━━━━━━━━━━ 🔊   │  ← Master volume (horizontal)
├──────────────────────────┤
│ SOUNDS                   │
│ Schwarzwald          ●   │  ← Active (filled dot)
│ Finnland             ●   │
│ Frankreich           ○   │  ← Inactive (empty dot)
│ Malaysia             ○   │
│ ...                      │  ← Scrollable if many sounds
│                          │
│     ＋ Hinzufügen        │  ← Import button (opens file dialog)
└──────────────────────────┘
```

### Mixer Section
- Only shows faders for currently active sounds
- When no sounds are active: section is hidden, sound list fills the popover
- Vertical faders: thin bars (3px wide), fill from bottom
- Labels above each fader (sound name)
- Master volume below as horizontal slider

### Popover Behavior
- Left-click on tray icon toggles the popover open/closed
- Clicking outside the popover closes it
- Popover appears anchored below the tray icon (standard macOS behavior via Tauri)

### Sound List Section
- Scrollable list of all sounds (bundled + imported)
- Each row: sound name + toggle indicator (● active, ○ inactive)
- Click toggles activation (triggers fade in/out)
- "＋ Hinzufügen" opens native macOS file picker (filtered to MP3/WAV)

### Settings View
- Accessible via ⚙ icon in header
- About: App name, version, credits
- Autostart toggle
- Crossfade duration slider (default: 2 seconds, range: 0.5 - 5 seconds)

## Bundled Assets

**Icons (existing):**
- `assets/icons/app-icon.png` (512x512) -- application icon
- `assets/icons/trayIconWhite.png` (22x22) -- menu bar icon
- `assets/icons/trayIconWhite@2x.png` (44x44) -- menu bar icon (Retina)

**Sounds (existing, 6 MP3 files, 192kbps stereo):**
- `assets/sounds/blackforest.mp3` (9.0 MB)
- `assets/sounds/finland.mp3` (3.0 MB)
- `assets/sounds/france.mp3` (5.5 MB)
- `assets/sounds/malaysia.mp3` (3.1 MB)
- `assets/sounds/netherlands.mp3` (4.6 MB)
- `assets/sounds/sweden.mp3` (4.1 MB)

## File Storage

| Path | Content |
|------|---------|
| `~/Library/Application Support/BackBird/config.json` | Persisted AppState (volumes, active sounds, master volume, autostart) |
| `~/Library/Application Support/BackBird/sounds/` | User-imported sound files (copied here on import) |
| App bundle `Resources/sounds/` | Bundled sound files |

## Project Structure

```
BackBird2/
├── assets/
│   ├── icons/              # App + tray icons (existing)
│   └── sounds/             # Bundled bird songs (existing)
├── src-tauri/
│   ├── src/
│   │   ├── main.rs         # Tauri app setup, tray, window config
│   │   ├── audio.rs        # Audio engine, mixing, crossfade
│   │   ├── state.rs        # AppState, persistence (load/save JSON)
│   │   └── commands.rs     # Tauri command handlers
│   ├── Cargo.toml
│   └── tauri.conf.json
├── src/
│   ├── App.svelte          # Root component
│   ├── lib/
│   │   ├── Header.svelte   # Title, pause, settings
│   │   ├── Mixer.svelte    # Active sound faders + master volume
│   │   └── SoundList.svelte # Sound library with toggles
│   ├── main.js             # Svelte mount
│   └── app.css             # Minimal Mono theme, light/dark
├── package.json
└── vite.config.js
```

## Supported Audio Formats

- **MP3** -- bundled sounds and user imports
- **WAV** -- user imports

## Verification

1. `cargo tauri dev` starts the app with tray icon visible in menu bar
2. Clicking tray icon opens the popover
3. Toggling a sound in the list starts playback with fade-in; it appears in the mixer
4. Adjusting a fader changes that sound's volume in real-time
5. Master volume affects all active sounds proportionally
6. Deactivating a sound fades it out and removes it from the mixer
7. Looping a sound produces a seamless crossfade at the loop point
8. Multiple sounds play simultaneously without distortion
9. "＋ Hinzufügen" opens file picker, imported MP3/WAV appears in the list
10. Quitting and relaunching restores the exact same mix state
11. App starts automatically after macOS login
12. Light/Dark mode switch updates the UI immediately
