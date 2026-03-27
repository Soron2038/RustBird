mod audio;
mod commands;
mod crossfade;
mod error;
#[cfg(target_os = "macos")]
mod lock_listener;
mod state;

use audio::AudioEngine;
use commands::{AppStateMutex, AudioEngineMutex};
use state::{discover_bundled_sounds, discover_user_sounds, load_persisted_state, AppState};
use std::sync::Mutex;
use tauri::Manager;
use tauri_plugin_autostart::ManagerExt;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

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
            let mut audio_engine = AudioEngine::new()?;

            // Set initial master volume
            audio_engine.set_master_volume(app_state.master_volume);

            // Resume any sounds that were active when the app was last closed
            if !app_state.is_paused {
                let crossfade = app_state.crossfade_duration;
                for sound in &app_state.sounds {
                    if sound.is_active {
                        if let Err(e) = audio_engine.play_sound(
                            &sound.id,
                            &sound.file_path,
                            sound.volume,
                            crossfade,
                        ) {
                            log::error!("Failed to resume sound {}: {}", sound.id, e);
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

            // Start macOS screen-lock listener
            #[cfg(target_os = "macos")]
            lock_listener::start(app.handle().clone());

            // Set up tray menu (right-click only) and popover (left-click)
            let quit_item =
                tauri::menu::MenuItemBuilder::with_id("quit", "Quit RustBird").build(app)?;
            let tray_menu = tauri::menu::MenuBuilder::new(app)
                .item(&quit_item)
                .build()?;

            // Attach menu to tray but disable it on left-click
            if let Some(tray) = app.tray_by_id("main") {
                tray.set_menu(Some(tray_menu))?;
                tray.set_show_menu_on_left_click(false)?;
            }

            // Left-click toggles the popover window
            let app_handle = app.handle().clone();
            app.on_tray_icon_event(move |_tray, event| {
                use tauri::tray::{MouseButton, MouseButtonState, TrayIconEvent};

                if let TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    rect,
                    ..
                } = event
                {
                    if let Some(window) = app_handle.get_webview_window("main") {
                        if window.is_visible().unwrap_or(false) {
                            let _ = window.hide();
                        } else {
                            let scale = window.scale_factor().unwrap_or(1.0);
                            let tray_pos = rect.position.to_logical::<f64>(scale);
                            let tray_size = rect.size.to_logical::<f64>(scale);
                            let win_size = window
                                .outer_size()
                                .unwrap_or_default()
                                .to_logical::<f64>(scale);
                            let x = tray_pos.x - (win_size.width / 2.0) + (tray_size.width / 2.0);
                            let y = tray_pos.y + tray_size.height;
                            let _ = window.set_position(tauri::LogicalPosition::new(x, y));
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                }
            });

            Ok(())
        })
        .on_menu_event(|app, event| {
            if event.id().as_ref() == "quit" {
                // Save state before quitting
                if let Some(state) = app.try_state::<AppStateMutex>() {
                    let app_state = state.0.lock().unwrap_or_else(|e| e.into_inner());
                    let _ = crate::state::save_state(&app_state);
                }
                app.exit(0);
            }
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::Focused(false) = event {
                if window.label() == "main" {
                    let should_hide = window
                        .try_state::<AppStateMutex>()
                        .map(|state| {
                            !state
                                .0
                                .lock()
                                .unwrap_or_else(|e| e.into_inner())
                                .dialog_open
                        })
                        .unwrap_or(true);
                    if should_hide {
                        let _ = window.hide();
                    }
                }
            }
            if let tauri::WindowEvent::Destroyed = event {
                if window.label() == "main" {
                    if let Some(state) = window.try_state::<AppStateMutex>() {
                        let app_state = state.0.lock().unwrap_or_else(|e| e.into_inner());
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
            commands::set_dialog_open,
            commands::set_dialog_closed,
            commands::set_autopause_on_lock,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
