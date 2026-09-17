//! Background auto-update: polls the release manifest, downloads a newer
//! version silently and tells the frontend once it only needs to be installed.
//!
//! The manifest lives at `plugins.updater.endpoints` in `tauri.conf.json`
//! (`latest.json` on the `main` branch, written by the release workflow).
//! Debug builds can point elsewhere via `RUSTBIRD_UPDATER_ENDPOINT` to test
//! the flow against a local manifest.

use crate::commands::{AppStateMutex, PendingUpdate};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_updater::{Update, Updater, UpdaterExt};

/// How often the manifest is polled while the app is running. A menubar app
/// stays open for weeks, so a startup-only check would rarely fire.
pub const CHECK_INTERVAL: Duration = Duration::from_secs(6 * 60 * 60);

/// Emitted with the new version string once an update has been downloaded and
/// signature-verified. The frontend then offers a one-click restart.
pub const UPDATE_READY_EVENT: &str = "update-ready";

/// An update whose package has already been downloaded and verified.
pub struct DownloadedUpdate {
    pub update: Update,
    pub bytes: Vec<u8>,
}

/// Start the periodic update check. Runs immediately, then every
/// [`CHECK_INTERVAL`]; each round respects the user's auto-update setting.
pub fn spawn_check_loop(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        loop {
            if auto_update_enabled(&app) {
                check_and_download(&app).await;
            }
            tokio::time::sleep(CHECK_INTERVAL).await;
        }
    });
}

/// One check round: fetch the manifest and, if a newer version is published,
/// download it into [`PendingUpdate`] and emit [`UPDATE_READY_EVENT`].
///
/// A version that is already downloaded (the user chose "later") is not
/// fetched again — the event is just re-emitted so the banner shows up again.
pub async fn check_and_download(app: &AppHandle) {
    let updater = match build_updater(app) {
        Ok(updater) => updater,
        Err(e) => {
            log::warn!("Updater not available: {e}");
            return;
        }
    };

    let update = match updater.check().await {
        Ok(Some(update)) => update,
        Ok(None) => {
            log::debug!("No update available");
            return;
        }
        Err(e) => {
            log::warn!("Update check failed: {e}");
            return;
        }
    };
    let version = update.version.clone();

    if !is_downloaded(app, &version) {
        log::info!("Downloading update {version}");
        let bytes = match update.download(|_, _| {}, || {}).await {
            Ok(bytes) => bytes,
            Err(e) => {
                log::warn!("Update download failed: {e}");
                return;
            }
        };
        if let Some(pending) = app.try_state::<PendingUpdate>() {
            *pending.0.lock().unwrap_or_else(|e| e.into_inner()) =
                Some(DownloadedUpdate { update, bytes });
        }
    }

    let _ = app.emit(UPDATE_READY_EVENT, version);
}

fn auto_update_enabled(app: &AppHandle) -> bool {
    app.try_state::<AppStateMutex>()
        .map(|state| {
            state
                .0
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .auto_update_enabled
        })
        .unwrap_or(false)
}

fn is_downloaded(app: &AppHandle, version: &str) -> bool {
    app.try_state::<PendingUpdate>()
        .map(|pending| {
            pending
                .0
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .as_ref()
                .is_some_and(|d| d.update.version == version)
        })
        .unwrap_or(false)
}

fn build_updater(app: &AppHandle) -> tauri_plugin_updater::Result<Updater> {
    let builder = app.updater_builder();

    // Dev-only escape hatch to test the flow against a local manifest.
    #[cfg(debug_assertions)]
    let builder = match std::env::var("RUSTBIRD_UPDATER_ENDPOINT") {
        Ok(url) => builder.endpoints(vec![url.parse()?])?,
        Err(_) => builder,
    };

    builder.build()
}
