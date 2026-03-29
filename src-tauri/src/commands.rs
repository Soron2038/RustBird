// Lock ordering convention:
// Always acquire AppStateMutex before AudioEngineMutex — never in reverse order.
// This prevents deadlocks. Holding both simultaneously is allowed when needed (e.g. set_volume),
// but the acquisition order must always be AppStateMutex first.

use crate::audio::AudioEngine;
use crate::error::AppError;
use crate::state::{capitalize_first, save_state, user_sounds_dir, AppState, Sound};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager, State};
use uuid::Uuid;

pub struct AppStateMutex(pub Mutex<AppState>);
pub struct AudioEngineMutex(pub Mutex<AudioEngine>);

// SAFETY: AudioEngine contains rodio's OutputStream which holds a *mut () via CoreAudio's
// NotSendSyncAcrossAllPlatforms.  We wrap it in a Mutex so concurrent access is impossible,
// and the pointer is only ever touched on the thread that locked the Mutex.
unsafe impl Send for AudioEngineMutex {}
unsafe impl Sync for AudioEngineMutex {}

#[tauri::command]
pub fn get_state(state: State<'_, AppStateMutex>) -> Result<AppState, AppError> {
    let app_state = state
        .0
        .lock()
        .map_err(|_| AppError::Audio("State lock poisoned".into()))?;
    Ok(app_state.clone())
}

#[tauri::command]
pub fn toggle_sound(
    id: String,
    app_handle: AppHandle,
    state: State<'_, AppStateMutex>,
    audio: State<'_, AudioEngineMutex>,
) -> Result<AppState, AppError> {
    // Read info and update state immediately so the UI responds instantly
    let (was_active, file_path, volume, is_paused, crossfade) = {
        let mut app_state = state
            .0
            .lock()
            .map_err(|_| AppError::Audio("State lock poisoned".into()))?;
        let is_paused = app_state.is_paused;
        let crossfade = app_state.crossfade_duration;
        let sound = app_state
            .sounds
            .iter_mut()
            .find(|s| s.id == id)
            .ok_or_else(|| AppError::SoundNotFound(id.clone()))?;
        let was_active = sound.is_active;
        let file_path = sound.file_path.clone();
        let volume = sound.volume;
        sound.is_active = !was_active;
        let _ = save_state(&app_state);
        (was_active, file_path, volume, is_paused, crossfade)
    };

    // Stop is fast — do it inline
    if was_active {
        let mut audio_engine = audio
            .0
            .lock()
            .map_err(|_| AppError::Audio("Audio lock poisoned".into()))?;
        audio_engine.stop_sound(&id);
    } else if !is_paused {
        // Decode + play is slow — spawn on background thread so UI doesn't freeze
        let id_clone = id.clone();
        std::thread::spawn(move || {
            let audio_state = app_handle.state::<AudioEngineMutex>();
            let mut audio_engine = audio_state.0.lock().unwrap_or_else(|e| e.into_inner());
            if let Err(e) = audio_engine.play_sound(&id_clone, &file_path, volume, crossfade) {
                log::error!("Failed to play sound {}: {}", id_clone, e);
                // Release audio lock before acquiring state lock (lock ordering)
                drop(audio_engine);
                let state = app_handle.state::<AppStateMutex>();
                let mut app_state = state.0.lock().unwrap_or_else(|e| e.into_inner());
                if let Some(sound) = app_state.sounds.iter_mut().find(|s| s.id == id_clone) {
                    sound.is_active = false;
                }
                let _ = save_state(&app_state);
                let _ = app_handle.emit("sound-playback-failed", &id_clone);
            }
        });
    }

    // Return updated state immediately — audio loads in background
    let app_state = state
        .0
        .lock()
        .map_err(|_| AppError::Audio("State lock poisoned".into()))?;
    Ok(app_state.clone())
}

#[tauri::command]
pub fn set_volume(
    id: String,
    volume: f32,
    state: State<'_, AppStateMutex>,
    audio: State<'_, AudioEngineMutex>,
) -> Result<AppState, AppError> {
    let mut app_state = state
        .0
        .lock()
        .map_err(|_| AppError::Audio("State lock poisoned".into()))?;
    let mut audio_engine = audio
        .0
        .lock()
        .map_err(|_| AppError::Audio("Audio lock poisoned".into()))?;

    let sound = app_state
        .sounds
        .iter_mut()
        .find(|s| s.id == id)
        .ok_or_else(|| AppError::SoundNotFound(id.clone()))?;

    sound.volume = volume;
    audio_engine.set_volume(&id, volume);

    save_state(&app_state)?;
    Ok(app_state.clone())
}

#[tauri::command]
pub fn set_master_volume(
    volume: f32,
    state: State<'_, AppStateMutex>,
    audio: State<'_, AudioEngineMutex>,
) -> Result<AppState, AppError> {
    let mut app_state = state
        .0
        .lock()
        .map_err(|_| AppError::Audio("State lock poisoned".into()))?;
    let mut audio_engine = audio
        .0
        .lock()
        .map_err(|_| AppError::Audio("Audio lock poisoned".into()))?;

    app_state.master_volume = volume;
    audio_engine.set_master_volume(volume);

    save_state(&app_state)?;
    Ok(app_state.clone())
}

