use crate::audio::AudioEngine;
use crate::state::{save_state, user_sounds_dir, AppState, Sound};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Manager, State};
use uuid::Uuid;

pub struct AppStateMutex(pub Mutex<AppState>);
pub struct AudioEngineMutex(pub Mutex<AudioEngine>);

// SAFETY: AudioEngine contains rodio's OutputStream which holds a *mut () via CoreAudio's
// NotSendSyncAcrossAllPlatforms.  We wrap it in a Mutex so concurrent access is impossible,
// and the pointer is only ever touched on the thread that locked the Mutex.
unsafe impl Send for AudioEngineMutex {}
unsafe impl Sync for AudioEngineMutex {}

#[tauri::command]
pub fn get_state(state: State<'_, AppStateMutex>) -> Result<AppState, String> {
    let app_state = state.0.lock().map_err(|e| format!("Lock error: {}", e))?;
    Ok(app_state.clone())
}

#[tauri::command]
pub fn toggle_sound(
    id: String,
    app_handle: AppHandle,
    state: State<'_, AppStateMutex>,
    audio: State<'_, AudioEngineMutex>,
) -> Result<AppState, String> {
    // Read info and update state immediately so the UI responds instantly
    let (was_active, file_path, volume, is_paused, crossfade) = {
        let mut app_state = state.0.lock().map_err(|e| format!("Lock error: {}", e))?;
        // Read all values first (immutable borrows)
        let is_paused = app_state.is_paused;
        let crossfade = app_state.crossfade_duration;
        let sound = app_state
            .sounds
            .iter()
            .find(|s| s.id == id)
            .ok_or_else(|| format!("Sound not found: {}", id))?;
        let was_active = sound.is_active;
        let file_path = sound.file_path.clone();
        let volume = sound.volume;
        // Now mutate
        let sound = app_state.sounds.iter_mut().find(|s| s.id == id).unwrap();
        sound.is_active = !was_active;
        let _ = save_state(&app_state);
        (was_active, file_path, volume, is_paused, crossfade)
    };

    // Stop is fast — do it inline
    if was_active {
        let mut audio_engine = audio.0.lock().map_err(|e| format!("Lock error: {}", e))?;
        audio_engine.stop_sound(&id);
    } else if !is_paused {
        // Decode + play is slow — spawn on background thread so UI doesn't freeze
        let id_clone = id.clone();
        std::thread::spawn(move || {
            let audio_state = app_handle.state::<AudioEngineMutex>();
            let mut audio_engine = audio_state.0.lock().unwrap();
            if let Err(e) = audio_engine.play_sound(&id_clone, &file_path, volume, crossfade) {
                eprintln!("Failed to play sound {}: {}", id_clone, e);
                // Revert state on error
                let state = app_handle.state::<AppStateMutex>();
                let mut app_state = state.0.lock().unwrap();
                if let Some(sound) = app_state.sounds.iter_mut().find(|s| s.id == id_clone) {
                    sound.is_active = false;
                }
                let _ = save_state(&app_state);
            }
        });
    }

    // Return updated state immediately — audio loads in background
    let app_state = state.0.lock().map_err(|e| format!("Lock error: {}", e))?;
    Ok(app_state.clone())
}

#[tauri::command]
pub fn set_volume(
    id: String,
    volume: f32,
    state: State<'_, AppStateMutex>,
    audio: State<'_, AudioEngineMutex>,
) -> Result<AppState, String> {
    let mut app_state = state.0.lock().map_err(|e| format!("Lock error: {}", e))?;
    let mut audio_engine = audio.0.lock().map_err(|e| format!("Lock error: {}", e))?;

    let sound = app_state
        .sounds
        .iter_mut()
        .find(|s| s.id == id)
        .ok_or_else(|| format!("Sound not found: {}", id))?;

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
) -> Result<AppState, String> {
    let mut app_state = state.0.lock().map_err(|e| format!("Lock error: {}", e))?;
    let mut audio_engine = audio.0.lock().map_err(|e| format!("Lock error: {}", e))?;

    app_state.master_volume = volume;
    audio_engine.set_master_volume(volume);

    save_state(&app_state)?;
    Ok(app_state.clone())
}

