use crate::audio::AudioEngine;
use crate::state::{save_state, user_sounds_dir, AppState, Sound};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::State;
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
    state: State<'_, AppStateMutex>,
    audio: State<'_, AudioEngineMutex>,
) -> Result<AppState, String> {
    let mut app_state = state.0.lock().map_err(|e| format!("Lock error: {}", e))?;
    let mut audio_engine = audio.0.lock().map_err(|e| format!("Lock error: {}", e))?;

    // Collect the values we need before taking the mutable borrow
    let (was_active, file_path, volume) = {
        let sound = app_state
            .sounds
            .iter()
            .find(|s| s.id == id)
            .ok_or_else(|| format!("Sound not found: {}", id))?;
        (sound.is_active, sound.file_path.clone(), sound.volume)
    };

    let is_paused = app_state.is_paused;
    let crossfade = app_state.crossfade_duration;

    // Now mutate the sound
    let sound = app_state
        .sounds
        .iter_mut()
        .find(|s| s.id == id)
        .unwrap(); // safe: we found it above

    if was_active {
        sound.is_active = false;
        audio_engine.stop_sound(&id);
    } else {
        sound.is_active = true;
        if !is_paused {
            audio_engine.play_sound(&id, &file_path, volume, crossfade)?;
        }
    }

    save_state(&app_state)?;
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
