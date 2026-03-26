mod audio;
mod commands;
mod crossfade;
mod state;

use audio::AudioEngine;
use commands::{AudioEngineMutex, AppStateMutex};
use state::{discover_bundled_sounds, discover_user_sounds, load_persisted_state, AppState};
use std::sync::Mutex;
use tauri::Manager;
use tauri_plugin_autostart::ManagerExt;

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

            // Discover bundled sounds
            let mut sounds = Vec::new();

            // In production: sounds are in the app bundle's resource dir
            // In development: fall back to the project's assets/sounds/ directory
            let mut found_bundled = false;
            if let Ok(resource_dir) = app.path().resource_dir() {
                // Try several possible resource paths
                for subpath in &["sounds", "assets/sounds"] {
                    let dir = resource_dir.join(subpath);
                    let bundled = discover_bundled_sounds(&dir);
                    if !bundled.is_empty() {
                        sounds.extend(bundled);
                        found_bundled = true;
                        break;
                    }
                }
            }

            // Dev fallback: check relative to the Cargo manifest directory
            if !found_bundled {
                let dev_sounds = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .parent()
                    .map(|p| p.join("assets").join("sounds"))
                    .unwrap_or_default();
                let bundled = discover_bundled_sounds(&dev_sounds);
                sounds.extend(bundled);
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

            // Enable autostart if configured
            if app_state.autostart_enabled {
                let _ = app.autolaunch().enable();
            }

            // Register managed state
            app.manage(AppStateMutex(Mutex::new(app_state)));
            app.manage(AudioEngineMutex(Mutex::new(audio_engine)));

            // Build the context menu (but don't attach it permanently)
            let quit_item = tauri::menu::MenuItemBuilder::with_id("quit", "Quit BackBird")
                .build(app)?;
            let tray_menu = tauri::menu::MenuBuilder::new(app)
                .item(&quit_item)
                .build()?;

            // Set up tray icon events: left-click → popover, right-click → context menu
            let app_handle = app.handle().clone();
            app.on_tray_icon_event(move |tray, event| {
                use tauri::tray::{MouseButton, MouseButtonState, TrayIconEvent};

                match event {
                    TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        rect,
                        ..
                    } => {
                        // Left click → toggle popover
                        if let Some(window) = app_handle.get_webview_window("main") {
                            if window.is_visible().unwrap_or(false) {
                                let _ = window.hide();
                            } else {
                                let tray_pos = rect.position.to_logical::<f64>(1.0);
                                let tray_size = rect.size.to_logical::<f64>(1.0);
                                let scale = window.scale_factor().unwrap_or(1.0);
                                let win_size = window.outer_size().unwrap_or_default()
                                    .to_logical::<f64>(scale);
                                let x = tray_pos.x - (win_size.width / 2.0) + (tray_size.width / 2.0);
                                let y = tray_pos.y + tray_size.height;
                                let _ = window.set_position(tauri::LogicalPosition::new(x, y));
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                    }
                    TrayIconEvent::Click {
                        button: MouseButton::Right,
                        button_state: MouseButtonState::Up,
                        ..
                    } => {
                        // Right click → show context menu
                        let _ = tray.set_menu(tray_menu.clone());
                    }
                    _ => {}
                }
            });

            Ok(())
        })
        .on_menu_event(|app, event| {
            if event.id().as_ref() == "quit" {
                // Save state before quitting
                if let Some(state) = app.try_state::<AppStateMutex>() {
                    let app_state = state.0.lock().unwrap();
                    let _ = crate::state::save_state(&app_state);
                }
                app.exit(0);
            }
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::Focused(false) = event {
                if window.label() == "main" {
                    let _ = window.hide();
                }
            }
            if let tauri::WindowEvent::Destroyed = event {
                if window.label() == "main" {
                    if let Some(state) = window.try_state::<AppStateMutex>() {
                        let app_state = state.0.lock().unwrap();
                        let _ = crate::state::save_state(&app_state);
                    }
                }
            }
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