#[tauri::command]
pub fn pause_all(
    state: State<'_, AppStateMutex>,
    audio: State<'_, AudioEngineMutex>,
) -> Result<AppState, String> {
    let mut app_state = state.0.lock().map_err(|e| format!("Lock error: {}", e))?;
    let audio_engine = audio.0.lock().map_err(|e| format!("Lock error: {}", e))?;

    app_state.is_paused = true;
    audio_engine.pause_all();

    save_state(&app_state)?;
    Ok(app_state.clone())
}

#[tauri::command]
pub fn resume_all(
    state: State<'_, AppStateMutex>,
    audio: State<'_, AudioEngineMutex>,
) -> Result<AppState, String> {
    let mut app_state = state.0.lock().map_err(|e| format!("Lock error: {}", e))?;
    let audio_engine = audio.0.lock().map_err(|e| format!("Lock error: {}", e))?;

    app_state.is_paused = false;
    audio_engine.resume_all();

    save_state(&app_state)?;
    Ok(app_state.clone())
}

#[tauri::command]
pub fn import_sound(
    file_path: String,
    state: State<'_, AppStateMutex>,
) -> Result<AppState, String> {
    let src_path = PathBuf::from(&file_path);

    let ext = src_path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .ok_or("File has no extension")?;

    if ext != "mp3" && ext != "wav" {
        return Err(format!("Unsupported file type: {}", ext));
    }

    let dest_dir = user_sounds_dir();
    fs::create_dir_all(&dest_dir).map_err(|e| format!("Failed to create sounds dir: {}", e))?;

    // Determine destination filename, deduplicating with a UUID suffix if needed
    let stem = src_path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or("Invalid filename")?
        .to_string();

    let candidate = dest_dir.join(format!("{}.{}", stem, ext));
    let dest_path = if candidate.exists() {
        let unique_stem = format!("{}_{}", stem, Uuid::new_v4());
        dest_dir.join(format!("{}.{}", unique_stem, ext))
    } else {
        candidate
    };

    fs::copy(&src_path, &dest_path)
        .map_err(|e| format!("Failed to copy file: {}", e))?;

    let final_stem = dest_path
        .file_stem()
        .unwrap()
        .to_string_lossy()
        .to_string();

    let name = {
        let mut chars = final_stem.chars();
        match chars.next() {
            None => String::new(),
            Some(c) => c.to_uppercase().to_string() + chars.as_str(),
        }
    };

    let new_sound = Sound {
        id: format!("user_{}", final_stem),
        name,
        file_path: dest_path,
        is_bundled: false,
        is_active: false,
        volume: 0.7,
    };

    let mut app_state = state.0.lock().map_err(|e| format!("Lock error: {}", e))?;
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
) -> Result<AppState, String> {
    let mut app_state = state.0.lock().map_err(|e| format!("Lock error: {}", e))?;

    let sound = app_state
        .sounds
        .iter()
        .find(|s| s.id == id)
        .ok_or_else(|| format!("Sound not found: {}", id))?;

    if sound.is_bundled {
        return Err("Cannot remove bundled sounds".to_string());
    }

    let file_path = sound.file_path.clone();

    // Stop playback first
    {
        let mut audio_engine = audio.0.lock().map_err(|e| format!("Lock error: {}", e))?;
        audio_engine.stop_sound(&id);
    }

    // Remove the file from disk
    if file_path.exists() {
        fs::remove_file(&file_path)
            .map_err(|e| format!("Failed to delete sound file: {}", e))?;
    }

    app_state.sounds.retain(|s| s.id != id);

    save_state(&app_state)?;
    Ok(app_state.clone())
}

#[tauri::command]
pub fn set_crossfade_duration(
    duration: f32,
    state: State<'_, AppStateMutex>,
) -> Result<AppState, String> {
    let mut app_state = state.0.lock().map_err(|e| format!("Lock error: {}", e))?;

    app_state.crossfade_duration = duration;

    save_state(&app_state)?;
    Ok(app_state.clone())
}

#[tauri::command]
pub fn set_dialog_open(state: State<'_, AppStateMutex>) -> Result<(), String> {
    let mut app_state = state.0.lock().map_err(|e| format!("Lock error: {}", e))?;
    app_state.dialog_open = true;
    Ok(())
}

#[tauri::command]
pub fn set_dialog_closed(state: State<'_, AppStateMutex>) -> Result<(), String> {
    let mut app_state = state.0.lock().map_err(|e| format!("Lock error: {}", e))?;
    app_state.dialog_open = false;
    Ok(())
}
