mod audio;
mod commands;
mod crossfade;
mod error;
#[cfg(any(target_os = "macos", target_os = "windows"))]
mod lock_listener;
mod state;

use audio::AudioEngine;
use commands::{AppStateMutex, AudioEngineMutex, PendingUpdate};
use state::{discover_bundled_sounds, discover_user_sounds, load_persisted_state, AppState};
use std::sync::Mutex;
use tauri::{Emitter, Manager};
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
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
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

            let auto_update_enabled = app_state.auto_update_enabled;

            // Register managed state
            app.manage(AppStateMutex(Mutex::new(app_state)));
            app.manage(AudioEngineMutex(Mutex::new(audio_engine)));
            app.manage(PendingUpdate(Mutex::new(None)));

            // Spawn async update check if enabled
            if auto_update_enabled {
                let app_handle = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    use tauri_plugin_updater::UpdaterExt;
                    match app_handle.updater() {
                        Ok(updater) => match updater.check().await {
                            Ok(Some(update)) => {
                                let version = update.version.clone();
                                if let Some(pending) = app_handle.try_state::<PendingUpdate>() {
                                    *pending.0.lock().unwrap() = Some(update);
                                }
                                let _ = app_handle.emit("update-available", version);
                            }
                            Ok(None) => {}
                            Err(e) => log::warn!("Update check failed: {e}"),
                        },
                        Err(e) => log::warn!("Updater not available: {e}"),
                    }
                });
            }

            // Start screen-lock listener (macOS + Windows)
            #[cfg(any(target_os = "macos", target_os = "windows"))]
            lock_listener::start(app.handle().clone());

            // Tray menu (right-click) and popover (left-click).
            let quit_item =
                tauri::menu::MenuItemBuilder::with_id("quit", "Quit RustBird").build(app)?;
            let tray_menu = tauri::menu::MenuBuilder::new(app)
                .item(&quit_item)
                .build()?;

            if let Some(tray) = app.tray_by_id("main") {
                // On macOS the menu is deliberately NOT attached here: a menu that is
                // permanently attached to the NSStatusItem swallows the left click on
                // macOS 27 (AppKit pops the menu before tray-icon's overlay view sees
                // the event). It is attached only while shown — see `show_tray_menu`.
                #[cfg(not(target_os = "macos"))]
                tray.set_menu(Some(tray_menu.clone()))?;
                tray.set_show_menu_on_left_click(false)?;
            }

            app.on_tray_icon_event(move |app, event| {
                use tauri::tray::{MouseButton, MouseButtonState, TrayIconEvent};

                match event {
                    TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        rect,
                        ..
                    } => toggle_popover(app, rect),
                    TrayIconEvent::Click {
                        id,
                        button: MouseButton::Right,
                        button_state: MouseButtonState::Down,
                        ..
                    } => {
                        #[cfg(target_os = "macos")]
                        if let Some(tray) = app.tray_by_id(&id) {
                            show_tray_menu(&tray, &tray_menu);
                        }
                        // Elsewhere the menu is permanently attached and tray-icon
                        // pops it on its own.
                        #[cfg(not(target_os = "macos"))]
                        let _ = (id, &tray_menu);
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
            commands::set_auto_update_enabled,
            commands::install_update,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Show the tray context menu, attaching it to the status item only for the
/// duration of the popup.
///
/// macOS 27 stopped forwarding clicks to tray-icon's overlay view while an
/// `NSMenu` is attached to the `NSStatusItem`, so a permanently attached menu
/// swallows the left click and pops the menu instead
/// (tauri-apps/tray-icon#355). This mirrors upstream's fix (tray-icon 0.25.1,
/// PR #365: `setMenu(Some) → performClick → setMenu(None)`) at the app level
/// until a Tauri 2.x release pulls that version in. `show_menu` blocks until
/// the menu is dismissed, so detaching right after it returns is correct.
///
/// Runs inside the tray event handler, i.e. on the main thread; Tauri's
/// `run_on_main_thread` executes inline there, so this cannot deadlock.
#[cfg(target_os = "macos")]
fn show_tray_menu(tray: &tauri::tray::TrayIcon, menu: &tauri::menu::Menu<tauri::Wry>) {
    // A failed attach only degrades to "no menu this time"; never worth a panic.
    let _ = tray.set_menu(Some(menu.clone()));
    let _ = tray.with_inner_tray_icon(|inner| inner.show_menu());
    let _ = tray.set_menu(None::<tauri::menu::Menu<tauri::Wry>>);
}

/// Toggle the popover: hide it when visible, otherwise place it at the tray
/// icon and show it.
fn toggle_popover(app: &tauri::AppHandle, tray_rect: tauri::Rect) {
    let Some(window) = app.get_webview_window("main") else {
        return;
    };
    if window.is_visible().unwrap_or(false) {
        let _ = window.hide();
        return;
    }

    let scale = window.scale_factor().unwrap_or(1.0);
    let tray_pos = tray_rect.position.to_logical::<f64>(scale);
    let tray_size = tray_rect.size.to_logical::<f64>(scale);
    let win_size = window
        .outer_size()
        .unwrap_or_default()
        .to_logical::<f64>(scale);

    // Determine screen bounds so we can pick the right side of the tray.
    let (screen_pos, screen_size) = window
        .current_monitor()
        .ok()
        .flatten()
        .map(|m| {
            (
                m.position().to_logical::<f64>(scale),
                m.size().to_logical::<f64>(scale),
            )
        })
        .unwrap_or((
            tauri::LogicalPosition::new(0.0, 0.0),
            tauri::LogicalSize::new(1920.0, 1080.0),
        ));

    let pos = popover_position(tray_pos, tray_size, win_size, screen_pos, screen_size);
    let _ = window.set_position(pos);
    let _ = window.show();
    let _ = window.set_focus();
}

/// Where to place the popover so it hugs the tray icon and stays on-screen.
///
/// Shown below the tray when it sits in the upper screen half (macOS menu bar),
/// above it when in the lower half (Windows taskbar); horizontally centered on
/// the icon. Both axes are clamped to the monitor bounds.
fn popover_position(
    tray_pos: tauri::LogicalPosition<f64>,
    tray_size: tauri::LogicalSize<f64>,
    win_size: tauri::LogicalSize<f64>,
    screen_pos: tauri::LogicalPosition<f64>,
    screen_size: tauri::LogicalSize<f64>,
) -> tauri::LogicalPosition<f64> {
    let y = if tray_pos.y > screen_pos.y + screen_size.height / 2.0 {
        tray_pos.y - win_size.height
    } else {
        tray_pos.y + tray_size.height
    }
    .max(screen_pos.y)
    .min(screen_pos.y + screen_size.height - win_size.height);

    let x = (tray_pos.x - (win_size.width / 2.0) + (tray_size.width / 2.0))
        .max(screen_pos.x)
        .min(screen_pos.x + screen_size.width - win_size.width);

    tauri::LogicalPosition::new(x, y)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tauri::{LogicalPosition, LogicalSize};

    const WIN: LogicalSize<f64> = LogicalSize {
        width: 280.0,
        height: 420.0,
    };
    const PRIMARY_POS: LogicalPosition<f64> = LogicalPosition { x: 0.0, y: 0.0 };
    const PRIMARY_SIZE: LogicalSize<f64> = LogicalSize {
        width: 1920.0,
        height: 1080.0,
    };

    fn place(tray_x: f64, tray_y: f64, tray_h: f64) -> LogicalPosition<f64> {
        popover_position(
            LogicalPosition::new(tray_x, tray_y),
            LogicalSize::new(24.0, tray_h),
            WIN,
            PRIMARY_POS,
            PRIMARY_SIZE,
        )
    }

    // ── popover_position ────────────────────────────────────────────────

    #[test]
    fn popover_below_tray_on_menu_bar() {
        let pos = place(1000.0, 0.0, 24.0);
        assert_eq!(pos.y, 24.0);
        // Centered on the icon: 1000 - 140 + 12
        assert_eq!(pos.x, 872.0);
    }

    #[test]
    fn popover_above_tray_on_taskbar() {
        let pos = place(1000.0, 1050.0, 30.0);
        assert_eq!(pos.y, 1050.0 - 420.0);
        assert_eq!(pos.x, 872.0);
    }

    #[test]
    fn popover_clamped_to_right_edge() {
        let pos = place(1900.0, 0.0, 24.0);
        assert_eq!(pos.x, 1920.0 - 280.0);
        assert_eq!(pos.y, 24.0);
    }

    #[test]
    fn popover_clamped_to_left_edge() {
        let pos = place(10.0, 0.0, 24.0);
        assert_eq!(pos.x, 0.0);
        assert_eq!(pos.y, 24.0);
    }

    #[test]
    fn popover_respects_monitor_origin() {
        // Secondary monitor to the right of the primary one.
        let pos = popover_position(
            LogicalPosition::new(2900.0, 0.0),
            LogicalSize::new(24.0, 24.0),
            WIN,
            LogicalPosition::new(1920.0, 0.0),
            PRIMARY_SIZE,
        );
        assert_eq!(pos.x, 2772.0);
        assert_eq!(pos.y, 24.0);
    }
}