#[tauri::command]
pub fn pause_all(
    state: State<'_, AppStateMutex>,
    audio: State<'_, AudioEngineMutex>,
) -> Result<AppState, AppError> {
    let mut app_state = state
        .0
        .lock()
        .map_err(|_| AppError::Audio("State lock poisoned".into()))?;
    let audio_engine = audio
        .0
        .lock()
        .map_err(|_| AppError::Audio("Audio lock poisoned".into()))?;

    app_state.is_paused = true;
    audio_engine.pause_all();

    save_state(&app_state)?;
    Ok(app_state.clone())
}

#[tauri::command]
pub fn resume_all(
    state: State<'_, AppStateMutex>,
    audio: State<'_, AudioEngineMutex>,
) -> Result<AppState, AppError> {
    let mut app_state = state
        .0
        .lock()
        .map_err(|_| AppError::Audio("State lock poisoned".into()))?;
    let audio_engine = audio
        .0
        .lock()
        .map_err(|_| AppError::Audio("Audio lock poisoned".into()))?;

    app_state.is_paused = false;
    audio_engine.resume_all();

    save_state(&app_state)?;
    Ok(app_state.clone())
}

#[tauri::command]
pub fn import_sound(
    file_path: String,
    state: State<'_, AppStateMutex>,
) -> Result<AppState, AppError> {
    let src_path = PathBuf::from(&file_path);

    let ext = src_path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .ok_or(AppError::InvalidFilename)?;

    if ext != "mp3" && ext != "wav" {
        return Err(AppError::UnsupportedFileType(ext));
    }

    let dest_dir = user_sounds_dir()?;
    fs::create_dir_all(&dest_dir)?;

    // Determine destination filename, deduplicating with a UUID suffix if needed
    let stem = src_path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or(AppError::InvalidFilename)?
        .to_string();

    let candidate = dest_dir.join(format!("{}.{}", stem, ext));
    let dest_path = if candidate.exists() {
        let unique_stem = format!("{}_{}", stem, Uuid::new_v4());
        dest_dir.join(format!("{}.{}", unique_stem, ext))
    } else {
        candidate
    };

    fs::copy(&src_path, &dest_path)?;

    let final_stem = dest_path
        .file_stem()
        .ok_or(AppError::InvalidFilename)?
        .to_string_lossy()
        .to_string();

    let name = capitalize_first(&final_stem);

    let new_sound = Sound {
        id: format!("user_{}", final_stem),
        name,
        file_path: dest_path,
        is_bundled: false,
        is_active: false,
        volume: 0.7,
    };

    let mut app_state = state
        .0
        .lock()
        .map_err(|_| AppError::Audio("State lock poisoned".into()))?;
    app_state.sounds.push(new_sound);
    app_state.sounds.sort_by(|a, b| a.name.cmp(&b.name));

    save_state(&app_state)?;
    Ok(app_state.clone())
}

#[tauri::command]
pub fn remove_sound(
    id: String,
    state: State<'_, AppStateMutex>,
    audio: State<'_, AudioEngineMutex>,
) -> Result<AppState, AppError> {
    let mut app_state = state
        .0
        .lock()
        .map_err(|_| AppError::Audio("State lock poisoned".into()))?;

    let sound = app_state
        .sounds
        .iter()
        .find(|s| s.id == id)
        .ok_or_else(|| AppError::SoundNotFound(id.clone()))?;

    if sound.is_bundled {
        return Err(AppError::BundledSoundRemoval);
    }

    let file_path = sound.file_path.clone();

    // Stop playback first (acquire audio lock, then release before continuing)
    {
        let mut audio_engine = audio
            .0
            .lock()
            .map_err(|_| AppError::Audio("Audio lock poisoned".into()))?;
        audio_engine.stop_sound(&id);
    }

    // Remove the file from disk
    if file_path.exists() {
        fs::remove_file(&file_path)?;
    }

    app_state.sounds.retain(|s| s.id != id);

    save_state(&app_state)?;
    Ok(app_state.clone())
}

#[tauri::command]
pub fn set_crossfade_duration(
    duration: f32,
    state: State<'_, AppStateMutex>,
) -> Result<(), AppError> {
    let mut app_state = state
        .0
        .lock()
        .map_err(|_| AppError::Audio("State lock poisoned".into()))?;
    app_state.crossfade_duration = duration;
    save_state(&app_state)?;
    Ok(())
}

#[tauri::command]
pub fn set_autopause_on_lock(
    enabled: bool,
    state: State<'_, AppStateMutex>,
) -> Result<AppState, AppError> {
    let mut app_state = state
        .0
        .lock()
        .map_err(|_| AppError::Audio("State lock poisoned".into()))?;
    app_state.autopause_on_lock = enabled;
    save_state(&app_state)?;
    Ok(app_state.clone())
}

#[tauri::command]
pub fn set_dialog_open(state: State<'_, AppStateMutex>) -> Result<(), AppError> {
    let mut app_state = state
        .0
        .lock()
        .map_err(|_| AppError::Audio("State lock poisoned".into()))?;
    app_state.dialog_open = true;
    Ok(())
}

#[tauri::command]
pub fn set_dialog_closed(state: State<'_, AppStateMutex>) -> Result<(), AppError> {
    let mut app_state = state
        .0
        .lock()
        .map_err(|_| AppError::Audio("State lock poisoned".into()))?;
    app_state.dialog_open = false;
    Ok(())
}
