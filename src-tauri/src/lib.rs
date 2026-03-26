mod audio;
mod commands;
mod crossfade;
mod state;

use audio::AudioEngine;
use commands::{AudioEngineMutex, AppStateMutex};
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

            // Discover bundled sounds from the app resources directory
            let mut sounds = Vec::new();

            if let Ok(resource_dir) = app.path().resource_dir() {
                let bundled_dir = resource_dir.join("assets").join("sounds");
                let mut bundled = discover_bundled_sounds(&bundled_dir);
                sounds.append(&mut bundled);
            }

            // Discover user-imported sounds
            let mut user_sounds = discover_user_sounds();
            sounds.append(&mut user_sounds);

            // Build initial app state
            let mut app_state = AppState {
                sounds,
                ..AppState::default()
            };

            // Apply persisted state if available
            if let Some(persisted) = load_persisted_state() {
                app_state.apply_persisted(&persisted);
            }

            // Initialize audio engine
            let mut audio_engine = AudioEngine::new().map_err(|e| {
                eprintln!("Failed to initialize audio engine: {}", e);
                Box::new(std::io::Error::new(std::io::ErrorKind::Other, e))
                    as Box<dyn std::error::Error>
            })?;

            // Set initial master volume
            audio_engine.set_master_volume(app_state.master_volume);

            // Resume any sounds that were active when the app was last closed
            if !app_state.is_paused {
                let crossfade = app_state.crossfade_duration;
                for sound in &app_state.sounds {
                    if sound.is_active {
                        if let Err(e) =
                            audio_engine.play_sound(&sound.id, &sound.file_path, sound.volume, crossfade)
                        {
                            eprintln!("Failed to resume sound {}: {}", sound.id, e);
                        }
                    }
                }
            }

            // Register managed state
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
